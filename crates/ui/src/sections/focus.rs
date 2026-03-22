use gtk4::prelude::*;
use relm4::prelude::*;

#[derive(Debug)]
pub struct FocusSection {
    focus_active: bool,
    pomodoro_running: bool,
    active_rule_set: Option<String>,
    strict_mode: bool,
    root_widget: gtk4::Box,
}

#[derive(Debug)]
pub enum FocusInput {
    SkipBreak,
    TakeBreak { break_secs: u64 },
    StatusUpdated {
        active: bool,
        rule_set: Option<String>,
    },
    PomodoroActive(bool),
    StrictModeUpdated(bool),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusOutput {
    SkipBreak,
    TakeBreak { break_secs: u64 },
}

#[relm4::component(pub)]
impl Component for FocusSection {
    type Init = ();
    type Input = FocusInput;
    type Output = FocusOutput;
    type CommandOutput = ();

    view! {
        gtk4::Box {
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 12,
            set_margin_all: 20,

            gtk4::Label {
                set_label: "Focus",
                add_css_class: "title-1",
                set_halign: gtk4::Align::Start,
            },

            gtk4::Frame {
                set_hexpand: true,

                gtk4::Box {
                    set_orientation: gtk4::Orientation::Vertical,
                    set_spacing: 12,
                    set_margin_all: 12,

                    // ── Status ───────────────────────────────────────────
                    gtk4::Box {
                        set_orientation: gtk4::Orientation::Horizontal,
                        set_spacing: 8,

                        gtk4::Label {
                            #[watch]
                            set_label: if model.focus_active || model.pomodoro_running { "● Active" } else { "○ Inactive" },
                            #[watch]
                            set_css_classes: if model.focus_active || model.pomodoro_running {
                                &["accent"]
                            } else {
                                &["dim-label"]
                            },
                            set_halign: gtk4::Align::Start,
                        },

                        gtk4::Label {
                            #[watch]
                            set_label: &match &model.active_rule_set {
                                Some(name) => format!("— {name}"),
                                None => String::new(),
                            },
                            #[watch]
                            set_visible: model.active_rule_set.is_some(),
                            add_css_class: "dim-label",
                        },
                    },

                    gtk4::Separator {
                        set_orientation: gtk4::Orientation::Horizontal,
                        set_margin_top: 2,
                        set_margin_bottom: 2,
                    },

                    // ── Quick break ──────────────────────────────────────
                    gtk4::Label {
                        set_label: "QUICK BREAK",
                        add_css_class: "dim-label",
                        set_halign: gtk4::Align::Start,
                    },

                    gtk4::Box {
                        set_orientation: gtk4::Orientation::Horizontal,
                        set_spacing: 8,

                        gtk4::Button {
                            set_label: "5m",
                            add_css_class: "suggested-action",
                            connect_clicked => FocusInput::TakeBreak { break_secs: 5 * 60 },
                        },
                        gtk4::Button {
                            set_label: "15m",
                            add_css_class: "suggested-action",
                            connect_clicked => FocusInput::TakeBreak { break_secs: 15 * 60 },
                        },
                        gtk4::Button {
                            set_label: "30m",
                            add_css_class: "suggested-action",
                            connect_clicked => FocusInput::TakeBreak { break_secs: 30 * 60 },
                        },

                        gtk4::Button {
                            set_label: "Skip Break",
                            add_css_class: "suggested-action",
                            #[watch]
                            set_visible: model.focus_active || model.pomodoro_running,
                            connect_clicked => FocusInput::SkipBreak,
                        },
                    },
                },
            },
        }
    }

    fn init(_: (), root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let model = FocusSection {
            focus_active: false,
            pomodoro_running: false,
            active_rule_set: None,
            strict_mode: false,
            root_widget: root.clone(),
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: FocusInput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            FocusInput::SkipBreak => {
                let _ = sender.output(FocusOutput::SkipBreak);
            }
            FocusInput::TakeBreak { break_secs } => {
                if self.strict_mode {
                    let root_clone = self.root_widget.clone();
                    let s = sender.clone();
                    crate::sections::strict_mode::show_strict_mode_dialog(
                        &root_clone,
                        "Strict Mode is active.\n\nTaking a quick break is restricted. To enable quick breaks, disable Strict Mode first, or confirm below.",
                        "Take Break",
                        move || { let _ = s.output(FocusOutput::TakeBreak { break_secs }); },
                    );
                } else {
                    let _ = sender.output(FocusOutput::TakeBreak { break_secs });
                }
            }
            FocusInput::StatusUpdated { active, rule_set } => {
                self.focus_active = active;
                if active {
                    // Always take the new rule set when activating
                    if rule_set.is_some() {
                        self.active_rule_set = rule_set;
                    }
                } else if !self.pomodoro_running {
                    // Only clear rule set when both focus and pomodoro are off
                    self.active_rule_set = None;
                }
            }
            FocusInput::PomodoroActive(running) => {
                self.pomodoro_running = running;
                if !running && !self.focus_active {
                    self.active_rule_set = None;
                }
            }
            FocusInput::StrictModeUpdated(enabled) => {
                self.strict_mode = enabled;
            }
        }
        self.update_view(widgets, sender);
    }
}

#[cfg(test)]
#[path = "focus_tests.rs"]
mod tests;
