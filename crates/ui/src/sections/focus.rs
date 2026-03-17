use relm4::prelude::*;
use gtk4::prelude::*;

#[derive(Debug)]
pub struct FocusSection {
    focus_active: bool,
    active_rule_set: Option<String>,
}

#[derive(Debug)]
pub enum FocusInput {
    Toggle,
    SkipBreak,
    StatusUpdated { active: bool, rule_set: Option<String> },
}

#[derive(Debug)]
pub enum FocusOutput {
    StartFocus,
    StopFocus,
    SkipBreak,
}

#[relm4::component(pub)]
impl SimpleComponent for FocusSection {
    type Init = ();
    type Input = FocusInput;
    type Output = FocusOutput;

    view! {
        gtk4::Box {
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 16,
            set_margin_all: 24,

            gtk4::Label {
                set_label: "Focus",
                add_css_class: "title-1",
                set_halign: gtk4::Align::Start,
            },

            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 12,

                #[name = "toggle_btn"]
                gtk4::Button {
                    #[watch]
                    set_label: if model.focus_active { "Stop Focus" } else { "Start Focus" },
                    #[watch]
                    set_css_classes: if model.focus_active {
                        &["destructive-action"]
                    } else {
                        &["suggested-action"]
                    },
                    connect_clicked => FocusInput::Toggle,
                },

                gtk4::Button {
                    set_label: "Skip Break",
                    #[watch]
                    set_visible: model.focus_active,
                    connect_clicked => FocusInput::SkipBreak,
                },
            },

            gtk4::Label {
                #[watch]
                set_label: &match &model.active_rule_set {
                    Some(name) => format!("Active list: {name}"),
                    None => "No list active".into(),
                },
                set_halign: gtk4::Align::Start,
                add_css_class: "dim-label",
            },
        }
    }

    fn init(_: (), root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let model = FocusSection {
            focus_active: false,
            active_rule_set: None,
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: FocusInput, sender: ComponentSender<Self>) {
        match msg {
            FocusInput::Toggle => {
                if self.focus_active {
                    let _ = sender.output(FocusOutput::StopFocus);
                } else {
                    let _ = sender.output(FocusOutput::StartFocus);
                }
            }
            FocusInput::SkipBreak => {
                let _ = sender.output(FocusOutput::SkipBreak);
            }
            FocusInput::StatusUpdated { active, rule_set } => {
                self.focus_active = active;
                self.active_rule_set = rule_set;
            }
        }
    }
}
