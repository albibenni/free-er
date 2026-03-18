use crate::ipc_client;
use crate::sections::{
    allowed_lists::{AllowedListsInput, AllowedListsOutput, AllowedListsSection},
    focus::{FocusInput, FocusOutput, FocusSection},
    pomodoro::{PomodoroInput, PomodoroOutput, PomodoroSection},
    schedule::{ScheduleInput, ScheduleOutput, ScheduleSection},
    settings::{SettingsInput, SettingsOutput, SettingsSection, AI_SITES, SEARCH_ENGINES},
};
use gtk4::prelude::*;
use relm4::prelude::*;
use shared::ipc::{Command, ScheduleSummary, ScheduleType};
use tracing::{error, warn};
use uuid::Uuid;

#[derive(Debug)]
pub enum Page {
    Focus,
    AllowedLists,
    Pomodoro,
    Schedule,
    Settings,
}

pub struct App {
    current_page: Page,
    /// ID of the first (default) rule set; used by Settings quick-toggles.
    default_rule_set_id: Option<Uuid>,
    focus: Controller<FocusSection>,
    pomodoro: Controller<PomodoroSection>,
    allowed_lists: Controller<AllowedListsSection>,
    schedule: Controller<ScheduleSection>,
    settings: Controller<SettingsSection>,
}

#[derive(Debug)]
pub enum AppMsg {
    Navigate(Page),
    // Forwarded from child components
    StartFocus,
    StopFocus,
    SkipBreak,
    StartPomodoro { focus_secs: u64, break_secs: u64, rule_set_id: Option<Uuid> },
    StopPomodoro,
    /// Add URL to the default rule set (used by Settings quick-toggles).
    AddUrl(String),
    /// Remove URL from the default rule set (used by Settings quick-toggles).
    RemoveUrl(String),
    /// Add a URL to a specific rule set (from AllowedLists section).
    AddUrlToList { rule_set_id: Uuid, url: String },
    /// Remove a URL from a specific rule set (from AllowedLists section).
    RemoveUrlFromList { rule_set_id: Uuid, url: String },
    /// Create a new rule set with the given name.
    CreateRuleSet(String),
    /// Delete an existing rule set.
    DeleteRuleSet(Uuid),
    ConnectGoogle,
    DisconnectGoogle,
    StrictModeChanged(bool),
    AllowNewTabChanged(bool),
    AiSitesToggled(bool),
    SearchEnginesToggled(bool),
    SaveCalDav { url: String, user: String, pass: String },
    // Periodic status poll result
    StatusTick,
    // Internal: schedules fetched from daemon
    SchedulesUpdated(Vec<ScheduleSummary>),
    CreateSchedule {
        name: String,
        days: Vec<u8>,
        start_min: u32,
        end_min: u32,
        specific_date: String,
        rule_set_id: Option<Uuid>,
        schedule_type: ScheduleType,
    },
    UpdateSchedule {
        id: Uuid,
        name: String,
        days: Vec<u8>,
        start_min: u32,
        end_min: u32,
        rule_set_id: Option<Uuid>,
        specific_date: Option<String>,
        schedule_type: ScheduleType,
    },
    DeleteSchedule(Uuid),
    RefreshSchedules,
    RefreshRuleSets,
    SetDefaultRuleSet(Uuid),
}

#[relm4::component(pub)]
impl Component for App {
    type Init = ();
    type Input = AppMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        gtk4::ApplicationWindow {
            set_title: Some("free-er"),
            set_default_size: (800, 550),

            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,

                // ── Sidebar ──────────────────────────────────────────────
                gtk4::Box {
                    set_orientation: gtk4::Orientation::Vertical,
                    set_width_request: 160,
                    add_css_class: "sidebar",
                    set_spacing: 4,
                    set_margin_all: 8,

                    gtk4::Button {
                        set_label: "Focus",
                        connect_clicked => AppMsg::Navigate(Page::Focus),
                    },
                    gtk4::Button {
                        set_label: "Allowed Lists",
                        connect_clicked => AppMsg::Navigate(Page::AllowedLists),
                    },
                    gtk4::Button {
                        set_label: "Pomodoro",
                        connect_clicked => AppMsg::Navigate(Page::Pomodoro),
                    },
                    gtk4::Button {
                        set_label: "Schedule",
                        connect_clicked => AppMsg::Navigate(Page::Schedule),
                    },
                    gtk4::Button {
                        set_label: "Settings",
                        connect_clicked => AppMsg::Navigate(Page::Settings),
                    },
                },

                gtk4::Separator { set_orientation: gtk4::Orientation::Vertical },

                // ── Content area ─────────────────────────────────────────
                #[name = "stack"]
                gtk4::Stack {
                    set_hexpand: true,
                    set_vexpand: true,
                },
            },
        }
    }

    fn init(_: (), root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let focus = FocusSection::builder()
            .launch(())
            .forward(sender.input_sender(), |out| match out {
                FocusOutput::StartFocus => AppMsg::StartFocus,
                FocusOutput::StopFocus => AppMsg::StopFocus,
                FocusOutput::SkipBreak => AppMsg::SkipBreak,
            });

        let pomodoro = PomodoroSection::builder()
            .launch(())
            .forward(sender.input_sender(), |out| match out {
                PomodoroOutput::Start { focus_secs, break_secs, rule_set_id } => {
                    AppMsg::StartPomodoro { focus_secs, break_secs, rule_set_id }
                }
                PomodoroOutput::Stop => AppMsg::StopPomodoro,
            });

        let allowed_lists = AllowedListsSection::builder()
            .launch(())
            .forward(sender.input_sender(), |out| match out {
                AllowedListsOutput::AddUrl { rule_set_id, url } => {
                    AppMsg::AddUrlToList { rule_set_id, url }
                }
                AllowedListsOutput::RemoveUrl { rule_set_id, url } => {
                    AppMsg::RemoveUrlFromList { rule_set_id, url }
                }
                AllowedListsOutput::CreateRuleSet(name) => AppMsg::CreateRuleSet(name),
                AllowedListsOutput::DeleteRuleSet(id) => AppMsg::DeleteRuleSet(id),
            });

        let schedule = ScheduleSection::builder()
            .launch(())
            .forward(sender.input_sender(), |out| match out {
                ScheduleOutput::CreateSchedule { name, days, start_min, end_min, specific_date, rule_set_id, schedule_type } => {
                    AppMsg::CreateSchedule { name, days, start_min, end_min, specific_date, rule_set_id, schedule_type }
                }
                ScheduleOutput::UpdateSchedule { id, name, days, start_min, end_min, rule_set_id, specific_date, schedule_type } => {
                    AppMsg::UpdateSchedule { id, name, days, start_min, end_min, rule_set_id, specific_date, schedule_type }
                }
                ScheduleOutput::DeleteSchedule(id) => AppMsg::DeleteSchedule(id),
            });

        let settings = SettingsSection::builder()
            .launch(false)
            .forward(sender.input_sender(), |out| match out {
                SettingsOutput::StrictModeChanged(v) => AppMsg::StrictModeChanged(v),
                SettingsOutput::AllowNewTabChanged(v) => AppMsg::AllowNewTabChanged(v),
                SettingsOutput::AiSitesToggled(v) => AppMsg::AiSitesToggled(v),
                SettingsOutput::SearchEnginesToggled(v) => AppMsg::SearchEnginesToggled(v),
                SettingsOutput::QuickUrlToggled { url, enabled } => {
                    if enabled { AppMsg::AddUrl(url.to_string()) } else { AppMsg::RemoveUrl(url.to_string()) }
                }
                SettingsOutput::CalDavSaved { url, user, pass } => {
                    AppMsg::SaveCalDav { url, user, pass }
                }
                SettingsOutput::ConnectGoogleRequested => AppMsg::ConnectGoogle,
                SettingsOutput::DisconnectGoogleRequested => AppMsg::DisconnectGoogle,
            });

        let model = App {
            current_page: Page::Focus,
            default_rule_set_id: None,
            focus,
            pomodoro,
            allowed_lists,
            schedule,
            settings,
        };

        let widgets = view_output!();

        widgets.stack.add_named(model.focus.widget(), Some("focus"));
        widgets.stack.add_named(model.allowed_lists.widget(), Some("allowed_lists"));
        widgets.stack.add_named(model.pomodoro.widget(), Some("pomodoro"));
        widgets.stack.add_named(model.schedule.widget(), Some("schedule"));
        widgets.stack.add_named(model.settings.widget(), Some("settings"));

        // Poll daemon status every 2 seconds
        let tick_sender = sender.clone();
        gtk4::glib::timeout_add_seconds_local(2, move || {
            tick_sender.input(AppMsg::StatusTick);
            gtk4::glib::ControlFlow::Continue
        });

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: AppMsg,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            AppMsg::Navigate(page) => {
                let name = match page {
                    Page::Focus => "focus",
                    Page::AllowedLists => "allowed_lists",
                    Page::Pomodoro => "pomodoro",
                    Page::Schedule => "schedule",
                    Page::Settings => "settings",
                };
                widgets.stack.set_visible_child_name(name);
                self.current_page = page;
            }

            AppMsg::StartFocus => {
                let rule_set_id = self.default_rule_set_id.unwrap_or_else(Uuid::nil);
                tokio::spawn(async move {
                    if let Err(e) = ipc_client::send(&Command::StartFocus { rule_set_id }).await {
                        error!("StartFocus IPC failed: {e}");
                    }
                });
            }
            AppMsg::StopFocus => {
                tokio::spawn(async {
                    if let Err(e) = ipc_client::send(&Command::StopFocus).await {
                        error!("StopFocus IPC failed: {e}");
                    }
                });
            }
            AppMsg::SkipBreak => {
                tokio::spawn(async {
                    if let Err(e) = ipc_client::send(&Command::SkipBreak).await {
                        error!("SkipBreak IPC failed: {e}");
                    }
                });
            }
            AppMsg::StartPomodoro { focus_secs, break_secs, rule_set_id } => {
                tokio::spawn(async move {
                    if let Err(e) = ipc_client::send(&Command::StartPomodoro {
                        focus_secs,
                        break_secs,
                        rule_set_id,
                    }).await {
                        error!("StartPomodoro IPC failed: {e}");
                    }
                });
            }
            AppMsg::StopPomodoro => {
                tokio::spawn(async {
                    if let Err(e) = ipc_client::send(&Command::StopPomodoro).await {
                        error!("StopPomodoro IPC failed: {e}");
                    }
                });
            }

            // Quick-toggle from Settings: target the default rule set
            AppMsg::AddUrl(url) => {
                let existing_id = self.default_rule_set_id;
                let inner_sender = _sender.clone();
                tokio::spawn(async move {
                    if let Some(id) = existing_id {
                        if let Err(e) = ipc_client::send(&Command::AddUrlToRuleSet {
                            rule_set_id: id,
                            url,
                        }).await {
                            error!("AddUrlToRuleSet IPC failed: {e}");
                        }
                    } else {
                        match ipc_client::add_rule_set("Default").await {
                            Ok(id) => {
                                inner_sender.input(AppMsg::RefreshRuleSets);
                                if let Err(e) = ipc_client::send(&Command::AddUrlToRuleSet {
                                    rule_set_id: id,
                                    url,
                                }).await {
                                    error!("AddUrlToRuleSet IPC failed: {e}");
                                }
                            }
                            Err(e) => error!("AddRuleSet IPC failed: {e}"),
                        }
                    }
                });
            }
            AppMsg::RemoveUrl(url) => {
                if let Some(id) = self.default_rule_set_id {
                    tokio::spawn(async move {
                        if let Err(e) = ipc_client::send(&Command::RemoveUrlFromRuleSet {
                            rule_set_id: id,
                            url,
                        }).await {
                            error!("RemoveUrlFromRuleSet IPC failed: {e}");
                        }
                    });
                }
            }

            AppMsg::AddUrlToList { rule_set_id, url } => {
                tokio::spawn(async move {
                    if let Err(e) = ipc_client::send(&Command::AddUrlToRuleSet {
                        rule_set_id,
                        url,
                    }).await {
                        error!("AddUrlToRuleSet IPC failed: {e}");
                    }
                });
            }
            AppMsg::RemoveUrlFromList { rule_set_id, url } => {
                tokio::spawn(async move {
                    if let Err(e) = ipc_client::send(&Command::RemoveUrlFromRuleSet {
                        rule_set_id,
                        url,
                    }).await {
                        error!("RemoveUrlFromRuleSet IPC failed: {e}");
                    }
                });
            }

            AppMsg::CreateRuleSet(name) => {
                let refresh = _sender.clone();
                tokio::spawn(async move {
                    match ipc_client::add_rule_set(&name).await {
                        Ok(_) => refresh.input(AppMsg::RefreshRuleSets),
                        Err(e) => error!("AddRuleSet IPC failed: {e}"),
                    }
                });
            }
            AppMsg::DeleteRuleSet(id) => {
                let refresh = _sender.clone();
                tokio::spawn(async move {
                    match ipc_client::remove_rule_set(id).await {
                        Ok(_) => refresh.input(AppMsg::RefreshRuleSets),
                        Err(e) => error!("RemoveRuleSet IPC failed: {e}"),
                    }
                });
            }

            AppMsg::ConnectGoogle => {
                tokio::spawn(async {
                    match ipc_client::start_google_oauth().await {
                        Ok(url) => {
                            let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
                        }
                        Err(e) => error!("Google OAuth failed: {e}"),
                    }
                });
            }
            AppMsg::DisconnectGoogle => {
                tokio::spawn(async {
                    if let Err(e) = ipc_client::revoke_google_calendar().await {
                        error!("RevokeGoogleCalendar IPC failed: {e}");
                    }
                });
            }
            AppMsg::StrictModeChanged(enabled) => {
                tokio::spawn(async move {
                    if let Err(e) = ipc_client::send(&Command::SetStrictMode { enabled }).await {
                        error!("SetStrictMode IPC failed: {e}");
                    }
                });
            }
            AppMsg::AllowNewTabChanged(enabled) => {
                tokio::spawn(async move {
                    if let Err(e) = ipc_client::send(&Command::SetAllowNewTab { enabled }).await {
                        error!("SetAllowNewTab IPC failed: {e}");
                    }
                });
            }
            AppMsg::AiSitesToggled(enabled) => {
                let rule_set_id = self.default_rule_set_id;
                let inner_sender = _sender.clone();
                tokio::spawn(async move {
                    let id = if let Some(id) = rule_set_id {
                        id
                    } else {
                        match ipc_client::add_rule_set("Default").await {
                            Ok(id) => { inner_sender.input(AppMsg::RefreshRuleSets); id }
                            Err(e) => { error!("AddRuleSet IPC failed: {e}"); return; }
                        }
                    };
                    for url in AI_SITES {
                        let cmd = if enabled {
                            Command::AddUrlToRuleSet { rule_set_id: id, url: url.to_string() }
                        } else {
                            Command::RemoveUrlFromRuleSet { rule_set_id: id, url: url.to_string() }
                        };
                        if let Err(e) = ipc_client::send(&cmd).await {
                            error!("AI sites toggle IPC failed: {e}");
                        }
                    }
                });
            }
            AppMsg::SearchEnginesToggled(enabled) => {
                let rule_set_id = self.default_rule_set_id;
                let inner_sender = _sender.clone();
                tokio::spawn(async move {
                    let id = if let Some(id) = rule_set_id {
                        id
                    } else {
                        match ipc_client::add_rule_set("Default").await {
                            Ok(id) => { inner_sender.input(AppMsg::RefreshRuleSets); id }
                            Err(e) => { error!("AddRuleSet IPC failed: {e}"); return; }
                        }
                    };
                    for url in SEARCH_ENGINES {
                        let cmd = if enabled {
                            Command::AddUrlToRuleSet { rule_set_id: id, url: url.to_string() }
                        } else {
                            Command::RemoveUrlFromRuleSet { rule_set_id: id, url: url.to_string() }
                        };
                        if let Err(e) = ipc_client::send(&cmd).await {
                            error!("Search engines toggle IPC failed: {e}");
                        }
                    }
                });
            }
            AppMsg::SaveCalDav { url, user, pass } => {
                tokio::spawn(async move {
                    if let Err(e) = ipc_client::send(&Command::SetCalDav {
                        url,
                        username: user,
                        password: pass,
                    }).await {
                        error!("SetCalDav IPC failed: {e}");
                    }
                });
            }

            AppMsg::SchedulesUpdated(schedules) => {
                self.schedule.sender().emit(ScheduleInput::SchedulesUpdated(schedules));
            }

            AppMsg::CreateSchedule { name, days, start_min, end_min, specific_date, rule_set_id, schedule_type } => {
                let refresh = _sender.clone();
                tokio::spawn(async move {
                    match ipc_client::add_schedule(&name, days, start_min, end_min, Some(specific_date), rule_set_id, schedule_type).await {
                        Ok(_) => refresh.input(AppMsg::RefreshSchedules),
                        Err(e) => error!("add_schedule failed: {e}"),
                    }
                });
            }
            AppMsg::UpdateSchedule { id, name, days, start_min, end_min, rule_set_id, specific_date, schedule_type } => {
                let refresh = _sender.clone();
                tokio::spawn(async move {
                    match ipc_client::update_schedule(id, &name, days, start_min, end_min, rule_set_id, specific_date, schedule_type).await {
                        Ok(_) => refresh.input(AppMsg::RefreshSchedules),
                        Err(e) => error!("update_schedule failed: {e}"),
                    }
                });
            }
            AppMsg::DeleteSchedule(id) => {
                let refresh = _sender.clone();
                tokio::spawn(async move {
                    match ipc_client::remove_schedule(id).await {
                        Ok(_) => refresh.input(AppMsg::RefreshSchedules),
                        Err(e) => error!("remove_schedule failed: {e}"),
                    }
                });
            }
            AppMsg::RefreshSchedules => {
                let tick_sender = _sender.clone();
                tokio::spawn(async move {
                    match ipc_client::list_schedules().await {
                        Ok(schedules) => tick_sender.input(AppMsg::SchedulesUpdated(schedules)),
                        Err(e) => warn!("list_schedules failed: {e}"),
                    }
                });
            }
            AppMsg::RefreshRuleSets => {
                let lists_sender = self.allowed_lists.sender().clone();
                let pom_sender = self.pomodoro.sender().clone();
                let sched_sender = self.schedule.sender().clone();
                let settings_sender = self.settings.sender().clone();
                let tick_sender = _sender.clone();
                tokio::spawn(async move {
                    match ipc_client::list_rule_sets().await {
                        Ok(sets) => {
                            lists_sender.emit(AllowedListsInput::RuleSetsUpdated(sets.clone()));
                            pom_sender.emit(PomodoroInput::RuleSetsUpdated(sets.clone()));
                            sched_sender.emit(ScheduleInput::RuleSetsUpdated(sets.clone()));
                            let all_urls: Vec<String> = sets.iter()
                                .flat_map(|s| s.allowed_urls.clone())
                                .collect();
                            settings_sender.emit(SettingsInput::QuickUrlsUpdated(all_urls));
                            if let Some(first_id) = sets.first().map(|s| s.id) {
                                tick_sender.input(AppMsg::SetDefaultRuleSet(first_id));
                            }
                        }
                        Err(e) => warn!("list_rule_sets failed: {e}"),
                    }
                });
            }
            AppMsg::SetDefaultRuleSet(id) => {
                self.default_rule_set_id = Some(id);
            }

            AppMsg::StatusTick => {
                let focus_sender = self.focus.sender().clone();
                let pom_sender = self.pomodoro.sender().clone();
                let lists_sender = self.allowed_lists.sender().clone();
                let settings_sender = self.settings.sender().clone();
                let schedule_sender = self.schedule.sender().clone();
                let tick_sender = _sender.clone();
                tokio::spawn(async move {
                    match ipc_client::get_status().await {
                        Ok(status) => {
                            focus_sender.emit(FocusInput::StatusUpdated {
                                active: status.focus_active,
                                rule_set: status.active_rule_set_name,
                            });
                            pom_sender.emit(PomodoroInput::StatusUpdated {
                                phase: status.pomodoro_phase.map(|p| format!("{p:?}")),
                                seconds_remaining: status.seconds_remaining,
                            });
                            settings_sender.emit(SettingsInput::GoogleStatusUpdated(
                                status.google_calendar_connected,
                            ));
                            settings_sender.emit(SettingsInput::AllowNewTabUpdated(
                                status.allow_new_tab,
                            ));
                        }
                        Err(e) => warn!("status poll failed: {e}"),
                    }
                    match ipc_client::list_rule_sets().await {
                        Ok(sets) => {
                            lists_sender.emit(AllowedListsInput::RuleSetsUpdated(sets.clone()));
                            pom_sender.emit(PomodoroInput::RuleSetsUpdated(sets.clone()));
                            schedule_sender.emit(ScheduleInput::RuleSetsUpdated(sets.clone()));
                            let all_urls: Vec<String> = sets.iter()
                                .flat_map(|s| s.allowed_urls.clone())
                                .collect();
                            settings_sender.emit(SettingsInput::QuickUrlsUpdated(all_urls));
                            if let Some(first_id) = sets.first().map(|s| s.id) {
                                tick_sender.input(AppMsg::SetDefaultRuleSet(first_id));
                            }
                        }
                        Err(e) => warn!("list_rule_sets failed: {e}"),
                    }
                    match ipc_client::list_schedules().await {
                        Ok(schedules) => {
                            tick_sender.input(AppMsg::SchedulesUpdated(schedules));
                        }
                        Err(e) => warn!("list_schedules failed: {e}"),
                    }
                });
            }
        }
    }
}
