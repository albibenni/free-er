use gtk4::prelude::*;
use relm4::prelude::*;
use shared::ipc::RuleSetSummary;
use std::cell::RefCell;
use std::f64::consts::{FRAC_PI_2, PI};
use std::rc::Rc;
use uuid::Uuid;

#[derive(Debug, Default)]
struct RingVisualState {
    focus_secs: u64,
    break_secs: u64,
    phase: Option<String>,
    seconds_remaining: Option<u64>,
}

#[derive(Debug)]
pub struct PomodoroSection {
    phase: Option<String>,
    seconds_remaining: Option<u64>,
    rule_sets: Vec<RuleSetSummary>,
    selected_rule_set_id: Option<Uuid>,
    focus_secs: u64,
    break_secs: u64,
    ring_visual: Rc<RefCell<RingVisualState>>,
}

#[derive(Debug)]
pub enum PomodoroInput {
    SelectPreset { focus_secs: u64, break_secs: u64 },
    SetQuickBreak { break_secs: u64 },
    AdjustFocus(i64),
    AdjustBreak(i64),
    Start,
    Stop,
    RuleSetChanged,
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
            set_spacing: 12,
            set_margin_all: 20,

            gtk4::Label {
                set_label: "Pomodoro Mode",
                add_css_class: "title-1",
                set_halign: gtk4::Align::Start,
            },

            gtk4::Frame {
                set_hexpand: true,
                set_margin_bottom: 6,

                gtk4::Box {
                    set_orientation: gtk4::Orientation::Vertical,
                    set_spacing: 14,
                    set_margin_all: 12,

                    gtk4::Box {
                        set_orientation: gtk4::Orientation::Horizontal,
                        set_spacing: 24,

                        // Left rail: presets + quick break
                        gtk4::Box {
                            set_orientation: gtk4::Orientation::Vertical,
                            set_spacing: 8,
                            set_width_request: 130,

                            gtk4::Label {
                                set_label: "PRESETS",
                                add_css_class: "dim-label",
                                set_halign: gtk4::Align::Start,
                            },
                            gtk4::Button {
                                set_label: "25 / 5",
                                connect_clicked => PomodoroInput::SelectPreset { focus_secs: 25 * 60, break_secs: 5 * 60 },
                            },
                            gtk4::Button {
                                set_label: "45 / 15",
                                add_css_class: "suggested-action",
                                connect_clicked => PomodoroInput::SelectPreset { focus_secs: 45 * 60, break_secs: 15 * 60 },
                            },
                            gtk4::Button {
                                set_label: "50 / 10",
                                connect_clicked => PomodoroInput::SelectPreset { focus_secs: 50 * 60, break_secs: 10 * 60 },
                            },
                            gtk4::Button {
                                set_label: "90 / 20",
                                connect_clicked => PomodoroInput::SelectPreset { focus_secs: 90 * 60, break_secs: 20 * 60 },
                            },

                            gtk4::Separator {
                                set_orientation: gtk4::Orientation::Horizontal,
                                set_margin_top: 6,
                                set_margin_bottom: 4,
                            },

                            gtk4::Label {
                                set_label: "QUICK BREAK",
                                add_css_class: "dim-label",
                                set_halign: gtk4::Align::Start,
                            },
                            gtk4::Button {
                                set_label: "5m",
                                connect_clicked => PomodoroInput::SetQuickBreak { break_secs: 5 * 60 },
                            },
                            gtk4::Button {
                                set_label: "15m",
                                connect_clicked => PomodoroInput::SetQuickBreak { break_secs: 15 * 60 },
                            },
                            gtk4::Button {
                                set_label: "30m",
                                connect_clicked => PomodoroInput::SetQuickBreak { break_secs: 30 * 60 },
                            },
                        },

                        // Center: focus / break controls
                        gtk4::Box {
                            set_orientation: gtk4::Orientation::Horizontal,
                            set_spacing: 28,
                            set_halign: gtk4::Align::Center,
                            set_hexpand: true,

                            gtk4::Frame {
                                set_width_request: 220,
                                gtk4::Box {
                                    set_orientation: gtk4::Orientation::Vertical,
                                    set_spacing: 8,
                                    set_margin_all: 12,

                                    gtk4::Label {
                                        set_label: "FOCUS",
                                        add_css_class: "dim-label",
                                        set_halign: gtk4::Align::Center,
                                    },
                                    gtk4::Overlay {
                                        set_halign: gtk4::Align::Center,
                                        set_valign: gtk4::Align::Center,

                                        #[name = "focus_ring"]
                                        gtk4::DrawingArea {
                                            set_content_width: 180,
                                            set_content_height: 180,
                                        },

                                        add_overlay = &gtk4::Box {
                                            set_orientation: gtk4::Orientation::Vertical,
                                            set_halign: gtk4::Align::Center,
                                            set_valign: gtk4::Align::Center,
                                            set_spacing: 4,

                                            gtk4::Image {
                                                set_icon_name: Some("weather-clear-symbolic"),
                                                set_pixel_size: 28,
                                                set_halign: gtk4::Align::Center,
                                            },
                                            gtk4::Label {
                                                #[watch]
                                                set_label: &format!("{}m", model.focus_secs / 60),
                                                add_css_class: "title-1",
                                                set_halign: gtk4::Align::Center,
                                            },
                                        },
                                    },
                                    gtk4::Box {
                                        set_orientation: gtk4::Orientation::Horizontal,
                                        set_halign: gtk4::Align::Center,
                                        set_spacing: 6,
                                        gtk4::Button {
                                            set_label: "−",
                                            connect_clicked => PomodoroInput::AdjustFocus(-5),
                                        },
                                        gtk4::Button {
                                            set_label: "+",
                                            connect_clicked => PomodoroInput::AdjustFocus(5),
                                        },
                                    },
                                },
                            },

                            gtk4::Frame {
                                set_width_request: 220,
                                gtk4::Box {
                                    set_orientation: gtk4::Orientation::Vertical,
                                    set_spacing: 8,
                                    set_margin_all: 12,

                                    gtk4::Label {
                                        set_label: "BREAK",
                                        add_css_class: "dim-label",
                                        set_halign: gtk4::Align::Center,
                                    },
                                    gtk4::Overlay {
                                        set_halign: gtk4::Align::Center,
                                        set_valign: gtk4::Align::Center,

                                        #[name = "break_ring"]
                                        gtk4::DrawingArea {
                                            set_content_width: 180,
                                            set_content_height: 180,
                                        },

                                        add_overlay = &gtk4::Box {
                                            set_orientation: gtk4::Orientation::Vertical,
                                            set_halign: gtk4::Align::Center,
                                            set_valign: gtk4::Align::Center,
                                            set_spacing: 4,

                                            gtk4::Image {
                                                set_icon_name: Some("emblem-favorite-symbolic"),
                                                set_pixel_size: 28,
                                                set_halign: gtk4::Align::Center,
                                            },
                                            gtk4::Label {
                                                #[watch]
                                                set_label: &format!("{}m", model.break_secs / 60),
                                                add_css_class: "title-1",
                                                set_halign: gtk4::Align::Center,
                                            },
                                        },
                                    },
                                    gtk4::Box {
                                        set_orientation: gtk4::Orientation::Horizontal,
                                        set_halign: gtk4::Align::Center,
                                        set_spacing: 6,
                                        gtk4::Button {
                                            set_label: "−",
                                            connect_clicked => PomodoroInput::AdjustBreak(-5),
                                        },
                                        gtk4::Button {
                                            set_label: "+",
                                            connect_clicked => PomodoroInput::AdjustBreak(5),
                                        },
                                    },
                                },
                            },
                        },
                    },

                    gtk4::Label {
                        #[watch]
                        set_label: &match (model.phase.as_deref(), model.seconds_remaining) {
                            (Some(phase), Some(secs)) => {
                                let m = secs / 60;
                                let s = secs % 60;
                                format!("{phase} - {m:02}:{s:02}")
                            }
                            _ => "Inactive".into(),
                        },
                        add_css_class: "dim-label",
                        set_halign: gtk4::Align::Start,
                    },

                    gtk4::Label {
                        set_label: "SELECT LIST",
                        add_css_class: "dim-label",
                        set_halign: gtk4::Align::Start,
                    },

                    #[name = "rule_set_combo"]
                    gtk4::ComboBoxText {
                        set_hexpand: true,
                        connect_changed => PomodoroInput::RuleSetChanged,
                    },

                    gtk4::Box {
                        set_orientation: gtk4::Orientation::Horizontal,
                        set_spacing: 8,
                        gtk4::Button {
                            set_label: "Start Focus Session",
                            add_css_class: "suggested-action",
                            set_hexpand: true,
                            connect_clicked => PomodoroInput::Start,
                        },
                        gtk4::Button {
                            set_label: "Stop",
                            add_css_class: "destructive-action",
                            #[watch]
                            set_sensitive: model.phase.is_some(),
                            connect_clicked => PomodoroInput::Stop,
                        },
                    },
                },
            },
        }
    }

    fn init(_: (), _root: Self::Root, _sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let model = PomodoroSection {
            phase: None,
            seconds_remaining: None,
            rule_sets: vec![],
            selected_rule_set_id: None,
            focus_secs: 45 * 60,
            break_secs: 15 * 60,
            ring_visual: Rc::new(RefCell::new(RingVisualState {
                focus_secs: 45 * 60,
                break_secs: 15 * 60,
                phase: None,
                seconds_remaining: None,
            })),
        };
        let widgets = view_output!();
        {
            let ring = model.ring_visual.clone();
            widgets.focus_ring.set_draw_func(move |_, cr, w, h| {
                let s = ring.borrow();
                draw_ring(
                    cr,
                    w as f64,
                    h as f64,
                    focus_fraction(&s),
                    (0.12, 0.55, 0.95),
                );
            });
        }
        {
            let ring = model.ring_visual.clone();
            widgets.break_ring.set_draw_func(move |_, cr, w, h| {
                let s = ring.borrow();
                draw_ring(
                    cr,
                    w as f64,
                    h as f64,
                    break_fraction(&s),
                    (0.98, 0.60, 0.18),
                );
            });
        }
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
            PomodoroInput::SelectPreset {
                focus_secs,
                break_secs,
            } => {
                self.focus_secs = focus_secs;
                self.break_secs = break_secs;
                let mut s = self.ring_visual.borrow_mut();
                s.focus_secs = self.focus_secs;
                s.break_secs = self.break_secs;
            }
            PomodoroInput::SetQuickBreak { break_secs } => {
                self.break_secs = break_secs;
                self.ring_visual.borrow_mut().break_secs = self.break_secs;
            }
            PomodoroInput::AdjustFocus(delta_min) => {
                let mins = (self.focus_secs / 60) as i64;
                let new_mins = (mins + delta_min).clamp(5, 180) as u64;
                self.focus_secs = new_mins * 60;
                self.ring_visual.borrow_mut().focus_secs = self.focus_secs;
            }
            PomodoroInput::AdjustBreak(delta_min) => {
                let mins = (self.break_secs / 60) as i64;
                let new_mins = (mins + delta_min).clamp(1, 90) as u64;
                self.break_secs = new_mins * 60;
                self.ring_visual.borrow_mut().break_secs = self.break_secs;
            }
            PomodoroInput::Start => {
                let _ = sender.output(PomodoroOutput::Start {
                    focus_secs: self.focus_secs,
                    break_secs: self.break_secs,
                    rule_set_id: self.selected_rule_set_id,
                });
            }
            PomodoroInput::Stop => {
                let _ = sender.output(PomodoroOutput::Stop);
            }
            PomodoroInput::RuleSetChanged => {
                self.selected_rule_set_id = widgets
                    .rule_set_combo
                    .active_id()
                    .and_then(|id| id.parse::<Uuid>().ok());
            }
            PomodoroInput::StatusUpdated {
                phase,
                seconds_remaining,
            } => {
                self.phase = phase.clone();
                self.seconds_remaining = seconds_remaining;
                let mut s = self.ring_visual.borrow_mut();
                s.phase = phase;
                s.seconds_remaining = self.seconds_remaining;
            }
            PomodoroInput::RuleSetsUpdated(sets) => {
                let prev_id = self.selected_rule_set_id;

                widgets.rule_set_combo.remove_all();
                for (i, rs) in sets.iter().enumerate() {
                    let label = if i == 0 {
                        format!("{} (default)", rs.name)
                    } else {
                        rs.name.clone()
                    };
                    widgets
                        .rule_set_combo
                        .append(Some(&rs.id.to_string()), &label);
                }

                let restore_id = prev_id
                    .filter(|id| sets.iter().any(|s| s.id == *id))
                    .or_else(|| sets.first().map(|s| s.id));
                if let Some(id) = restore_id {
                    widgets.rule_set_combo.set_active_id(Some(&id.to_string()));
                    self.selected_rule_set_id = Some(id);
                } else {
                    self.selected_rule_set_id = None;
                }
                self.rule_sets = sets;
            }
        }
        widgets.focus_ring.queue_draw();
        widgets.break_ring.queue_draw();
        self.update_view(widgets, sender);
    }
}

fn focus_fraction(state: &RingVisualState) -> f64 {
    if state.phase.as_deref() == Some("Focus") {
        if let Some(rem) = state.seconds_remaining {
            return (rem as f64 / state.focus_secs.max(1) as f64).clamp(0.05, 1.0);
        }
    }
    ((state.focus_secs as f64 / 60.0) / 90.0).clamp(0.15, 0.95)
}

fn break_fraction(state: &RingVisualState) -> f64 {
    if state.phase.as_deref() == Some("Break") {
        if let Some(rem) = state.seconds_remaining {
            return (rem as f64 / state.break_secs.max(1) as f64).clamp(0.05, 1.0);
        }
    }
    ((state.break_secs as f64 / 60.0) / 30.0).clamp(0.10, 0.95)
}

fn draw_ring(
    cr: &gtk4::cairo::Context,
    width: f64,
    height: f64,
    fraction: f64,
    color: (f64, f64, f64),
) {
    let cx = width / 2.0;
    let cy = height / 2.0;
    let radius = (width.min(height) / 2.0) - 10.0;
    let start = -FRAC_PI_2;
    let sweep = 2.0 * PI * fraction.clamp(0.0, 1.0);
    let end = start + sweep;

    cr.set_line_width(12.0);
    cr.set_source_rgb(0.22, 0.22, 0.24);
    cr.arc(cx, cy, radius, 0.0, 2.0 * PI);
    let _ = cr.stroke();

    cr.set_source_rgb(color.0, color.1, color.2);
    cr.arc(cx, cy, radius, start, end);
    let _ = cr.stroke();

    let hx = cx + radius * end.cos();
    let hy = cy + radius * end.sin();
    cr.set_source_rgb(color.0, color.1, color.2);
    cr.arc(hx, hy, 5.5, 0.0, 2.0 * PI);
    let _ = cr.fill();

    cr.set_source_rgb(0.92, 0.92, 0.92);
    cr.arc(hx, hy, 3.5, 0.0, 2.0 * PI);
    let _ = cr.fill();
}
