use relm4::prelude::*;
use gtk4::prelude::*;
use shared::ipc::RuleSetSummary;
use uuid::Uuid;

#[derive(Debug)]
pub struct PomodoroSection {
    phase: Option<String>,
    seconds_remaining: Option<u64>,
    rule_sets: Vec<RuleSetSummary>,
    selected_rule_set_id: Option<Uuid>,
}

#[derive(Debug)]
pub enum PomodoroInput {
    StartPreset { focus_secs: u64, break_secs: u64 },
    Stop,
    StatusUpdated { phase: Option<String>, seconds_remaining: Option<u64> },
    RuleSetsUpdated(Vec<RuleSetSummary>),
}

#[derive(Debug)]
pub enum PomodoroOutput {
    Start { focus_secs: u64, break_secs: u64, rule_set_id: Option<Uuid> },
    Stop,
}

#[relm4::component(pub)]
impl Component for PomodoroSection {
    type Init = ();
    type Input = PomodoroInput;
    type Output = PomodoroOutput;
    type CommandOutput = ();

    view! {
        gtk4::Box {
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 16,
            set_margin_all: 24,

            gtk4::Label {
                set_label: "Pomodoro",
                add_css_class: "title-1",
                set_halign: gtk4::Align::Start,
            },

            gtk4::Label {
                #[watch]
                set_label: &match (model.phase.as_deref(), model.seconds_remaining) {
                    (Some(phase), Some(secs)) => {
                        let m = secs / 60;
                        let s = secs % 60;
                        format!("{phase} — {m:02}:{s:02}")
                    }
                    _ => "Idle".into(),
                },
                add_css_class: "title-2",
            },

            // ── Rule set selector ─────────────────────────────────────────
            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 8,
                set_valign: gtk4::Align::Center,

                gtk4::Label {
                    set_label: "Allowed list:",
                    add_css_class: "dim-label",
                },

                #[name = "rule_set_combo"]
                gtk4::ComboBoxText {
                    set_hexpand: true,
                },
            },

            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 8,
                set_homogeneous: true,

                gtk4::Button {
                    set_label: "25 / 5",
                    connect_clicked => PomodoroInput::StartPreset { focus_secs: 25 * 60, break_secs: 5 * 60 },
                },
                gtk4::Button {
                    set_label: "50 / 10",
                    connect_clicked => PomodoroInput::StartPreset { focus_secs: 50 * 60, break_secs: 10 * 60 },
                },
                gtk4::Button {
                    set_label: "90 / 20",
                    connect_clicked => PomodoroInput::StartPreset { focus_secs: 90 * 60, break_secs: 20 * 60 },
                },
            },

            gtk4::Button {
                set_label: "Stop",
                set_css_classes: &["destructive-action"],
                #[watch]
                set_sensitive: model.phase.is_some(),
                connect_clicked => PomodoroInput::Stop,
            },
        }
    }

    fn init(_: (), root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let model = PomodoroSection {
            phase: None,
            seconds_remaining: None,
            rule_sets: vec![],
            selected_rule_set_id: None,
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: PomodoroInput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            PomodoroInput::StartPreset { focus_secs, break_secs } => {
                // Read selected rule set from the combo box
                let rule_set_id = widgets.rule_set_combo
                    .active_id()
                    .and_then(|id| id.parse::<Uuid>().ok());
                let _ = sender.output(PomodoroOutput::Start { focus_secs, break_secs, rule_set_id });
            }
            PomodoroInput::Stop => {
                let _ = sender.output(PomodoroOutput::Stop);
            }
            PomodoroInput::StatusUpdated { phase, seconds_remaining } => {
                self.phase = phase;
                self.seconds_remaining = seconds_remaining;
            }
            PomodoroInput::RuleSetsUpdated(sets) => {
                let prev_id = widgets.rule_set_combo
                    .active_id()
                    .and_then(|id| id.parse::<Uuid>().ok());

                widgets.rule_set_combo.remove_all();
                for (i, rs) in sets.iter().enumerate() {
                    let label = if i == 0 {
                        format!("{} (default)", rs.name)
                    } else {
                        rs.name.clone()
                    };
                    widgets.rule_set_combo.append(Some(&rs.id.to_string()), &label);
                }

                // Restore selection or default to first
                let restore_id = prev_id
                    .filter(|id| sets.iter().any(|s| s.id == *id))
                    .or_else(|| sets.first().map(|s| s.id));
                if let Some(id) = restore_id {
                    widgets.rule_set_combo.set_active_id(Some(&id.to_string()));
                    self.selected_rule_set_id = Some(id);
                }

                self.rule_sets = sets;
            }
        }
        self.update_view(widgets, sender);
    }
}
