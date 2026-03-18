use std::cell::RefCell;
use std::rc::Rc;

use chrono::{Datelike, Duration, Local, Timelike};
use gtk4::prelude::*;
use relm4::prelude::*;
use shared::ipc::{RuleSetSummary, ScheduleSummary, ScheduleType};

// Visible hour range
const START_HOUR: u32 = 6;
const END_HOUR: u32 = 23;

// Palette for event blocks (R, G, B in 0..1)
const COLORS: &[(f64, f64, f64)] = &[
    (0.26, 0.54, 0.96), // blue
    (0.18, 0.69, 0.51), // teal
    (0.93, 0.42, 0.22), // orange
    (0.62, 0.32, 0.82), // purple
    (0.24, 0.71, 0.29), // green
    (0.95, 0.26, 0.45), // pink
];

#[derive(Debug, Clone, Default)]
enum DragMode {
    #[default]
    None,
    Create {
        col: usize,
        start_min: u32,
        end_min: u32,
    },
    Move {
        id: uuid::Uuid,
        col: usize,
        start_min: u32,
        end_min: u32,
        duration_min: u32,
        click_offset_min: i32,
    },
    Resize {
        id: uuid::Uuid,
        col: usize,
        start_min: u32,
        end_min: u32,
        from_top: bool,
    },
}

#[derive(Debug, Default)]
struct DrawData {
    schedules: Vec<ScheduleSummary>,
    week_offset: i32,
    drag_start: Option<(f64, f64)>,
    drag_mode: DragMode,
}

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
    #[allow(dead_code)] DragBegin(f64, f64),
    #[allow(dead_code)] DragUpdate(f64, f64, f64, f64),
    #[allow(dead_code)] DragEnd(f64, f64, f64, f64),
    ClickAt(f64, f64, f64, f64),
    ShowCreateDialog { col: usize, start_min: u32, end_min: u32 },
    ShowViewDialog { id: uuid::Uuid, name: String, col: usize, start_min: u32, end_min: u32, schedule_type: ScheduleType, rule_set_id: uuid::Uuid },
    ShowEditDialog { id: uuid::Uuid, name: String, col: usize, start_min: u32, end_min: u32, schedule_type: ScheduleType, rule_set_id: uuid::Uuid },
    CommitCreate { name: String, col: usize, start_min: u32, end_min: u32, specific_date: String, schedule_type: ScheduleType, rule_set_id: Option<uuid::Uuid> },
    CommitEdit { id: uuid::Uuid, name: String, col: usize, start_min: u32, end_min: u32, schedule_type: ScheduleType, rule_set_id: Option<uuid::Uuid> },
    CommitDelete(uuid::Uuid),
    CommitDragMove { id: uuid::Uuid, col: usize, start_min: u32, end_min: u32 },
    CommitDragResize { id: uuid::Uuid, col: usize, start_min: u32, end_min: u32 },
}

#[derive(Debug)]
pub enum ScheduleOutput {
    CreateSchedule { name: String, days: Vec<u8>, start_min: u32, end_min: u32, specific_date: String, schedule_type: ScheduleType, rule_set_id: Option<uuid::Uuid> },
    UpdateSchedule { id: uuid::Uuid, name: String, days: Vec<u8>, start_min: u32, end_min: u32, schedule_type: ScheduleType, rule_set_id: Option<uuid::Uuid> },
    DeleteSchedule(uuid::Uuid),
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
                    connect_clicked => ScheduleInput::PrevWeek,
                },
                gtk4::Button {
                    set_label: "Today",
                    connect_clicked => ScheduleInput::Today,
                },
                gtk4::Button {
                    set_label: "›",
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
                data.drag_mode = if let Some((id, _name, col, start_min, end_min, imported, _stype, _rs)) = hit {
                    if imported {
                        DragMode::None // imported events open a view dialog on click
                    } else {
                        const HEADER_H: f64 = 40.0;
                        let ef = clamp_hour_frac(end_min as f64 / 60.0);
                        let sf = clamp_hour_frac(start_min as f64 / 60.0);
                        let y_start = HEADER_H + sf * (h - HEADER_H);
                        let y_end = HEADER_H + ef * (h - HEADER_H);
                        if y >= y_end - 10.0 {
                            // Bottom-edge zone → resize end time
                            DragMode::Resize { id, col, start_min, end_min, from_top: false }
                        } else if y <= y_start + 10.0 {
                            // Top-edge zone → resize start time
                            DragMode::Resize { id, col, start_min, end_min, from_top: true }
                        } else {
                            // Body → move; record where inside the block the user clicked
                            let sf = clamp_hour_frac(start_min as f64 / 60.0);
                            let y_start = HEADER_H + sf * (h - HEADER_H);
                            let hour_h = (h - HEADER_H) / (END_HOUR - START_HOUR) as f64;
                            let click_offset_min = ((y - y_start) / hour_h * 60.0) as i32;
                            let duration_min = end_min.saturating_sub(start_min);
                            DragMode::Move { id, col, start_min, end_min, duration_min, click_offset_min }
                        }
                    }
                } else {
                    DragMode::Create { col: 0, start_min: 0, end_min: 0 }
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
                const HEADER_H: f64 = 40.0;
                const MARGIN_LEFT: f64 = 52.0;
                const MARGIN_RIGHT: f64 = 4.0;
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
                    DragMode::Move { id, duration_min, click_offset_min, .. } => {
                        if let Some((sx, sy)) = data.drag_start {
                            let cx = sx + off_x;
                            let cy = sy + off_y;
                            // Compute new column from current x
                            let col_w = (w - MARGIN_LEFT - MARGIN_RIGHT) / 7.0;
                            let new_col = if cx >= MARGIN_LEFT {
                                (((cx - MARGIN_LEFT) / col_w) as usize).min(6)
                            } else { 0 };
                            // Compute new start from current y adjusted by click offset
                            let top_y = cy - click_offset_min as f64 / 60.0 * hour_h;
                            let new_start_raw = if top_y >= HEADER_H {
                                let hour_frac = (top_y - HEADER_H) / hour_h;
                                snap15((START_HOUR as f64 * 60.0 + hour_frac * 60.0) as u32)
                            } else {
                                START_HOUR * 60
                            };
                            let new_start = new_start_raw.clamp(START_HOUR * 60, END_HOUR * 60);
                            let new_end = (new_start + duration_min).min(END_HOUR * 60);
                            let new_start = new_end.saturating_sub(duration_min);
                            data.drag_mode = DragMode::Move {
                                id, col: new_col,
                                start_min: new_start, end_min: new_end,
                                duration_min, click_offset_min,
                            };
                        }
                    }
                    DragMode::Resize { id, col, start_min, end_min, from_top } => {
                        if let Some((sx, sy)) = data.drag_start {
                            let cy = sy + off_y;
                            if let Some((_, raw_min)) = pixel_to_day_time(sx, cy, w, h) {
                                let snapped = snap15(raw_min);
                                let (new_start, new_end) = if from_top {
                                    let s = snapped.min(end_min.saturating_sub(15)).max(START_HOUR * 60);
                                    (s, end_min)
                                } else {
                                    let e = snapped.max(start_min + 15).min(END_HOUR * 60);
                                    (start_min, e)
                                };
                                data.drag_mode = DragMode::Resize { id, col, start_min: new_start, end_min: new_end, from_top };
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
                let mut data = dd.borrow_mut();
                let mode = std::mem::replace(&mut data.drag_mode, DragMode::None);
                let start_pos = data.drag_start.take();
                drop(data);
                da.queue_draw();

                let dist = (off_x * off_x + off_y * off_y).sqrt();

                if dist <= 10.0 {
                    // Treat as click regardless of mode
                    if let Some((x, y)) = start_pos {
                        let w = da.width() as f64;
                        let h = da.allocated_height() as f64;
                        s.input(ScheduleInput::ClickAt(x, y, w, h));
                    }
                    return;
                }

                match mode {
                    DragMode::Create { col, start_min, end_min } => {
                        if end_min > start_min + 14 {
                            s.input(ScheduleInput::ShowCreateDialog { col, start_min, end_min });
                        }
                    }
                    DragMode::Move { id, col, start_min, end_min, .. } => {
                        s.input(ScheduleInput::CommitDragMove { id, col, start_min, end_min });
                    }
                    DragMode::Resize { id, col, start_min, end_min, .. } => {
                        s.input(ScheduleInput::CommitDragResize { id, col, start_min, end_min });
                    }
                    DragMode::None => {}
                }
            });
        }
        widgets.drawing_area.add_controller(drag);

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
                self.week_offset -= 1;
                self.draw_data.borrow_mut().week_offset = self.week_offset;
            }
            ScheduleInput::NextWeek => {
                self.week_offset += 1;
                self.draw_data.borrow_mut().week_offset = self.week_offset;
            }
            ScheduleInput::Today => {
                self.week_offset = 0;
                self.draw_data.borrow_mut().week_offset = 0;
            }
            ScheduleInput::SchedulesUpdated(schedules) => {
                self.draw_data.borrow_mut().schedules = schedules;
            }
            ScheduleInput::DragBegin(..) | ScheduleInput::DragUpdate(..) | ScheduleInput::DragEnd(..) => {
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
                        sender.input(ScheduleInput::ShowViewDialog { id, name, col, start_min, end_min, schedule_type, rule_set_id });
                    } else {
                        sender.input(ScheduleInput::ShowEditDialog { id, name, col, start_min, end_min, schedule_type, rule_set_id });
                    }
                }
                self.update_view(widgets, sender);
                return;
            }
            ScheduleInput::ShowViewDialog { id, name, col, start_min, end_min, schedule_type, rule_set_id } => {
                let week_monday = {
                    let data = self.draw_data.borrow();
                    let today = chrono::Local::now().date_naive();
                    let dfm = today.weekday().num_days_from_monday() as i64;
                    let this_mon = today - chrono::Duration::days(dfm);
                    this_mon + chrono::Duration::weeks(data.week_offset as i64)
                };
                let rule_sets = self.rule_sets.clone();
                show_view_dialog(id, &name, col, start_min, end_min, schedule_type, rule_set_id, week_monday, rule_sets, _root, sender.clone());
                self.update_view(widgets, sender);
                return;
            }
            ScheduleInput::ShowCreateDialog { col, start_min, end_min } => {
                let week_monday = {
                    let data = self.draw_data.borrow();
                    let today = chrono::Local::now().date_naive();
                    let dfm = today.weekday().num_days_from_monday() as i64;
                    let this_mon = today - chrono::Duration::days(dfm);
                    this_mon + chrono::Duration::weeks(data.week_offset as i64)
                };
                show_create_dialog(col, start_min, end_min, week_monday, self.rule_sets.clone(), _root, sender.clone());
                self.update_view(widgets, sender);
                return;
            }
            ScheduleInput::ShowEditDialog { id, name, col, start_min, end_min, schedule_type, rule_set_id } => {
                let rule_sets = self.rule_sets.clone();
                show_edit_dialog(id, &name, col, start_min, end_min, schedule_type, rule_set_id, rule_sets, _root, sender.clone());
                self.update_view(widgets, sender);
                return;
            }
            ScheduleInput::RuleSetsUpdated(sets) => {
                self.rule_sets = sets;
            }
            ScheduleInput::CommitCreate { name, col, start_min, end_min, specific_date, schedule_type, rule_set_id } => {
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
            ScheduleInput::CommitEdit { id, name, col, start_min, end_min, schedule_type, rule_set_id } => {
                let _ = sender.output(ScheduleOutput::UpdateSchedule {
                    id,
                    name,
                    days: vec![col as u8],
                    start_min,
                    end_min,
                    schedule_type,
                    rule_set_id,
                });
            }
            ScheduleInput::CommitDelete(id) => {
                let _ = sender.output(ScheduleOutput::DeleteSchedule(id));
            }
            ScheduleInput::CommitDragMove { id, col, start_min, end_min } => {
                let sched = self.draw_data.borrow().schedules.iter().find(|s| s.id == id).cloned();
                if let Some(sched) = sched {
                    let rule_set_id = if sched.rule_set_id.is_nil() { None } else { Some(sched.rule_set_id) };
                    let _ = sender.output(ScheduleOutput::UpdateSchedule {
                        id,
                        name: sched.name,
                        days: vec![col as u8],
                        start_min,
                        end_min,
                        schedule_type: sched.schedule_type,
                        rule_set_id,
                    });
                }
            }
            ScheduleInput::CommitDragResize { id, col, start_min, end_min } => {
                let sched = self.draw_data.borrow().schedules.iter().find(|s| s.id == id).cloned();
                if let Some(sched) = sched {
                    let rule_set_id = if sched.rule_set_id.is_nil() { None } else { Some(sched.rule_set_id) };
                    let _ = sender.output(ScheduleOutput::UpdateSchedule {
                        id,
                        name: sched.name,
                        days: vec![col as u8],
                        start_min,
                        end_min,
                        schedule_type: sched.schedule_type,
                        rule_set_id,
                    });
                }
            }
        }
        widgets.drawing_area.queue_draw();
        self.update_view(widgets, sender);
    }
}

// ── Drawing ───────────────────────────────────────────────────────────────────

struct Theme {
    bg: (f64, f64, f64),
    text: (f64, f64, f64),
    text_dim: (f64, f64, f64),
    text_today: (f64, f64, f64),
    grid: (f64, f64, f64),
    today_highlight: (f64, f64, f64, f64), // rgba
}

impl Theme {
    fn from_widget(da: &gtk4::DrawingArea) -> Self {
        let fg = da.style_context().color();
        // Perceived luminance of the foreground colour — high means light text → dark theme
        let lum = 0.299 * fg.red() as f64 + 0.587 * fg.green() as f64 + 0.114 * fg.blue() as f64;
        let dark = lum > 0.5;
        if dark {
            Theme {
                bg: (0.16, 0.16, 0.16),
                text: (0.90, 0.90, 0.90),
                text_dim: (0.55, 0.55, 0.55),
                text_today: (0.50, 0.78, 1.00),
                grid: (0.30, 0.30, 0.30),
                today_highlight: (0.15, 0.27, 0.45, 0.45),
            }
        } else {
            Theme {
                bg: (1.00, 1.00, 1.00),
                text: (0.15, 0.15, 0.15),
                text_dim: (0.50, 0.50, 0.50),
                text_today: (0.20, 0.45, 0.90),
                grid: (0.82, 0.82, 0.82),
                today_highlight: (0.88, 0.94, 1.00, 0.60),
            }
        }
    }
}

fn draw_calendar(
    da: &gtk4::DrawingArea,
    cr: &gtk4::cairo::Context,
    width: i32,
    height: i32,
    data: &DrawData,
) {
    let t = Theme::from_widget(da);
    let w = width as f64;
    let h = height as f64;

    const MARGIN_LEFT: f64 = 52.0;
    const HEADER_H: f64 = 40.0;
    const MARGIN_RIGHT: f64 = 4.0;

    let total_hours = (END_HOUR - START_HOUR) as f64;
    let col_w = (w - MARGIN_LEFT - MARGIN_RIGHT) / 7.0;
    let hour_h = (h - HEADER_H) / total_hours;

    let now = Local::now();
    let today = now.date_naive();
    let days_from_mon = today.weekday().num_days_from_monday() as i64;
    let this_monday = today - Duration::days(days_from_mon);
    let week_monday = this_monday + Duration::weeks(data.week_offset as i64);

    // ── Background ────────────────────────────────────────────────────────
    cr.set_source_rgb(t.bg.0, t.bg.1, t.bg.2);
    let _ = cr.paint();

    // ── Today column highlight ────────────────────────────────────────────
    let today_col = if data.week_offset == 0 {
        Some(today.weekday().num_days_from_monday() as usize)
    } else {
        None
    };

    if let Some(col) = today_col {
        let x = MARGIN_LEFT + col as f64 * col_w;
        let (r, g, b, a) = t.today_highlight;
        cr.set_source_rgba(r, g, b, a);
        cr.rectangle(x, 0.0, col_w, h);
        let _ = cr.fill();
    }

    // ── Hour grid lines + labels ──────────────────────────────────────────
    cr.select_font_face(
        "Sans",
        gtk4::cairo::FontSlant::Normal,
        gtk4::cairo::FontWeight::Normal,
    );
    cr.set_font_size(11.0);

    for h_idx in 0..=(END_HOUR - START_HOUR) {
        let hour = START_HOUR + h_idx;
        let y = HEADER_H + h_idx as f64 * hour_h;

        cr.set_source_rgb(t.grid.0, t.grid.1, t.grid.2);
        cr.set_line_width(0.5);
        cr.move_to(MARGIN_LEFT, y);
        cr.line_to(w - MARGIN_RIGHT, y);
        let _ = cr.stroke();

        let label = format!("{hour:02}:00");
        cr.set_source_rgb(t.text_dim.0, t.text_dim.1, t.text_dim.2);
        cr.move_to(2.0, y + 4.0);
        let _ = cr.show_text(&label);
    }

    // ── Vertical column separators + day headers ──────────────────────────
    const DAY_NAMES: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

    for col in 0..7usize {
        let x = MARGIN_LEFT + col as f64 * col_w;

        cr.set_source_rgb(t.grid.0, t.grid.1, t.grid.2);
        cr.set_line_width(0.5);
        cr.move_to(x, 0.0);
        cr.line_to(x, h);
        let _ = cr.stroke();

        let date = week_monday + Duration::days(col as i64);
        let header = format!("{} {}", DAY_NAMES[col], date.day());

        if Some(col) == today_col {
            cr.set_source_rgb(t.text_today.0, t.text_today.1, t.text_today.2);
            cr.select_font_face(
                "Sans",
                gtk4::cairo::FontSlant::Normal,
                gtk4::cairo::FontWeight::Bold,
            );
        } else {
            cr.set_source_rgb(t.text.0, t.text.1, t.text.2);
            cr.select_font_face(
                "Sans",
                gtk4::cairo::FontSlant::Normal,
                gtk4::cairo::FontWeight::Normal,
            );
        }
        cr.set_font_size(12.0);
        let te = cr
            .text_extents(&header)
            .unwrap_or(gtk4::cairo::TextExtents::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0));
        cr.move_to(x + (col_w - te.width()) / 2.0, HEADER_H - 10.0);
        let _ = cr.show_text(&header);
    }

    // Reset font
    cr.select_font_face(
        "Sans",
        gtk4::cairo::FontSlant::Normal,
        gtk4::cairo::FontWeight::Normal,
    );

    // ── Event blocks ──────────────────────────────────────────────────────
    for sched in &data.schedules {
        if !sched.enabled {
            continue;
        }

        // Skip blocks that are being resized — the drag preview renders them.
        let is_resizing = matches!(&data.drag_mode, DragMode::Resize { id, .. } if *id == sched.id);
        if is_resizing { continue; }

        let is_moving = matches!(&data.drag_mode, DragMode::Move { id, .. } if *id == sched.id);

        // Stable color derived from the event name so all instances of the same
        // event (e.g. each day's "Study") share the same color.
        let color_idx = sched
            .name
            .bytes()
            .fold(0usize, |acc, b| acc.wrapping_add(b as usize));
        let (r, g, b) = COLORS[color_idx % COLORS.len()];

        // Dim the original block while moving.
        let fill_alpha = if is_moving { 0.25 } else { 0.80 };

        // Determine which columns to draw in for this week.
        let cols: Vec<usize> = if let Some(date_str) = &sched.specific_date {
            // One-time event: only draw if the date falls within the displayed week.
            if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                let days_offset = (date - week_monday).num_days();
                if days_offset >= 0 && days_offset < 7 {
                    vec![days_offset as usize]
                } else {
                    vec![]
                }
            } else {
                vec![]
            }
        } else {
            // Recurring: draw on each listed weekday.
            sched.days.iter().map(|&d| d as usize).collect()
        };

        for col in cols {
            let x = MARGIN_LEFT + col as f64 * col_w + 2.0;
            let block_w = col_w - 4.0;

            let start_frac = clamp_hour_frac(sched.start_min as f64 / 60.0);
            let end_frac = clamp_hour_frac(sched.end_min as f64 / 60.0);

            let y_start = HEADER_H + start_frac * (h - HEADER_H);
            let y_end = HEADER_H + end_frac * (h - HEADER_H);
            let block_h = (y_end - y_start).max(4.0);

            // Filled rounded rect
            cr.set_source_rgba(r, g, b, fill_alpha);
            rounded_rect(cr, x, y_start, block_w, block_h, 4.0);
            let _ = cr.fill();

            // White outline for custom (non-imported) events
            if !sched.imported {
                cr.set_source_rgba(1.0, 1.0, 1.0, 0.5 * fill_alpha);
                cr.set_line_width(1.5);
                rounded_rect(cr, x, y_start, block_w, block_h, 4.0);
                let _ = cr.stroke();
            }

            if !is_moving {
                // Event name (+ calendar icon for imported events)
                if block_h > 14.0 {
                    cr.set_source_rgb(1.0, 1.0, 1.0);
                    cr.set_font_size(13.0);

                    const ICON_W: f64 = 13.0;
                    const ICON_GAP: f64 = 6.0;
                    let icon_total = if sched.imported { ICON_W + ICON_GAP } else { 0.0 };

                    let te = cr
                        .text_extents(&sched.name)
                        .unwrap_or(gtk4::cairo::TextExtents::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0));
                    let content_w = icon_total + te.width();
                    let text_x = (x + (block_w - content_w) / 2.0 + icon_total).max(x + 2.0 + icon_total);
                    let text_y = y_start + block_h / 2.0 + te.height() / 2.0;

                    if sched.imported {
                        let ix = text_x - icon_total;
                        let iy = text_y - te.height() - 1.0;
                        draw_calendar_icon(cr, ix, iy, ICON_W);
                    }

                    cr.move_to(text_x, text_y);
                    let _ = cr.show_text(&sched.name);
                }

                // Resize handles — small pills at top and bottom of draggable events
                if !sched.imported && block_h > 18.0 {
                    let handle_w = (block_w * 0.35).min(28.0);
                    let handle_h = 3.0;
                    let handle_x = x + (block_w - handle_w) / 2.0;
                    cr.set_source_rgba(1.0, 1.0, 1.0, 0.55);
                    // Bottom handle
                    let bottom_y = y_start + block_h - handle_h - 3.0;
                    rounded_rect(cr, handle_x, bottom_y, handle_w, handle_h, 1.5);
                    let _ = cr.fill();
                    // Top handle
                    let top_y = y_start + 3.0;
                    rounded_rect(cr, handle_x, top_y, handle_w, handle_h, 1.5);
                    let _ = cr.fill();
                }
            }
        }
    }

    // ── Drag preview ──────────────────────────────────────────────────────
    let preview_geom: Option<(usize, u32, u32)> = match &data.drag_mode {
        DragMode::Create { col, start_min, end_min } => Some((*col, *start_min, *end_min)),
        DragMode::Move { col, start_min, end_min, .. } => Some((*col, *start_min, *end_min)),
        DragMode::Resize { col, start_min, end_min, .. } => Some((*col, *start_min, *end_min)),
        DragMode::None => None,
    };
    if let Some((col, s_min, e_min)) = preview_geom {
        let x = MARGIN_LEFT + col as f64 * col_w + 2.0;
        let bw = col_w - 4.0;
        let sf = clamp_hour_frac(s_min as f64 / 60.0);
        let ef = clamp_hour_frac(e_min as f64 / 60.0);
        let ys = HEADER_H + sf * (h - HEADER_H);
        let ye = HEADER_H + ef * (h - HEADER_H);
        let bh = (ye - ys).max(4.0);
        let (fill_a, stroke_a) = match &data.drag_mode {
            DragMode::Create { .. } => (0.35, 0.85),
            _ => (0.55, 0.95),
        };
        cr.set_source_rgba(0.26, 0.54, 0.96, fill_a);
        rounded_rect(cr, x, ys, bw, bh, 4.0);
        let _ = cr.fill();
        cr.set_source_rgba(0.26, 0.54, 0.96, stroke_a);
        cr.set_line_width(2.0);
        rounded_rect(cr, x, ys, bw, bh, 4.0);
        let _ = cr.stroke();
    }

    // ── Current time indicator (only on current week) ─────────────────────
    if data.week_offset == 0 {
        let col = today.weekday().num_days_from_monday() as usize;
        let now_min = now.hour() * 60 + now.minute();
        let frac = clamp_hour_frac(now_min as f64 / 60.0);
        let y = HEADER_H + frac * (h - HEADER_H);
        let x = MARGIN_LEFT + col as f64 * col_w;

        cr.set_source_rgb(0.90, 0.20, 0.20);
        cr.set_line_width(2.0);
        cr.move_to(x, y);
        cr.line_to(x + col_w, y);
        let _ = cr.stroke();

        // Small circle at left edge
        cr.arc(x, y, 4.0, 0.0, std::f64::consts::TAU);
        let _ = cr.fill();
    }
}

/// Draw a tiny calendar icon (outline + header bar + two dot-rows) at (x, y)
/// fitting within a square of `size` pixels. Colour inherits the current source.
fn draw_calendar_icon(cr: &gtk4::cairo::Context, x: f64, y: f64, size: f64) {
    let s = size;
    let lw = 1.0_f64;
    cr.set_line_width(lw);

    // Outer rounded rect
    rounded_rect(cr, x, y, s, s, 1.5);
    let _ = cr.stroke();

    // Header band (top ~30 %)
    let hh = (s * 0.30).max(2.0);
    cr.rectangle(x + lw / 2.0, y + lw / 2.0, s - lw, hh);
    let _ = cr.fill();

    // Two rows of two dots in the body
    let dot_r = (s * 0.08).max(0.8);
    let col1 = x + s * 0.28;
    let col2 = x + s * 0.68;
    let row1 = y + hh + (s - hh) * 0.35;
    let row2 = y + hh + (s - hh) * 0.70;
    for &(dx, dy) in &[(col1, row1), (col2, row1), (col1, row2), (col2, row2)] {
        cr.arc(dx, dy, dot_r, 0.0, std::f64::consts::TAU);
        let _ = cr.fill();
    }
}

/// Clamp a fractional hour (e.g. 7.5 = 07:30) to the visible range,
/// returning a fraction [0, 1] within [START_HOUR, END_HOUR].
fn clamp_hour_frac(hour_frac: f64) -> f64 {
    let start = START_HOUR as f64;
    let end = END_HOUR as f64;
    ((hour_frac - start) / (end - start)).clamp(0.0, 1.0)
}

fn rounded_rect(cr: &gtk4::cairo::Context, x: f64, y: f64, w: f64, h: f64, r: f64) {
    let r = r.min(w / 2.0).min(h / 2.0);
    cr.new_sub_path();
    cr.arc(
        x + r,
        y + r,
        r,
        std::f64::consts::PI,
        3.0 * std::f64::consts::PI / 2.0,
    );
    cr.arc(x + w - r, y + r, r, 3.0 * std::f64::consts::PI / 2.0, 0.0);
    cr.arc(x + w - r, y + h - r, r, 0.0, std::f64::consts::PI / 2.0);
    cr.arc(
        x + r,
        y + h - r,
        r,
        std::f64::consts::PI / 2.0,
        std::f64::consts::PI,
    );
    cr.close_path();
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

/// Round minutes to the nearest 15-minute boundary.
fn snap15(m: u32) -> u32 {
    ((m + 7) / 15) * 15
}

fn pixel_to_day_time(x: f64, y: f64, w: f64, h: f64) -> Option<(usize, u32)> {
    const MARGIN_LEFT: f64 = 52.0;
    const HEADER_H: f64 = 40.0;
    const MARGIN_RIGHT: f64 = 4.0;
    if x < MARGIN_LEFT || y < HEADER_H { return None; }
    let col_w = (w - MARGIN_LEFT - MARGIN_RIGHT) / 7.0;
    let hour_h = (h - HEADER_H) / (END_HOUR - START_HOUR) as f64;
    let col = ((x - MARGIN_LEFT) / col_w) as usize;
    if col >= 7 { return None; }
    let hour_frac = (y - HEADER_H) / hour_h;
    let minutes = (START_HOUR as f64 * 60.0 + hour_frac * 60.0) as u32;
    Some((col, minutes.clamp(START_HOUR * 60, END_HOUR * 60)))
}

fn hit_test_event(
    x: f64, y: f64, w: f64, h: f64,
    week_offset: i32,
    schedules: &[ScheduleSummary],
) -> Option<(uuid::Uuid, String, usize, u32, u32, bool, ScheduleType, uuid::Uuid)> {
    const MARGIN_LEFT: f64 = 52.0;
    const HEADER_H: f64 = 40.0;
    const MARGIN_RIGHT: f64 = 4.0;
    let col_w = (w - MARGIN_LEFT - MARGIN_RIGHT) / 7.0;

    let today = chrono::Local::now().date_naive();
    let dfm = today.weekday().num_days_from_monday() as i64;
    let this_mon = today - chrono::Duration::days(dfm);
    let week_monday = this_mon + chrono::Duration::weeks(week_offset as i64);

    for sched in schedules {
        let cols: Vec<usize> = if let Some(ds) = &sched.specific_date {
            if let Ok(date) = chrono::NaiveDate::parse_from_str(ds, "%Y-%m-%d") {
                let off = (date - week_monday).num_days();
                if off >= 0 && off < 7 { vec![off as usize] } else { vec![] }
            } else { vec![] }
        } else {
            sched.days.iter().map(|&d| d as usize).collect()
        };

        for col in cols {
            let ex = MARGIN_LEFT + col as f64 * col_w + 2.0;
            let bw = col_w - 4.0;
            let sf = clamp_hour_frac(sched.start_min as f64 / 60.0);
            let ef = clamp_hour_frac(sched.end_min as f64 / 60.0);
            let ys = HEADER_H + sf * (h - HEADER_H);
            let ye = HEADER_H + ef * (h - HEADER_H);
            let bh = (ye - ys).max(4.0);
            if x >= ex && x <= ex + bw && y >= ys && y <= ys + bh {
                return Some((sched.id, sched.name.clone(), col, sched.start_min, sched.end_min, sched.imported, sched.schedule_type.clone(), sched.rule_set_id));
            }
        }
    }
    None
}

fn parse_hhmm(s: &str) -> Option<u32> {
    let mut parts = s.splitn(2, ':');
    let h: u32 = parts.next()?.trim().parse().ok()?;
    let m: u32 = parts.next()?.trim().parse().ok()?;
    if h > 23 || m > 59 { return None; }
    Some(h * 60 + m)
}

fn show_view_dialog(
    id: uuid::Uuid,
    name: &str,
    col: usize,
    start_min: u32,
    end_min: u32,
    schedule_type: ScheduleType,
    rule_set_id: uuid::Uuid,
    week_monday: chrono::NaiveDate,
    rule_sets: Vec<RuleSetSummary>,
    root: &gtk4::Box,
    sender: ComponentSender<ScheduleSection>,
) {
    let dialog = gtk4::Window::builder()
        .title("Calendar Event")
        .modal(true)
        .default_width(340)
        .resizable(false)
        .build();
    if let Some(top) = root.root().and_then(|r| r.downcast::<gtk4::Window>().ok()) {
        dialog.set_transient_for(Some(&top));
    }

    let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
    vbox.set_margin_all(16);

    // Badge
    let badge = gtk4::Label::new(Some("Imported from calendar — name and time are read-only"));
    badge.add_css_class("caption");
    badge.set_halign(gtk4::Align::Start);
    badge.set_opacity(0.6);
    badge.set_wrap(true);
    vbox.append(&badge);

    // Read-only name
    let name_lbl = gtk4::Label::new(Some(name));
    name_lbl.add_css_class("title-3");
    name_lbl.set_halign(gtk4::Align::Start);
    name_lbl.set_wrap(true);
    vbox.append(&name_lbl);

    // Read-only date + time
    let date = week_monday + chrono::Duration::days(col as i64);
    let meta_lbl = gtk4::Label::new(Some(&format!(
        "{}   {:02}:{:02} – {:02}:{:02}",
        date.format("%A, %B %-d"),
        start_min / 60, start_min % 60,
        end_min / 60,   end_min % 60,
    )));
    meta_lbl.set_halign(gtk4::Align::Start);
    meta_lbl.set_opacity(0.65);
    meta_lbl.set_margin_bottom(4);
    vbox.append(&meta_lbl);

    // Editable: focus/break + allowed list
    let (focus_btn, _break_btn, list_combo) =
        build_type_and_list_rows(&vbox, &schedule_type, rule_set_id, &rule_sets);

    // Buttons
    let btn_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    btn_row.set_halign(gtk4::Align::End);
    btn_row.set_margin_top(8);
    let cancel_btn = gtk4::Button::with_label("Cancel");
    let save_btn = gtk4::Button::with_label("Save");
    save_btn.add_css_class("suggested-action");
    btn_row.append(&cancel_btn);
    btn_row.append(&save_btn);
    vbox.append(&btn_row);

    dialog.set_child(Some(&vbox));

    let d = dialog.clone();
    cancel_btn.connect_clicked(move |_| d.close());

    let d = dialog.clone();
    let day = col as u8;
    let name_owned = name.to_string();
    save_btn.connect_clicked(move |_| {
        let stype = if focus_btn.is_active() { ScheduleType::Focus } else { ScheduleType::Break };
        let new_rule_set_id = resolve_rule_set(&list_combo, &rule_sets);
        sender.input(ScheduleInput::CommitEdit {
            id,
            name: name_owned.clone(),
            col: day as usize,
            start_min,
            end_min,
            schedule_type: stype,
            rule_set_id: new_rule_set_id,
        });
        d.close();
    });

    dialog.present();
}

fn build_type_and_list_rows(
    vbox: &gtk4::Box,
    initial_type: &ScheduleType,
    initial_rule_set_id: uuid::Uuid,
    rule_sets: &[RuleSetSummary],
) -> (gtk4::ToggleButton, gtk4::ToggleButton, gtk4::ComboBoxText) {
    // ── Type row ──────────────────────────────────────────────────────────
    let type_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    let type_lbl = gtk4::Label::new(Some("Type:"));
    type_lbl.set_width_chars(8);
    type_lbl.set_halign(gtk4::Align::Start);
    let focus_btn = gtk4::ToggleButton::with_label("Focus");
    let break_btn = gtk4::ToggleButton::with_label("Break");
    break_btn.set_group(Some(&focus_btn));
    focus_btn.set_active(*initial_type == ScheduleType::Focus);
    break_btn.set_active(*initial_type == ScheduleType::Break);
    type_row.append(&type_lbl);
    type_row.append(&focus_btn);
    type_row.append(&break_btn);
    vbox.append(&type_row);

    // ── Allowed list row ──────────────────────────────────────────────────
    let list_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    let list_lbl = gtk4::Label::new(Some("Allowed list:"));
    list_lbl.set_width_chars(8);
    list_lbl.set_halign(gtk4::Align::Start);
    let list_combo = gtk4::ComboBoxText::new();
    list_combo.append_text("(none)");
    for rs in rule_sets {
        list_combo.append_text(&rs.name);
    }
    let sel_idx = rule_sets.iter().position(|r| r.id == initial_rule_set_id)
        .map(|i| i + 1)
        .unwrap_or(0);
    list_combo.set_active(Some(sel_idx as u32));
    list_combo.set_hexpand(true);
    list_row.append(&list_lbl);
    list_row.append(&list_combo);
    list_row.set_visible(*initial_type == ScheduleType::Focus);
    vbox.append(&list_row);

    // Hide/show list row when type changes
    {
        let list_row = list_row.clone();
        let bb = break_btn.clone();
        focus_btn.connect_toggled(move |fb| {
            list_row.set_visible(fb.is_active());
            let _ = bb.is_active(); // suppress unused warning
        });
    }

    (focus_btn, break_btn, list_combo)
}

fn resolve_rule_set(combo: &gtk4::ComboBoxText, rule_sets: &[RuleSetSummary]) -> Option<uuid::Uuid> {
    let idx = combo.active().unwrap_or(0) as usize;
    if idx == 0 { None } else { rule_sets.get(idx - 1).map(|r| r.id) }
}

fn show_create_dialog(
    col: usize,
    start_min: u32,
    end_min: u32,
    week_monday: chrono::NaiveDate,
    rule_sets: Vec<RuleSetSummary>,
    root: &gtk4::Box,
    sender: ComponentSender<ScheduleSection>,
) {
    let dialog = gtk4::Window::builder()
        .title("New Event")
        .modal(true)
        .default_width(340)
        .resizable(false)
        .build();
    if let Some(top) = root.root().and_then(|r| r.downcast::<gtk4::Window>().ok()) {
        dialog.set_transient_for(Some(&top));
    }

    let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
    vbox.set_margin_all(16);

    let date = week_monday + chrono::Duration::days(col as i64);
    let day_lbl = gtk4::Label::new(Some(&date.format("%A, %B %-d").to_string()));
    day_lbl.add_css_class("title-3");
    day_lbl.set_halign(gtk4::Align::Start);
    vbox.append(&day_lbl);

    let name_entry = gtk4::Entry::new();
    name_entry.set_text("Focus Session");
    name_entry.set_margin_top(4);
    vbox.append(&name_entry);

    let time_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    let start_entry = gtk4::Entry::new();
    start_entry.set_text(&format!("{:02}:{:02}", start_min / 60, start_min % 60));
    start_entry.set_width_chars(6);
    let sep_lbl = gtk4::Label::new(Some("–"));
    let end_entry = gtk4::Entry::new();
    end_entry.set_text(&format!("{:02}:{:02}", end_min / 60, end_min % 60));
    end_entry.set_width_chars(6);
    time_row.append(&start_entry);
    time_row.append(&sep_lbl);
    time_row.append(&end_entry);
    vbox.append(&time_row);

    let default_rule_set_id = rule_sets.first().map(|r| r.id).unwrap_or_else(uuid::Uuid::nil);
    let (focus_btn, break_btn, list_combo) = build_type_and_list_rows(
        &vbox, &ScheduleType::Focus, default_rule_set_id, &rule_sets,
    );

    // Update the name when the type toggle changes, but only if the user
    // hasn't edited it away from the default.
    let ne = name_entry.clone();
    focus_btn.connect_toggled(move |btn| {
        if btn.is_active() {
            let current = ne.text();
            if current == "Break Session" || current.is_empty() {
                ne.set_text("Focus Session");
            }
        }
    });
    let ne = name_entry.clone();
    break_btn.connect_toggled(move |btn| {
        if btn.is_active() {
            let current = ne.text();
            if current == "Focus Session" || current.is_empty() {
                ne.set_text("Break Session");
            }
        }
    });

    let btn_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    btn_row.set_halign(gtk4::Align::End);
    btn_row.set_margin_top(8);
    let cancel_btn = gtk4::Button::with_label("Cancel");
    let save_btn = gtk4::Button::with_label("Save");
    save_btn.add_css_class("suggested-action");
    btn_row.append(&cancel_btn);
    btn_row.append(&save_btn);
    vbox.append(&btn_row);

    dialog.set_child(Some(&vbox));

    let d = dialog.clone();
    cancel_btn.connect_clicked(move |_| d.close());

    let d = dialog.clone();
    let ne = name_entry.clone();
    let se = start_entry.clone();
    let ee = end_entry.clone();
    let day = col as u8;
    let date_str = date.format("%Y-%m-%d").to_string();
    save_btn.connect_clicked(move |_| {
        let name = ne.text().to_string();
        if name.is_empty() { return; }
        let Some(s_min) = parse_hhmm(&se.text()) else { return };
        let Some(e_min) = parse_hhmm(&ee.text()) else { return };
        if e_min <= s_min { return; }
        let stype = if focus_btn.is_active() { ScheduleType::Focus } else { ScheduleType::Break };
        let rule_set_id = resolve_rule_set(&list_combo, &rule_sets);
        sender.input(ScheduleInput::CommitCreate {
            name,
            col: day as usize,
            start_min: s_min,
            end_min: e_min,
            specific_date: date_str.clone(),
            schedule_type: stype,
            rule_set_id,
        });
        d.close();
    });

    dialog.present();
}

fn show_edit_dialog(
    id: uuid::Uuid,
    name: &str,
    col: usize,
    start_min: u32,
    end_min: u32,
    schedule_type: ScheduleType,
    rule_set_id: uuid::Uuid,
    rule_sets: Vec<RuleSetSummary>,
    root: &gtk4::Box,
    sender: ComponentSender<ScheduleSection>,
) {
    let dialog = gtk4::Window::builder()
        .title("Edit Event")
        .modal(true)
        .default_width(340)
        .resizable(false)
        .build();
    if let Some(top) = root.root().and_then(|r| r.downcast::<gtk4::Window>().ok()) {
        dialog.set_transient_for(Some(&top));
    }

    let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
    vbox.set_margin_all(16);

    let name_entry = gtk4::Entry::new();
    name_entry.set_text(name);
    name_entry.set_placeholder_text(Some("Event name"));
    vbox.append(&name_entry);

    let time_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    let start_entry = gtk4::Entry::new();
    start_entry.set_text(&format!("{:02}:{:02}", start_min / 60, start_min % 60));
    start_entry.set_width_chars(6);
    let sep_lbl = gtk4::Label::new(Some("–"));
    let end_entry = gtk4::Entry::new();
    end_entry.set_text(&format!("{:02}:{:02}", end_min / 60, end_min % 60));
    end_entry.set_width_chars(6);
    time_row.append(&start_entry);
    time_row.append(&sep_lbl);
    time_row.append(&end_entry);
    vbox.append(&time_row);

    let (focus_btn, _break_btn, list_combo) = build_type_and_list_rows(
        &vbox, &schedule_type, rule_set_id, &rule_sets,
    );

    let btn_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    btn_row.set_hexpand(true);
    let del_btn = gtk4::Button::with_label("Delete");
    del_btn.add_css_class("destructive-action");
    let spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    let cancel_btn = gtk4::Button::with_label("Cancel");
    let save_btn = gtk4::Button::with_label("Save");
    save_btn.add_css_class("suggested-action");
    btn_row.append(&del_btn);
    btn_row.append(&spacer);
    btn_row.append(&cancel_btn);
    btn_row.append(&save_btn);
    vbox.append(&btn_row);

    dialog.set_child(Some(&vbox));

    let d = dialog.clone();
    cancel_btn.connect_clicked(move |_| d.close());

    {
        let d = dialog.clone();
        let s = sender.clone();
        del_btn.connect_clicked(move |_| {
            s.input(ScheduleInput::CommitDelete(id));
            d.close();
        });
    }

    let d = dialog.clone();
    let ne = name_entry.clone();
    let se = start_entry.clone();
    let ee = end_entry.clone();
    let day = col as u8;
    save_btn.connect_clicked(move |_| {
        let name = ne.text().to_string();
        if name.is_empty() { return; }
        let Some(s_min) = parse_hhmm(&se.text()) else { return };
        let Some(e_min) = parse_hhmm(&ee.text()) else { return };
        if e_min <= s_min { return; }
        let stype = if focus_btn.is_active() { ScheduleType::Focus } else { ScheduleType::Break };
        let rule_set_id = resolve_rule_set(&list_combo, &rule_sets);
        sender.input(ScheduleInput::CommitEdit {
            id,
            name,
            col: day as usize,
            start_min: s_min,
            end_min: e_min,
            schedule_type: stype,
            rule_set_id,
        });
        d.close();
    });

    dialog.present();
}
