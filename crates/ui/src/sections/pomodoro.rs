use relm4::prelude::*;
use gtk4::prelude::*;

#[derive(Debug)]
pub struct PomodoroSection {
    phase: Option<String>,
    seconds_remaining: Option<u64>,
}

#[derive(Debug)]
pub enum PomodoroInput {
    StartPreset { focus_secs: u64, break_secs: u64 },
    Stop,
    StatusUpdated { phase: Option<String>, seconds_remaining: Option<u64> },
}

#[derive(Debug)]
pub enum PomodoroOutput {
    Start { focus_secs: u64, break_secs: u64 },
    Stop,
}

#[relm4::component(pub)]
impl SimpleComponent for PomodoroSection {
    type Init = ();
    type Input = PomodoroInput;
    type Output = PomodoroOutput;

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
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: PomodoroInput, sender: ComponentSender<Self>) {
        match msg {
            PomodoroInput::StartPreset { focus_secs, break_secs } => {
                let _ = sender.output(PomodoroOutput::Start { focus_secs, break_secs });
            }
            PomodoroInput::Stop => {
                let _ = sender.output(PomodoroOutput::Stop);
            }
            PomodoroInput::StatusUpdated { phase, seconds_remaining } => {
                self.phase = phase;
                self.seconds_remaining = seconds_remaining;
            }
        }
    }
}
