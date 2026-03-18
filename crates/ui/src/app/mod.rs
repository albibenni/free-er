mod focus_handlers;
mod schedule_handlers;
mod settings_handlers;
mod status_handlers;
mod url_handlers;

use crate::sections::{
    allowed_lists::{AllowedListsOutput, AllowedListsSection},
    focus::{FocusOutput, FocusSection},
    pomodoro::{PomodoroOutput, PomodoroSection},
    schedule::{ScheduleOutput, ScheduleSection},
    settings::{SettingsOutput, SettingsSection},
};
use gtk4::prelude::*;
use relm4::prelude::*;
use relm4::ComponentController;
use shared::ipc::{ScheduleSummary, ScheduleType};
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
    sidebar_open: bool,
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
    ToggleSidebar,
    // Focus / Pomodoro session control
    StartFocus,
    StopFocus,
    SkipBreak,
    StartPomodoro { focus_secs: u64, break_secs: u64, rule_set_id: Option<Uuid> },
    StopPomodoro,
    // URL / rule-set management
    AddUrl(String),
    RemoveUrl(String),
    AddUrlToList { rule_set_id: Uuid, url: String },
    RemoveUrlFromList { rule_set_id: Uuid, url: String },
    CreateRuleSet(String),
    DeleteRuleSet(Uuid),
    AiSitesToggled(bool),
    SearchEnginesToggled(bool),
    // Settings / integrations
    ConnectGoogle,
    DisconnectGoogle,
    StrictModeChanged(bool),
    AllowNewTabChanged(bool),
    SaveCalDav { url: String, user: String, pass: String },
    // Schedule CRUD
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
    // Status / refresh
    StatusTick,
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
                #[name = "sidebar"]
                gtk4::Box {
                    set_orientation: gtk4::Orientation::Vertical,
                    add_css_class: "sidebar",
                    set_spacing: 4,
                    set_margin_all: 8,

                    // Toggle button
                    #[name = "btn_toggle"]
                    gtk4::Button {
                        add_css_class: "flat",
                        set_halign: gtk4::Align::End,
                        set_icon_name: "pan-start-symbolic",
                        connect_clicked => AppMsg::ToggleSidebar,
                    },

                    gtk4::Button {
                        add_css_class: "flat",
                        connect_clicked => AppMsg::Navigate(Page::Focus),
                        gtk4::Box {
                            set_orientation: gtk4::Orientation::Horizontal,
                            set_spacing: 8,
                            gtk4::Image { set_icon_name: Some("media-playback-start-symbolic") },
                            #[name = "lbl_focus"]
                            gtk4::Label { set_label: "Focus" },
                        },
                    },
                    gtk4::Button {
                        add_css_class: "flat",
                        connect_clicked => AppMsg::Navigate(Page::AllowedLists),
                        gtk4::Box {
                            set_orientation: gtk4::Orientation::Horizontal,
                            set_spacing: 8,
                            gtk4::Image { set_icon_name: Some("security-high-symbolic") },
                            #[name = "lbl_allowed"]
                            gtk4::Label { set_label: "Allowed Lists" },
                        },
                    },
                    gtk4::Button {
                        add_css_class: "flat",
                        connect_clicked => AppMsg::Navigate(Page::Pomodoro),
                        gtk4::Box {
                            set_orientation: gtk4::Orientation::Horizontal,
                            set_spacing: 8,
                            gtk4::Image { set_icon_name: Some("alarm-symbolic") },
                            #[name = "lbl_pomodoro"]
                            gtk4::Label { set_label: "Pomodoro" },
                        },
                    },
                    gtk4::Button {
                        add_css_class: "flat",
                        connect_clicked => AppMsg::Navigate(Page::Schedule),
                        gtk4::Box {
                            set_orientation: gtk4::Orientation::Horizontal,
                            set_spacing: 8,
                            gtk4::Image { set_icon_name: Some("x-office-calendar-symbolic") },
                            #[name = "lbl_schedule"]
                            gtk4::Label { set_label: "Schedule" },
                        },
                    },

                    // Spacer — pushes Settings to the bottom
                    gtk4::Box { set_vexpand: true },

                    gtk4::Button {
                        add_css_class: "flat",
                        connect_clicked => AppMsg::Navigate(Page::Settings),
                        gtk4::Box {
                            set_orientation: gtk4::Orientation::Horizontal,
                            set_spacing: 8,
                            gtk4::Image { set_icon_name: Some("preferences-system-symbolic") },
                            #[name = "lbl_settings"]
                            gtk4::Label { set_label: "Settings" },
                        },
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
                ScheduleOutput::CreateSchedule {
                    name,
                    days,
                    start_min,
                    end_min,
                    specific_date,
                    rule_set_id,
                    schedule_type,
                } => AppMsg::CreateSchedule {
                    name,
                    days,
                    start_min,
                    end_min,
                    specific_date,
                    rule_set_id,
                    schedule_type,
                },
                ScheduleOutput::UpdateSchedule {
                    id,
                    name,
                    days,
                    start_min,
                    end_min,
                    rule_set_id,
                    specific_date,
                    schedule_type,
                } => AppMsg::UpdateSchedule {
                    id,
                    name,
                    days,
                    start_min,
                    end_min,
                    rule_set_id,
                    specific_date,
                    schedule_type,
                },
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
                    if enabled {
                        AppMsg::AddUrl(url.to_string())
                    } else {
                        AppMsg::RemoveUrl(url.to_string())
                    }
                }
                SettingsOutput::CalDavSaved { url, user, pass } => {
                    AppMsg::SaveCalDav { url, user, pass }
                }
                SettingsOutput::ConnectGoogleRequested => AppMsg::ConnectGoogle,
                SettingsOutput::DisconnectGoogleRequested => AppMsg::DisconnectGoogle,
            });

        let model = App {
            current_page: Page::Focus,
            sidebar_open: true,
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

        // CSS: enforce max-width on the collapsed sidebar so it actually shrinks
        let css = gtk4::CssProvider::new();
        css.load_from_data(
            ".sidebar { min-width: 160px; } \
             .sidebar.collapsed { min-width: 48px; max-width: 48px; }",
        );
        if let Some(display) = gtk4::gdk::Display::default() {
            gtk4::style_context_add_provider_for_display(
                &display,
                &css,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: AppMsg,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            // ── UI / Navigation ──────────────────────────────────────────
            AppMsg::ToggleSidebar => {
                self.sidebar_open = !self.sidebar_open;
                let open = self.sidebar_open;
                if open {
                    widgets.sidebar.remove_css_class("collapsed");
                } else {
                    widgets.sidebar.add_css_class("collapsed");
                }
                widgets.lbl_focus.set_visible(open);
                widgets.lbl_allowed.set_visible(open);
                widgets.lbl_pomodoro.set_visible(open);
                widgets.lbl_schedule.set_visible(open);
                widgets.lbl_settings.set_visible(open);
                widgets.btn_toggle.set_icon_name(if open {
                    "pan-start-symbolic"
                } else {
                    "pan-end-symbolic"
                });
            }
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

            // ── Focus / Pomodoro ─────────────────────────────────────────
            AppMsg::StartFocus => focus_handlers::start_focus(self.default_rule_set_id),
            AppMsg::StopFocus => focus_handlers::stop_focus(),
            AppMsg::SkipBreak => focus_handlers::skip_break(),
            AppMsg::StartPomodoro { focus_secs, break_secs, rule_set_id } => {
                focus_handlers::start_pomodoro(focus_secs, break_secs, rule_set_id);
            }
            AppMsg::StopPomodoro => focus_handlers::stop_pomodoro(),

            // ── URL / rule-set management ────────────────────────────────
            AppMsg::AddUrl(url) => {
                url_handlers::add_url(url, self.default_rule_set_id, sender);
            }
            AppMsg::RemoveUrl(url) => {
                url_handlers::remove_url(url, self.default_rule_set_id);
            }
            AppMsg::AddUrlToList { rule_set_id, url } => {
                url_handlers::add_url_to_list(rule_set_id, url);
            }
            AppMsg::RemoveUrlFromList { rule_set_id, url } => {
                url_handlers::remove_url_from_list(rule_set_id, url);
            }
            AppMsg::CreateRuleSet(name) => url_handlers::create_rule_set(name, sender),
            AppMsg::DeleteRuleSet(id) => url_handlers::delete_rule_set(id, sender),
            AppMsg::AiSitesToggled(enabled) => {
                url_handlers::toggle_ai_sites(enabled, self.default_rule_set_id, sender);
            }
            AppMsg::SearchEnginesToggled(enabled) => {
                url_handlers::toggle_search_engines(enabled, self.default_rule_set_id, sender);
            }

            // ── Settings / integrations ──────────────────────────────────
            AppMsg::ConnectGoogle => settings_handlers::connect_google(),
            AppMsg::DisconnectGoogle => settings_handlers::disconnect_google(),
            AppMsg::StrictModeChanged(enabled) => settings_handlers::set_strict_mode(enabled),
            AppMsg::AllowNewTabChanged(enabled) => settings_handlers::set_allow_new_tab(enabled),
            AppMsg::SaveCalDav { url, user, pass } => {
                settings_handlers::save_caldav(url, user, pass);
            }

            // ── Schedule CRUD ────────────────────────────────────────────
            AppMsg::SchedulesUpdated(schedules) => {
                schedule_handlers::schedules_updated(self.schedule.sender(), schedules);
            }
            AppMsg::CreateSchedule {
                name,
                days,
                start_min,
                end_min,
                specific_date,
                rule_set_id,
                schedule_type,
            } => {
                schedule_handlers::create_schedule(
                    name,
                    days,
                    start_min,
                    end_min,
                    specific_date,
                    rule_set_id,
                    schedule_type,
                    sender,
                );
            }
            AppMsg::UpdateSchedule {
                id,
                name,
                days,
                start_min,
                end_min,
                rule_set_id,
                specific_date,
                schedule_type,
            } => {
                schedule_handlers::update_schedule(
                    id,
                    name,
                    days,
                    start_min,
                    end_min,
                    rule_set_id,
                    specific_date,
                    schedule_type,
                    sender,
                );
            }
            AppMsg::DeleteSchedule(id) => schedule_handlers::delete_schedule(id, sender),
            AppMsg::RefreshSchedules => schedule_handlers::refresh_schedules(sender),

            // ── Status / refresh ─────────────────────────────────────────
            AppMsg::StatusTick => status_handlers::status_tick(self, sender),
            AppMsg::RefreshRuleSets => status_handlers::refresh_rule_sets(self, sender),
            AppMsg::SetDefaultRuleSet(id) => {
                self.default_rule_set_id = Some(id);
            }
        }
    }
}
