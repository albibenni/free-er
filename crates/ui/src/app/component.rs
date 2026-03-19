use gtk4::prelude::*;
use relm4::prelude::*;
use relm4::ComponentController;
use super::{focus_handlers, forwarders, schedule_handlers, settings_handlers, status_handlers, url_handlers};
use super::types::{App, AppMsg, Page};

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

                    gtk4::Separator {
                        set_orientation: gtk4::Orientation::Horizontal,
                        set_margin_top: 6,
                        set_margin_bottom: 6,
                    },

                    gtk4::Button {
                        add_css_class: "flat",
                        connect_clicked => AppMsg::Navigate(Page::Calendar),
                        gtk4::Box {
                            set_orientation: gtk4::Orientation::Horizontal,
                            set_spacing: 8,
                            gtk4::Image { set_icon_name: Some("emblem-system-symbolic") },
                            #[name = "lbl_calendar"]
                            gtk4::Label { set_label: "Calendar Settings" },
                        },
                    },

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
        let focus = forwarders::launch_focus(&sender);
        let pomodoro = forwarders::launch_pomodoro(&sender);
        let allowed_lists = forwarders::launch_allowed_lists(&sender);
        let schedule = forwarders::launch_schedule(&sender);
        let calendar_rules = forwarders::launch_calendar_rules(&sender);
        let settings = forwarders::launch_settings(&sender);

        let model = App {
            current_page: Page::Focus,
            sidebar_open: true,
            default_rule_set_id: None,
            focus,
            pomodoro,
            allowed_lists,
            schedule,
            calendar_rules,
            settings,
        };

        let widgets = view_output!();

        widgets.stack.add_named(model.focus.widget(), Some("focus"));
        widgets
            .stack
            .add_named(model.allowed_lists.widget(), Some("allowed_lists"));
        widgets
            .stack
            .add_named(model.pomodoro.widget(), Some("pomodoro"));
        widgets
            .stack
            .add_named(model.schedule.widget(), Some("schedule"));
        widgets
            .stack
            .add_named(model.calendar_rules.widget(), Some("calendar"));
        widgets
            .stack
            .add_named(model.settings.widget(), Some("settings"));

        // Poll daemon status every 2 seconds
        let tick_sender = sender.clone();
        gtk4::glib::timeout_add_seconds_local(2, move || {
            tick_sender.input(AppMsg::StatusTick);
            gtk4::glib::ControlFlow::Continue
        });

        // Set initial sidebar minimum width
        widgets.sidebar.set_width_request(160);

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
                // set_width_request controls the minimum width; when labels are
                // hidden the box naturally shrinks to icon width (~40px).
                widgets
                    .sidebar
                    .set_width_request(if open { 160 } else { 48 });
                widgets.lbl_focus.set_visible(open);
                widgets.lbl_allowed.set_visible(open);
                widgets.lbl_pomodoro.set_visible(open);
                widgets.lbl_schedule.set_visible(open);
                widgets.lbl_calendar.set_visible(open);
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
                    Page::Calendar => "calendar",
                    Page::Settings => "settings",
                };
                widgets.stack.set_visible_child_name(name);
                self.current_page = page;
            }

            // ── Focus / Pomodoro ─────────────────────────────────────────
            AppMsg::StartFocus => focus_handlers::start_focus(self.default_rule_set_id),
            AppMsg::StopFocus => focus_handlers::stop_focus(),
            AppMsg::SkipBreak => focus_handlers::skip_break(),
            AppMsg::StartPomodoro {
                focus_secs,
                break_secs,
                rule_set_id,
            } => {
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
            AppMsg::ChooseDefaultRuleSet(id) => {
                let s = sender.clone();
                relm4::spawn(async move {
                    if crate::ipc_client::set_default_rule_set(id).await.is_ok() {
                        s.input(AppMsg::SetDefaultRuleSet(id));
                    }
                });
            }
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
            AppMsg::ResyncCalendar => schedule_handlers::resync_calendar(sender),

            // ── Calendar import rules ────────────────────────────────────
            AppMsg::AddImportRule { keyword, schedule_type } => {
                relm4::spawn(async move {
                    let _ = crate::ipc_client::add_import_rule(&keyword, schedule_type).await;
                });
            }
            AppMsg::RemoveImportRule { keyword, schedule_type } => {
                relm4::spawn(async move {
                    let _ = crate::ipc_client::remove_import_rule(&keyword, schedule_type).await;
                });
            }
            // ── Status / refresh ─────────────────────────────────────────
            AppMsg::StatusTick => status_handlers::status_tick(self, self.default_rule_set_id, sender),
            AppMsg::RefreshRuleSets => {
                status_handlers::refresh_rule_sets(self, self.default_rule_set_id, sender)
            }
            AppMsg::SetDefaultRuleSet(id) => {
                self.default_rule_set_id = Some(id);
                self.allowed_lists
                    .sender()
                    .emit(crate::sections::allowed_lists::AllowedListsInput::DefaultRuleSetUpdated(
                        Some(id),
                    ));
                self.schedule
                    .sender()
                    .emit(crate::sections::schedule::ScheduleInput::DefaultRuleSetUpdated(
                        Some(id),
                    ));
            }
        }
    }
}
