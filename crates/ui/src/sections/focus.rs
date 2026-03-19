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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusOutput {
    StartFocus,
    StopFocus,
    SkipBreak,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum FocusEffect {
    Output(FocusOutput),
}

fn reduce_focus_input(model: &mut FocusSection, msg: FocusInput) -> Option<FocusEffect> {
    match msg {
        FocusInput::Toggle => {
            model.focus_active = !model.focus_active;
            if model.focus_active {
                Some(FocusEffect::Output(FocusOutput::StartFocus))
            } else {
                model.active_rule_set = None;
                Some(FocusEffect::Output(FocusOutput::StopFocus))
            }
        }
        FocusInput::SkipBreak => Some(FocusEffect::Output(FocusOutput::SkipBreak)),
        FocusInput::StatusUpdated { active, rule_set } => {
            model.focus_active = active;
            model.active_rule_set = rule_set;
            None
        }
    }
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
        if let Some(FocusEffect::Output(out)) = reduce_focus_input(self, msg) {
            let _ = sender.output(out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model() -> FocusSection {
        FocusSection {
            focus_active: false,
            active_rule_set: None,
        }
    }

    #[test]
    fn toggle_starts_and_stops_focus() {
        let mut m = model();
        assert_eq!(
            reduce_focus_input(&mut m, FocusInput::Toggle),
            Some(FocusEffect::Output(FocusOutput::StartFocus))
        );
        assert!(m.focus_active);

        m.active_rule_set = Some("Work".to_string());
        assert_eq!(
            reduce_focus_input(&mut m, FocusInput::Toggle),
            Some(FocusEffect::Output(FocusOutput::StopFocus))
        );
        assert!(!m.focus_active);
        assert!(m.active_rule_set.is_none());
    }

    #[test]
    fn skip_break_emits_output() {
        let mut m = model();
        assert_eq!(
            reduce_focus_input(&mut m, FocusInput::SkipBreak),
            Some(FocusEffect::Output(FocusOutput::SkipBreak))
        );
    }

    #[test]
    fn status_update_syncs_state() {
        let mut m = model();
        assert_eq!(
            reduce_focus_input(
                &mut m,
                FocusInput::StatusUpdated {
                    active: true,
                    rule_set: Some("Default".into()),
                }
            ),
            None
        );
        assert!(m.focus_active);
        assert_eq!(m.active_rule_set.as_deref(), Some("Default"));
    }
}
