mod dialogs;
mod draw_data;
mod drawing;
mod geometry;

use std::cell::RefCell;
use std::rc::Rc;

use chrono::{Datelike, Duration, Local};
use gtk4::prelude::*;
use relm4::prelude::*;
use shared::ipc::{RuleSetSummary, ScheduleSummary, ScheduleType};

use dialogs::{show_create_dialog, show_edit_dialog, show_view_dialog};
use draw_data::{DragMode, DrawData};
use drawing::draw_calendar;
use geometry::{
    clamp_hour_frac, hit_test_event, pixel_to_day_time, snap15, END_HOUR, HEADER_H, MARGIN_LEFT,
    MARGIN_RIGHT, START_HOUR,
};

const MIN_WEEK_OFFSET: i32 = -1;
const MAX_WEEK_OFFSET: i32 = 1;

pub struct ScheduleSection {
    week_offset: i32,
    draw_data: Rc<RefCell<DrawData>>,
    rule_sets: Vec<RuleSetSummary>,
}

#[derive(Debug)]
pub enum ScheduleInput {
    PrevWeek,
    NextWeek,
    Today,
    SchedulesUpdated(Vec<ScheduleSummary>),
    RuleSetsUpdated(Vec<RuleSetSummary>),
    #[allow(dead_code)]
    DragBegin(f64, f64),
    #[allow(dead_code)]
    DragUpdate(f64, f64, f64, f64),
    #[allow(dead_code)]
    DragEnd(f64, f64, f64, f64),
    ClickAt(f64, f64, f64, f64),
    ShowCreateDialog {
        col: usize,
        start_min: u32,
        end_min: u32,
    },
    ShowViewDialog {
        id: uuid::Uuid,
        name: String,
        col: usize,
        start_min: u32,
        end_min: u32,
        schedule_type: ScheduleType,
        rule_set_id: uuid::Uuid,
    },
    ShowEditDialog {
        id: uuid::Uuid,
        name: String,
        col: usize,
        start_min: u32,
        end_min: u32,
        schedule_type: ScheduleType,
        rule_set_id: uuid::Uuid,
    },
    CommitCreate {
        name: String,
        col: usize,
        start_min: u32,
        end_min: u32,
        specific_date: String,
        schedule_type: ScheduleType,
        rule_set_id: Option<uuid::Uuid>,
    },
    CommitEdit {
        id: uuid::Uuid,
        name: String,
        col: usize,
        start_min: u32,
        end_min: u32,
        schedule_type: ScheduleType,
        rule_set_id: Option<uuid::Uuid>,
    },
    CommitDelete(uuid::Uuid),
    CommitDragMove {
        id: uuid::Uuid,
        col: usize,
        start_min: u32,
        end_min: u32,
        specific_date: Option<String>,
    },
    CommitDragResize {
        id: uuid::Uuid,
        col: usize,
        start_min: u32,
        end_min: u32,
    },
    ResyncCalendar,
}

#[derive(Debug)]
pub enum ScheduleOutput {
    CreateSchedule {
        name: String,
        days: Vec<u8>,
        start_min: u32,
        end_min: u32,
        specific_date: String,
        schedule_type: ScheduleType,
        rule_set_id: Option<uuid::Uuid>,
    },
    UpdateSchedule {
        id: uuid::Uuid,
        name: String,
        days: Vec<u8>,
        start_min: u32,
        end_min: u32,
        schedule_type: ScheduleType,
        rule_set_id: Option<uuid::Uuid>,
        specific_date: Option<String>,
    },
    DeleteSchedule(uuid::Uuid),
    ResyncCalendar,
}

#[relm4::component(pub)]
impl Component for ScheduleSection {
    type Init = ();
    type Input = ScheduleInput;
    type Output = ScheduleOutput;
    type CommandOutput = ();

    view! {
        gtk4::Box {
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 0,
            set_margin_all: 16,

            // ── Navigation header ──────────────────────────────────────────
            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 8,
                set_margin_bottom: 12,

                gtk4::Button {
                    set_label: "‹",
                    #[watch]
                    set_sensitive: model.week_offset > MIN_WEEK_OFFSET,
                    connect_clicked => ScheduleInput::PrevWeek,
                },
                gtk4::Button {
                    set_label: "Today",
                    connect_clicked => ScheduleInput::Today,
                },
                gtk4::Button {
                    set_label: "›",
                    #[watch]
                    set_sensitive: model.week_offset < MAX_WEEK_OFFSET,
                    connect_clicked => ScheduleInput::NextWeek,
                },

                #[name = "week_label"]
                gtk4::Label {
                    #[watch]
                    set_label: &week_label_text(model.week_offset),
                    set_hexpand: true,
                    set_halign: gtk4::Align::Center,
                    add_css_class: "title-3",
                },

                gtk4::Button {
                    set_icon_name: "view-refresh-symbolic",
                    set_tooltip_text: Some("Resync calendar"),
                    add_css_class: "flat",
                    connect_clicked => ScheduleInput::ResyncCalendar,
                },
            },

            // ── Calendar canvas ────────────────────────────────────────────
            gtk4::ScrolledWindow {
                set_vexpand: true,
                set_hexpand: true,
                set_min_content_height: 400,

                #[name = "drawing_area"]
                gtk4::DrawingArea {
                    set_vexpand: true,
                    set_hexpand: true,
                    set_content_height: 900,
                },
            },
        }
    }

    fn init(_: (), root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let draw_data = Rc::new(RefCell::new(DrawData::default()));
        let model = ScheduleSection {
            week_offset: 0,
            draw_data: draw_data.clone(),
            rule_sets: vec![],
        };

        let widgets = view_output!();

        let dd = draw_data.clone();
        widgets
            .drawing_area
            .set_draw_func(move |da, cr, width, height| {
                draw_calendar(da, cr, width, height, &dd.borrow());
            });

        // Drag gesture — creates, moves, or resizes events
        let drag = gtk4::GestureDrag::new();

        {
            let dd = draw_data.clone();
            let da = widgets.drawing_area.clone();
            drag.connect_drag_begin(move |_, x, y| {
                let mut data = dd.borrow_mut();
                data.drag_start = Some((x, y));
                let w = da.width() as f64;
                let h = da.allocated_height() as f64;
                let week_offset = data.week_offset;
                let hit = hit_test_event(x, y, w, h, week_offset, &data.schedules);
                data.drag_mode =
                    if let Some((id, _name, col, start_min, end_min, imported, _stype, _rs)) = hit
                    {
                        if imported {
                            DragMode::None // imported events open a view dialog on click
                        } else {
                            let ef = clamp_hour_frac(end_min as f64 / 60.0);
                            let sf = clamp_hour_frac(start_min as f64 / 60.0);
                            let y_start = HEADER_H + sf * (h - HEADER_H);
                            let y_end = HEADER_H + ef * (h - HEADER_H);
                            if y >= y_end - 10.0 {
                                DragMode::Resize {
                                    id,
                                    col,
                                    start_min,
                                    end_min,
                                    from_top: false,
                                }
                            } else if y <= y_start + 10.0 {
                                DragMode::Resize {
                                    id,
                                    col,
                                    start_min,
                                    end_min,
                                    from_top: true,
                                }
                            } else {
                                let hour_h =
                                    (h - HEADER_H) / (END_HOUR - START_HOUR) as f64;
                                let click_offset_min =
                                    ((y - y_start) / hour_h * 60.0) as i32;
                                let duration_min = end_min.saturating_sub(start_min);
                                DragMode::Move {
                                    id,
                                    col,
                                    start_min,
                                    end_min,
                                    duration_min,
                                    click_offset_min,
                                }
                            }
                        }
                    } else {
                        DragMode::Create {
                            col: 0,
                            start_min: 0,
                            end_min: 0,
                        }
                    };
            });
        }

        {
            let dd = draw_data.clone();
            let da = widgets.drawing_area.clone();
            drag.connect_drag_update(move |_, off_x, off_y| {
                let mut data = dd.borrow_mut();
                let w = da.width() as f64;
                let h = da.allocated_height() as f64;
                let hour_h = (h - HEADER_H) / (END_HOUR - START_HOUR) as f64;

                match data.drag_mode.clone() {
                    DragMode::Create { .. } => {
                        if let Some((sx, sy)) = data.drag_start {
                            let cx = sx + off_x;
                            let cy = sy + off_y;
                            if let (Some((col, s_min)), Some((_, e_min_raw))) = (
                                pixel_to_day_time(sx, sy, w, h),
                                pixel_to_day_time(cx, cy, w, h),
                            ) {
                                let (s, e) = if e_min_raw >= s_min {
                                    (s_min, e_min_raw.max(s_min + 15))
                                } else {
                                    (e_min_raw, s_min)
                                };
                                data.drag_mode = DragMode::Create {
                                    col,
                                    start_min: snap15(s),
                                    end_min: snap15(e),
                                };
                            }
                        }
                    }
                    DragMode::Move {
                        id,
                        duration_min,
                        click_offset_min,
                        ..
                    } => {
                        if let Some((sx, sy)) = data.drag_start {
                            let cx = sx + off_x;
                            let cy = sy + off_y;
                            let col_w = (w - MARGIN_LEFT - MARGIN_RIGHT) / 7.0;
                            let new_col = if cx >= MARGIN_LEFT {
                                (((cx - MARGIN_LEFT) / col_w) as usize).min(6)
                            } else {
                                0
                            };
                            let top_y = cy - click_offset_min as f64 / 60.0 * hour_h;
                            let new_start_raw = if top_y >= HEADER_H {
                                let hour_frac = (top_y - HEADER_H) / hour_h;
                                snap15(
                                    (START_HOUR as f64 * 60.0 + hour_frac * 60.0) as u32,
                                )
                            } else {
                                START_HOUR * 60
                            };
                            let new_start =
                                new_start_raw.clamp(START_HOUR * 60, END_HOUR * 60);
                            let new_end = (new_start + duration_min).min(END_HOUR * 60);
                            let new_start = new_end.saturating_sub(duration_min);
                            data.drag_mode = DragMode::Move {
                                id,
                                col: new_col,
                                start_min: new_start,
                                end_min: new_end,
                                duration_min,
                                click_offset_min,
                            };
                        }
                    }
                    DragMode::Resize {
                        id,
                        col,
                        start_min,
                        end_min,
                        from_top,
                    } => {
                        if let Some((sx, sy)) = data.drag_start {
                            let cy = sy + off_y;
                            if let Some((_, raw_min)) = pixel_to_day_time(sx, cy, w, h) {
                                let snapped = snap15(raw_min);
                                let (new_start, new_end) = if from_top {
                                    let s = snapped
                                        .min(end_min.saturating_sub(15))
                                        .max(START_HOUR * 60);
                                    (s, end_min)
                                } else {
                                    let e = snapped
                                        .max(start_min + 15)
                                        .min(END_HOUR * 60);
                                    (start_min, e)
                                };
                                data.drag_mode = DragMode::Resize {
                                    id,
                                    col,
                                    start_min: new_start,
                                    end_min: new_end,
                                    from_top,
                                };
                            }
                        }
                    }
                    DragMode::None => {}
                }
                drop(data);
                da.queue_draw();
            });
        }

        {
            let dd = draw_data.clone();
            let da = widgets.drawing_area.clone();
            let s = sender.clone();
            drag.connect_drag_end(move |_, off_x, off_y| {
                let dist = (off_x * off_x + off_y * off_y).sqrt();

                let mut data = dd.borrow_mut();
                let mode = std::mem::replace(&mut data.drag_mode, DragMode::None);
                let start_pos = data.drag_start.take();

                // Optimistically apply the final position BEFORE queue_draw so
                // the canvas never shows the old position after release.
                let mut new_specific_date: Option<String> = None;
                let week_offset = data.week_offset;
                if dist > 10.0 {
                    match &mode {
                        DragMode::Move {
                            id,
                            col,
                            start_min,
                            end_min,
                            ..
                        } => {
                            if let Some(sched) =
                                data.schedules.iter_mut().find(|s| s.id == *id)
                            {
                                if sched.specific_date.is_some() {
                                    let today = chrono::Local::now().date_naive();
                                    let dfm =
                                        today.weekday().num_days_from_monday() as i64;
                                    let this_mon =
                                        today - chrono::Duration::days(dfm);
                                    let week_mon = this_mon
                                        + chrono::Duration::weeks(week_offset as i64);
                                    let new_date = week_mon
                                        + chrono::Duration::days(*col as i64);
                                    let date_str =
                                        new_date.format("%Y-%m-%d").to_string();
                                    sched.specific_date = Some(date_str.clone());
                                    new_specific_date = Some(date_str);
                                }
                                sched.days = vec![*col as u8];
                                sched.start_min = *start_min;
                                sched.end_min = *end_min;
                            }
                        }
                        DragMode::Resize {
                            id,
                            col,
                            start_min,
                            end_min,
                            ..
                        } => {
                            if let Some(sched) =
                                data.schedules.iter_mut().find(|s| s.id == *id)
                            {
                                sched.days = vec![*col as u8];
                                sched.start_min = *start_min;
                                sched.end_min = *end_min;
                            }
                        }
                        _ => {}
                    }
                }

                drop(data);
                da.queue_draw();

                if dist <= 10.0 {
                    if let Some((x, y)) = start_pos {
                        let w = da.width() as f64;
                        let h = da.allocated_height() as f64;
                        s.input(ScheduleInput::ClickAt(x, y, w, h));
                    }
                    return;
                }

                match mode {
                    DragMode::Create {
                        col,
                        start_min,
                        end_min,
                    } => {
                        if end_min > start_min + 14 {
                            s.input(ScheduleInput::ShowCreateDialog {
                                col,
                                start_min,
                                end_min,
                            });
                        }
                    }
                    DragMode::Move {
                        id,
                        col,
                        start_min,
                        end_min,
                        ..
                    } => {
                        s.input(ScheduleInput::CommitDragMove {
                            id,
                            col,
                            start_min,
                            end_min,
                            specific_date: new_specific_date,
                        });
                    }
                    DragMode::Resize {
                        id,
                        col,
                        start_min,
                        end_min,
                        ..
                    } => {
                        s.input(ScheduleInput::CommitDragResize {
                            id,
                            col,
                            start_min,
                            end_min,
                        });
                    }
                    DragMode::None => {}
                }
            });
        }
        widgets.drawing_area.add_controller(drag);

        // Motion controller — update cursor based on what's under the pointer
        let motion = gtk4::EventControllerMotion::new();
        {
            let dd = draw_data.clone();
            let da = widgets.drawing_area.clone();
            motion.connect_motion(move |_, x, y| {
                let data = dd.borrow();
                let w = da.width() as f64;
                let h = da.allocated_height() as f64;
                let col_w = (w - MARGIN_LEFT - MARGIN_RIGHT) / 7.0;

                let cursor = 'cursor: {
                    for sched in &data.schedules {
                        if !sched.enabled || sched.imported {
                            continue;
                        }

                        let cols: Vec<usize> = if let Some(ds) = &sched.specific_date {
                            if let Ok(date) =
                                chrono::NaiveDate::parse_from_str(ds, "%Y-%m-%d")
                            {
                                let today = chrono::Local::now().date_naive();
                                let dfm =
                                    today.weekday().num_days_from_monday() as i64;
                                let this_mon = today - chrono::Duration::days(dfm);
                                let week_mon = this_mon
                                    + chrono::Duration::weeks(data.week_offset as i64);
                                let off = (date - week_mon).num_days();
                                if off >= 0 && off < 7 {
                                    vec![off as usize]
                                } else {
                                    vec![]
                                }
                            } else {
                                vec![]
                            }
                        } else {
                            sched.days.iter().map(|&d| d as usize).collect()
                        };

                        for col in cols {
                            let bx = MARGIN_LEFT + col as f64 * col_w + 2.0;
                            let bw = col_w - 4.0;
                            let sf = clamp_hour_frac(sched.start_min as f64 / 60.0);
                            let ef = clamp_hour_frac(sched.end_min as f64 / 60.0);
                            let ys = HEADER_H + sf * (h - HEADER_H);
                            let ye = HEADER_H + ef * (h - HEADER_H);
                            let bh = (ye - ys).max(4.0);

                            if x >= bx && x <= bx + bw && y >= ys && y <= ys + bh {
                                if y <= ys + 10.0 || y >= ye - 10.0 {
                                    break 'cursor "ns-resize";
                                } else {
                                    break 'cursor "grab";
                                }
                            }
                        }
                    }
                    "default"
                };
                da.set_cursor_from_name(Some(cursor));
            });
        }
        widgets.drawing_area.add_controller(motion);

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: ScheduleInput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            ScheduleInput::PrevWeek => {
                self.week_offset = (self.week_offset - 1).max(MIN_WEEK_OFFSET);
                self.draw_data.borrow_mut().week_offset = self.week_offset;
            }
            ScheduleInput::NextWeek => {
                self.week_offset = (self.week_offset + 1).min(MAX_WEEK_OFFSET);
                self.draw_data.borrow_mut().week_offset = self.week_offset;
            }
            ScheduleInput::Today => {
                self.week_offset = 0;
                self.draw_data.borrow_mut().week_offset = 0;
            }
            ScheduleInput::SchedulesUpdated(schedules) => {
                self.draw_data.borrow_mut().schedules = schedules;
                widgets.drawing_area.queue_draw();
            }
            ScheduleInput::DragBegin(..)
            | ScheduleInput::DragUpdate(..)
            | ScheduleInput::DragEnd(..) => {
                // handled by gesture closures directly
                widgets.drawing_area.queue_draw();
            }
            ScheduleInput::ClickAt(x, y, w, h) => {
                let hit = {
                    let data = self.draw_data.borrow();
                    hit_test_event(x, y, w, h, data.week_offset, &data.schedules)
                };
                if let Some((id, name, col, start_min, end_min, imported, schedule_type, rule_set_id)) = hit {
                    if imported {
                        sender.input(ScheduleInput::ShowViewDialog {
                            id,
                            name,
                            col,
                            start_min,
                            end_min,
                            schedule_type,
                            rule_set_id,
                        });
                    } else {
                        sender.input(ScheduleInput::ShowEditDialog {
                            id,
                            name,
                            col,
                            start_min,
                            end_min,
                            schedule_type,
                            rule_set_id,
                        });
                    }
                }
                self.update_view(widgets, sender);
                return;
            }
            ScheduleInput::ShowViewDialog {
                id,
                name,
                col,
                start_min,
                end_min,
                schedule_type,
                rule_set_id,
            } => {
                let week_monday = {
                    let data = self.draw_data.borrow();
                    let today = chrono::Local::now().date_naive();
                    let dfm = today.weekday().num_days_from_monday() as i64;
                    let this_mon = today - chrono::Duration::days(dfm);
                    this_mon + chrono::Duration::weeks(data.week_offset as i64)
                };
                let rule_sets = self.rule_sets.clone();
                show_view_dialog(
                    id,
                    &name,
                    col,
                    start_min,
                    end_min,
                    schedule_type,
                    rule_set_id,
                    week_monday,
                    rule_sets,
                    _root,
                    sender.clone(),
                );
                self.update_view(widgets, sender);
                return;
            }
            ScheduleInput::ShowCreateDialog {
                col,
                start_min,
                end_min,
            } => {
                let week_monday = {
                    let data = self.draw_data.borrow();
                    let today = chrono::Local::now().date_naive();
                    let dfm = today.weekday().num_days_from_monday() as i64;
                    let this_mon = today - chrono::Duration::days(dfm);
                    this_mon + chrono::Duration::weeks(data.week_offset as i64)
                };
                show_create_dialog(
                    col,
                    start_min,
                    end_min,
                    week_monday,
                    self.rule_sets.clone(),
                    _root,
                    sender.clone(),
                );
                self.update_view(widgets, sender);
                return;
            }
            ScheduleInput::ShowEditDialog {
                id,
                name,
                col,
                start_min,
                end_min,
                schedule_type,
                rule_set_id,
            } => {
                let rule_sets = self.rule_sets.clone();
                show_edit_dialog(
                    id,
                    &name,
                    col,
                    start_min,
                    end_min,
                    schedule_type,
                    rule_set_id,
                    rule_sets,
                    _root,
                    sender.clone(),
                );
                self.update_view(widgets, sender);
                return;
            }
            ScheduleInput::RuleSetsUpdated(sets) => {
                self.rule_sets = sets;
            }
            ScheduleInput::CommitCreate {
                name,
                col,
                start_min,
                end_min,
                specific_date,
                schedule_type,
                rule_set_id,
            } => {
                let _ = sender.output(ScheduleOutput::CreateSchedule {
                    name,
                    days: vec![col as u8],
                    start_min,
                    end_min,
                    specific_date,
                    schedule_type,
                    rule_set_id,
                });
            }
            ScheduleInput::CommitEdit {
                id,
                name,
                col,
                start_min,
                end_min,
                schedule_type,
                rule_set_id,
            } => {
                let _ = sender.output(ScheduleOutput::UpdateSchedule {
                    id,
                    name,
                    days: vec![col as u8],
                    start_min,
                    end_min,
                    schedule_type,
                    rule_set_id,
                    specific_date: None,
                });
            }
            ScheduleInput::CommitDelete(id) => {
                let _ = sender.output(ScheduleOutput::DeleteSchedule(id));
            }
            ScheduleInput::ResyncCalendar => {
                let _ = sender.output(ScheduleOutput::ResyncCalendar);
            }
            ScheduleInput::CommitDragMove {
                id,
                col,
                start_min,
                end_min,
                specific_date,
            } => {
                let sched = self
                    .draw_data
                    .borrow()
                    .schedules
                    .iter()
                    .find(|s| s.id == id)
                    .cloned();
                if let Some(sched) = sched {
                    let rule_set_id = if sched.rule_set_id.is_nil() {
                        None
                    } else {
                        Some(sched.rule_set_id)
                    };
                    let _ = sender.output(ScheduleOutput::UpdateSchedule {
                        id,
                        name: sched.name,
                        days: vec![col as u8],
                        start_min,
                        end_min,
                        schedule_type: sched.schedule_type,
                        rule_set_id,
                        specific_date,
                    });
                }
            }
            ScheduleInput::CommitDragResize {
                id,
                col,
                start_min,
                end_min,
            } => {
                let sched = self
                    .draw_data
                    .borrow()
                    .schedules
                    .iter()
                    .find(|s| s.id == id)
                    .cloned();
                if let Some(sched) = sched {
                    let rule_set_id = if sched.rule_set_id.is_nil() {
                        None
                    } else {
                        Some(sched.rule_set_id)
                    };
                    let _ = sender.output(ScheduleOutput::UpdateSchedule {
                        id,
                        name: sched.name,
                        days: vec![col as u8],
                        start_min,
                        end_min,
                        schedule_type: sched.schedule_type,
                        rule_set_id,
                        specific_date: None,
                    });
                }
            }
        }
        widgets.drawing_area.queue_draw();
        self.update_view(widgets, sender);
    }
}

fn week_label_text(offset: i32) -> String {
    let today = Local::now().date_naive();
    let days_from_mon = today.weekday().num_days_from_monday() as i64;
    let this_monday = today - Duration::days(days_from_mon);
    let week_monday = this_monday + Duration::weeks(offset as i64);
    let week_sunday = week_monday + Duration::days(6);

    if week_monday.month() == week_sunday.month() {
        format!(
            "{} {}–{}",
            week_monday.format("%b"),
            week_monday.day(),
            week_sunday.day()
        )
    } else {
        format!(
            "{} {} – {} {}",
            week_monday.format("%b"),
            week_monday.day(),
            week_sunday.format("%b"),
            week_sunday.day()
        )
    }
}
