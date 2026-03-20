use super::ring::{break_fraction, draw_ring, focus_fraction, minutes_from_ring_pos, RingVisualState};

use gtk4::prelude::*;
use relm4::prelude::*;
use shared::ipc::RuleSetSummary;
use std::cell::RefCell;
use std::rc::Rc;
use uuid::Uuid;

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
    SelectPreset {
        focus_secs: u64,
        break_secs: u64,
    },
    SetQuickBreak {
        break_secs: u64,
    },
    AdjustFocus(i64),
    AdjustBreak(i64),
    DragFocusAt {
        x: f64,
        y: f64,
        w: f64,
        h: f64,
    },
    DragBreakAt {
        x: f64,
        y: f64,
        w: f64,
        h: f64,
    },
    Start,
    Stop,
    RuleSetRowSelected(i32),
    StatusUpdated {
        phase: Option<String>,
        seconds_remaining: Option<u64>,
    },
    RuleSetsUpdated(Vec<RuleSetSummary>),
}

#[derive(Debug)]
pub enum PomodoroOutput {
    Start {
        focus_secs: u64,
        break_secs: u64,
        rule_set_id: Option<Uuid>,
    },
    Stop,
}

fn adjust_duration_secs(current_secs: u64, delta_min: i64, min_m: u64, max_m: u64) -> u64 {
    let new_mins = ((current_secs / 60) as i64 + delta_min).clamp(min_m as i64, max_m as i64);
    new_mins as u64 * 60
}

fn restored_rule_set_id(prev_id: Option<Uuid>, sets: &[RuleSetSummary]) -> Option<Uuid> {
    prev_id
        .filter(|id| sets.iter().any(|s| s.id == *id))
        .or_else(|| sets.first().map(|s| s.id))
}

/// Attaches a drag gesture to `ring`.
///
/// `dims` is a shared cell written by the draw callback so that the gesture
/// uses the exact same `(w, h)` as the draw function — guaranteeing that the
/// centre point used for angle calculation matches the visual centre of the arc.
fn attach_ring_drag<F>(
    ring: &gtk4::DrawingArea,
    sender: ComponentSender<PomodoroSection>,
    dims: Rc<RefCell<(f64, f64)>>,
    make_msg: F,
) where
    F: Fn(f64, f64, f64, f64) -> PomodoroInput + Clone + 'static,
{
    let drag = gtk4::GestureDrag::new();

    let s = sender.clone();
    let d = dims.clone();
    let make_begin = make_msg.clone();
    drag.connect_drag_begin(move |_, x, y| {
        let (w, h) = *d.borrow();
        s.input(make_begin(x, y, w, h));
    });

    drag.connect_drag_update(move |gesture, off_x, off_y| {
        let (w, h) = *dims.borrow();
        if let Some((sx, sy)) = gesture.start_point() {
            sender.input(make_msg(sx + off_x, sy + off_y, w, h));
        }
    });

    ring.add_controller(drag);
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
                            set_spacing: 6,
                            set_width_request: 110,

                            gtk4::Label {
                                set_label: "PRESETS",
                                add_css_class: "dim-label",
                                set_halign: gtk4::Align::Start,
                            },
                            gtk4::Button {
                                set_label: "25/5",
                                connect_clicked => PomodoroInput::SelectPreset { focus_secs: 25 * 60, break_secs: 5 * 60 },
                            },
                            gtk4::Button {
                                set_label: "45/15",
                                add_css_class: "suggested-action",
                                connect_clicked => PomodoroInput::SelectPreset { focus_secs: 45 * 60, break_secs: 15 * 60 },
                            },
                            gtk4::Button {
                                set_label: "50/10",
                                connect_clicked => PomodoroInput::SelectPreset { focus_secs: 50 * 60, break_secs: 10 * 60 },
                            },
                            gtk4::Button {
                                set_label: "90/20",
                                connect_clicked => PomodoroInput::SelectPreset { focus_secs: 90 * 60, break_secs: 20 * 60 },
                            },

                            gtk4::Separator {
                                set_orientation: gtk4::Orientation::Horizontal,
                                set_margin_top: 4,
                                set_margin_bottom: 2,
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

                            // FOCUS ring (no inner frame)
                            gtk4::Box {
                                set_orientation: gtk4::Orientation::Vertical,
                                set_spacing: 8,
                                set_halign: gtk4::Align::Center,

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
                                        set_content_width: 200,
                                        set_content_height: 200,
                                    },

                                    add_overlay = &gtk4::Box {
                                        set_orientation: gtk4::Orientation::Vertical,
                                        set_halign: gtk4::Align::Center,
                                        set_valign: gtk4::Align::Center,
                                        set_spacing: 4,

                                        gtk4::Label {
                                            set_use_markup: true,
                                            set_markup: "<span size='20480'>🍃</span>",
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

                            // BREAK ring (no inner frame)
                            gtk4::Box {
                                set_orientation: gtk4::Orientation::Vertical,
                                set_spacing: 8,
                                set_halign: gtk4::Align::Center,

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
                                        set_content_width: 200,
                                        set_content_height: 200,
                                    },

                                    add_overlay = &gtk4::Box {
                                        set_orientation: gtk4::Orientation::Vertical,
                                        set_halign: gtk4::Align::Center,
                                        set_valign: gtk4::Align::Center,
                                        set_spacing: 4,

                                        gtk4::Label {
                                            set_use_markup: true,
                                            set_markup: "<span size='20480'>☕</span>",
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

                    gtk4::Label {
                        #[watch]
                        set_label: &match (model.phase.as_deref(), model.seconds_remaining) {
                            (Some(phase), Some(secs)) => {
                                format!("{phase} - {:02}:{:02}", secs / 60, secs % 60)
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

                    #[name = "rule_set_list"]
                    gtk4::ListBox {
                        set_selection_mode: gtk4::SelectionMode::Single,
                        set_hexpand: true,
                        add_css_class: "boxed-list",
                    },

                    gtk4::Box {
                        set_orientation: gtk4::Orientation::Horizontal,
                        set_spacing: 8,
                        gtk4::Button {
                            set_label: "Start Focus Session",
                            add_css_class: "suggested-action",
                            set_hexpand: true,
                            #[watch]
                            set_sensitive: model.phase.is_none(),
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
            let s = _sender.clone();
            widgets.rule_set_list.connect_row_selected(move |_, row| {
                s.input(PomodoroInput::RuleSetRowSelected(
                    row.map(|r| r.index()).unwrap_or(-1),
                ));
            });
        }

        // Shared draw dimensions — written by the draw callback, read by the
        // gesture handler, so both always use the same (w, h) and thus the
        // same arc centre.
        let focus_dims: Rc<RefCell<(f64, f64)>> = Rc::new(RefCell::new((200.0, 200.0)));
        let break_dims: Rc<RefCell<(f64, f64)>> = Rc::new(RefCell::new((200.0, 200.0)));

        let ring = model.ring_visual.clone();
        let fd = focus_dims.clone();
        widgets.focus_ring.set_draw_func(move |_, cr, w, h| {
            *fd.borrow_mut() = (w as f64, h as f64);
            draw_ring(cr, w as f64, h as f64, focus_fraction(&ring.borrow()), (0.12, 0.55, 0.95));
        });

        let ring = model.ring_visual.clone();
        let bd = break_dims.clone();
        widgets.break_ring.set_draw_func(move |_, cr, w, h| {
            *bd.borrow_mut() = (w as f64, h as f64);
            draw_ring(cr, w as f64, h as f64, break_fraction(&ring.borrow()), (0.98, 0.60, 0.18));
        });

        attach_ring_drag(&widgets.focus_ring, _sender.clone(), focus_dims, |x, y, w, h| {
            PomodoroInput::DragFocusAt { x, y, w, h }
        });
        attach_ring_drag(&widgets.break_ring, _sender, break_dims, |x, y, w, h| {
            PomodoroInput::DragBreakAt { x, y, w, h }
        });

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
            }
            PomodoroInput::SetQuickBreak { break_secs } => {
                self.break_secs = break_secs;
            }
            PomodoroInput::AdjustFocus(delta_min) => {
                self.focus_secs = adjust_duration_secs(self.focus_secs, delta_min, 5, 180);
            }
            PomodoroInput::AdjustBreak(delta_min) => {
                self.break_secs = adjust_duration_secs(self.break_secs, delta_min, 1, 90);
            }
            PomodoroInput::DragFocusAt { x, y, w, h } => {
                // Use the same display normalisation as focus_fraction (0–90 min).
                self.focus_secs = minutes_from_ring_pos(x, y, w, h, 0, 90).max(5) * 60;
            }
            PomodoroInput::DragBreakAt { x, y, w, h } => {
                // Use the same display normalisation as break_fraction (0–30 min).
                self.break_secs = minutes_from_ring_pos(x, y, w, h, 0, 30).max(1) * 60;
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
            PomodoroInput::RuleSetRowSelected(idx) => {
                self.selected_rule_set_id = if idx >= 0 {
                    self.rule_sets.get(idx as usize).map(|rs| rs.id)
                } else {
                    None
                };
            }
            PomodoroInput::StatusUpdated {
                phase,
                seconds_remaining,
            } => {
                self.phase = phase;
                self.seconds_remaining = seconds_remaining;
            }
            PomodoroInput::RuleSetsUpdated(sets) => {
                while let Some(child) = widgets.rule_set_list.first_child() {
                    widgets.rule_set_list.remove(&child);
                }
                for (i, rs) in sets.iter().enumerate() {
                    let label_text = if i == 0 {
                        format!("{} (default)", rs.name)
                    } else {
                        rs.name.clone()
                    };
                    let label = gtk4::Label::new(Some(&label_text));
                    label.set_halign(gtk4::Align::Start);
                    label.set_margin_start(8);
                    label.set_margin_end(8);
                    label.set_margin_top(6);
                    label.set_margin_bottom(6);
                    let row = gtk4::ListBoxRow::new();
                    row.set_child(Some(&label));
                    widgets.rule_set_list.append(&row);
                }
                let restore_id = restored_rule_set_id(self.selected_rule_set_id, &sets);
                if let Some(id) = restore_id {
                    let idx = sets.iter().position(|rs| rs.id == id).unwrap_or(0);
                    if let Some(row) = widgets.rule_set_list.row_at_index(idx as i32) {
                        widgets.rule_set_list.select_row(Some(&row));
                    }
                    self.selected_rule_set_id = Some(id);
                } else {
                    widgets.rule_set_list.unselect_all();
                    self.selected_rule_set_id = None;
                }
                self.rule_sets = sets;
            }
        }

        // Keep ring_visual in sync with model so draw callbacks see current state.
        {
            let mut rv = self.ring_visual.borrow_mut();
            rv.focus_secs = self.focus_secs;
            rv.break_secs = self.break_secs;
            rv.phase = self.phase.clone();
            rv.seconds_remaining = self.seconds_remaining;
        }

        widgets.focus_ring.queue_draw();
        widgets.break_ring.queue_draw();
        self.update_view(widgets, sender);
    }
}


#[cfg(test)]
#[path = "tests.rs"]
mod tests;
