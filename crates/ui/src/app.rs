use crate::ipc_client;
use crate::sections::{
    allowed_lists::{AllowedListsOutput, AllowedListsSection},
    focus::{FocusInput, FocusOutput, FocusSection},
    pomodoro::{PomodoroInput, PomodoroOutput, PomodoroSection},
    settings::{SettingsOutput, SettingsSection},
};
use gtk4::prelude::*;
use relm4::prelude::*;
use shared::ipc::Command;

#[derive(Debug)]
pub enum Page {
    Focus,
    AllowedLists,
    Pomodoro,
    Settings,
}

pub struct App {
    current_page: Page,
    focus: Controller<FocusSection>,
    pomodoro: Controller<PomodoroSection>,
    allowed_lists: Controller<AllowedListsSection>,
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
    StrictModeChanged(bool),
    // Periodic status poll result
    StatusTick,
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
            });

        let settings = SettingsSection::builder()
            .launch(false)
            .forward(sender.input_sender(), |out| match out {
                SettingsOutput::StrictModeChanged(v) => AppMsg::StrictModeChanged(v),
                SettingsOutput::CalDavSaved { .. } => AppMsg::StrictModeChanged(false), // placeholder
            });

        let model = App {
            current_page: Page::Focus,
            focus,
            pomodoro,
            allowed_lists,
            settings,
        };

        let widgets = view_output!();

        // Add child pages to the Stack now that we have both widgets and model
        widgets.stack.add_named(model.focus.widget(), Some("focus"));
        widgets.stack.add_named(model.allowed_lists.widget(), Some("allowed_lists"));
        widgets.stack.add_named(model.pomodoro.widget(), Some("pomodoro"));
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
                    Page::Settings => "settings",
                };
                widgets.stack.set_visible_child_name(name);
                self.current_page = page;
            }

            AppMsg::StartFocus => {
                // TODO: pick rule_set_id from selection; use first available for now
                tokio::spawn(async {
                    let _ = ipc_client::send(&Command::StartFocus {
                        rule_set_id: uuid::Uuid::nil(),
                    })
                    .await;
                });
            }
            AppMsg::StopFocus => {
                tokio::spawn(async {
                    let _ = ipc_client::send(&Command::StopFocus).await;
                });
            }
            AppMsg::SkipBreak => {
                tokio::spawn(async {
                    let _ = ipc_client::send(&Command::SkipBreak).await;
                });
            }
            AppMsg::StartPomodoro { focus_secs, break_secs } => {
                tokio::spawn(async move {
                    let _ = ipc_client::send(&Command::StartPomodoro { focus_secs, break_secs })
                        .await;
                });
            }
            AppMsg::StopPomodoro => {
                tokio::spawn(async {
                    let _ = ipc_client::send(&Command::StopPomodoro).await;
                });
            }
            AppMsg::AddUrl(url) => {
                tokio::spawn(async move {
                    let _ = ipc_client::send(&Command::AddRuleSet {
                        name: "Imported".into(),
                        allowed_urls: vec![url],
                    })
                    .await;
                });
            }
            AppMsg::StrictModeChanged(_) => {}

            AppMsg::StatusTick => {
                let focus_sender = self.focus.sender().clone();
                let pom_sender = self.pomodoro.sender().clone();
                tokio::spawn(async move {
                    if let Ok(status) = ipc_client::get_status().await {
                        focus_sender.emit(FocusInput::StatusUpdated {
                            active: status.focus_active,
                            rule_set: status.active_rule_set_name,
                        });
                        pom_sender.emit(PomodoroInput::StatusUpdated {
                            phase: status.pomodoro_phase.map(|p| format!("{p:?}")),
                            seconds_remaining: status.seconds_remaining,
                        });
                    }
                });
            }
        }
    }
}
