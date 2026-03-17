use crate::ipc_client;
use crate::sections::{
    allowed_lists::{AllowedListsInput, AllowedListsOutput, AllowedListsSection},
    focus::{FocusInput, FocusOutput, FocusSection},
    pomodoro::{PomodoroInput, PomodoroOutput, PomodoroSection},
    schedule::{ScheduleInput, ScheduleOutput, ScheduleSection},
    settings::{SettingsInput, SettingsOutput, SettingsSection},
};
use gtk4::prelude::*;
use relm4::prelude::*;
use shared::ipc::{Command, ScheduleSummary};
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
    /// ID of the rule set used when starting a focus session.
    /// Populated from ListRuleSets; defaults to nil until the first poll.
    active_rule_set_id: Option<Uuid>,
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
    StartPomodoro { focus_secs: u64, break_secs: u64 },
    StopPomodoro,
    AddUrl(String),
    RemoveUrl(String),
    ConnectGoogle,
    DisconnectGoogle,
    StrictModeChanged(bool),
    SaveCalDav { url: String, user: String, pass: String },
    // Periodic status poll result
    StatusTick,
    // Internal: rule sets fetched from daemon
    RuleSetsUpdated(Vec<Uuid>),
    // Internal: a new rule set was created, store its ID
    RuleSetCreated(Uuid),
    // Internal: schedules fetched from daemon
    SchedulesUpdated(Vec<ScheduleSummary>),
    CreateSchedule { name: String, days: Vec<u8>, start_min: u32, end_min: u32, specific_date: String },
    UpdateSchedule { id: Uuid, name: String, days: Vec<u8>, start_min: u32, end_min: u32 },
    DeleteSchedule(Uuid),
    RefreshSchedules,
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
                PomodoroOutput::Start { focus_secs, break_secs } => {
                    AppMsg::StartPomodoro { focus_secs, break_secs }
                }
                PomodoroOutput::Stop => AppMsg::StopPomodoro,
            });

        let allowed_lists = AllowedListsSection::builder()
            .launch(())
            .forward(sender.input_sender(), |out| match out {
                AllowedListsOutput::AddUrl(url) => AppMsg::AddUrl(url),
                AllowedListsOutput::RemoveUrl(url) => AppMsg::RemoveUrl(url),
            });

        let schedule = ScheduleSection::builder()
            .launch(())
            .forward(sender.input_sender(), |out| match out {
                ScheduleOutput::CreateSchedule { name, days, start_min, end_min, specific_date } => {
                    AppMsg::CreateSchedule { name, days, start_min, end_min, specific_date }
                }
                ScheduleOutput::UpdateSchedule { id, name, days, start_min, end_min } => {
                    AppMsg::UpdateSchedule { id, name, days, start_min, end_min }
                }
                ScheduleOutput::DeleteSchedule(id) => AppMsg::DeleteSchedule(id),
            });

        let settings = SettingsSection::builder()
            .launch(false)
            .forward(sender.input_sender(), |out| match out {
                SettingsOutput::StrictModeChanged(v) => AppMsg::StrictModeChanged(v),
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
            active_rule_set_id: None,
            focus,
            pomodoro,
            allowed_lists,
            schedule,
            settings,
        };

        let widgets = view_output!();

        // Add child pages to the Stack now that we have both widgets and model
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
                let rule_set_id = self.active_rule_set_id.unwrap_or_else(Uuid::nil);
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
            AppMsg::StartPomodoro { focus_secs, break_secs } => {
                tokio::spawn(async move {
                    if let Err(e) = ipc_client::send(&Command::StartPomodoro { focus_secs, break_secs }).await {
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
            AppMsg::AddUrl(url) => {
                let existing_id = self.active_rule_set_id;
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
                        match ipc_client::add_rule_set("My List").await {
                            Ok(id) => {
                                inner_sender.input(AppMsg::RuleSetCreated(id));
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
                if let Some(id) = self.active_rule_set_id {
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

            AppMsg::RuleSetsUpdated(ids) => {
                if self.active_rule_set_id.is_none() {
                    self.active_rule_set_id = ids.into_iter().next();
                }
            }
            AppMsg::RuleSetCreated(id) => {
                self.active_rule_set_id = Some(id);
            }
            AppMsg::SchedulesUpdated(schedules) => {
                self.schedule.sender().emit(ScheduleInput::SchedulesUpdated(schedules));
            }

            AppMsg::CreateSchedule { name, days, start_min, end_min, specific_date } => {
                let refresh = _sender.clone();
                tokio::spawn(async move {
                    match ipc_client::add_schedule(&name, days, start_min, end_min, Some(specific_date)).await {
                        Ok(_) => refresh.input(AppMsg::RefreshSchedules),
                        Err(e) => error!("add_schedule failed: {e}"),
                    }
                });
            }
            AppMsg::UpdateSchedule { id, name, days, start_min, end_min } => {
                let refresh = _sender.clone();
                tokio::spawn(async move {
                    match ipc_client::update_schedule(id, &name, days, start_min, end_min).await {
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

            AppMsg::StatusTick => {
                let focus_sender = self.focus.sender().clone();
                let pom_sender = self.pomodoro.sender().clone();
                let lists_sender = self.allowed_lists.sender().clone();
                let settings_sender = self.settings.sender().clone();
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
                        }
                        Err(e) => warn!("status poll failed: {e}"),
                    }
                    match ipc_client::list_rule_sets().await {
                        Ok(sets) => {
                            // Flatten all URLs from all rule sets into the display list
                            let all_urls: Vec<String> = sets
                                .iter()
                                .flat_map(|s| s.allowed_urls.clone())
                                .collect();
                            lists_sender.emit(AllowedListsInput::UrlsUpdated(all_urls.clone()));
                            settings_sender.emit(SettingsInput::QuickUrlsUpdated(all_urls));
                            tick_sender.input(AppMsg::RuleSetsUpdated(
                                sets.into_iter().map(|s| s.id).collect(),
                            ));
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
