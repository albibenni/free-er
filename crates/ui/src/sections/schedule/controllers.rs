use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use relm4::prelude::*;

use super::draw_data::{DragMode, DrawData};
use super::geometry::{
    clamp_hour_frac, hit_test_event, pixel_to_day_time, snap15, END_HOUR, HEADER_H, MARGIN_LEFT,
    MARGIN_RIGHT, START_HOUR,
};
use super::week::week_monday_for_offset;
use super::{ScheduleInput, ScheduleSection};

fn create_drag_range(
    sx: f64,
    sy: f64,
    cx: f64,
    cy: f64,
    w: f64,
    h: f64,
) -> Option<(usize, u32, u32)> {
    let (col, s_min) = pixel_to_day_time(sx, sy, w, h)?;
    let (_, e_min_raw) = pixel_to_day_time(cx, cy, w, h)?;
    let (s, e) = if e_min_raw >= s_min {
        (s_min, e_min_raw.max(s_min + 15))
    } else {
        (e_min_raw, s_min)
    };
    Some((col, snap15(s), snap15(e)))
}

fn move_drag_target(
    cx: f64,
    cy: f64,
    w: f64,
    h: f64,
    duration_min: u32,
    click_offset_min: i32,
) -> (usize, u32, u32) {
    let hour_h = (h - HEADER_H) / (END_HOUR - START_HOUR) as f64;
    let col_w = (w - MARGIN_LEFT - MARGIN_RIGHT) / 7.0;
    let new_col = if cx >= MARGIN_LEFT {
        (((cx - MARGIN_LEFT) / col_w) as usize).min(6)
    } else {
        0
    };
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
    (new_col, new_start, new_end)
}

fn resize_drag_target(
    sx: f64,
    cy: f64,
    w: f64,
    h: f64,
    start_min: u32,
    end_min: u32,
    from_top: bool,
) -> Option<(u32, u32)> {
    let (_, raw_min) = pixel_to_day_time(sx, cy, w, h)?;
    let snapped = snap15(raw_min);
    let (new_start, new_end) = if from_top {
        let s = snapped.min(end_min.saturating_sub(15)).max(START_HOUR * 60);
        (s, end_min)
    } else {
        let e = snapped.max(start_min + 15).min(END_HOUR * 60);
        (start_min, e)
    };
    Some((new_start, new_end))
}

fn begin_drag_mode(data: &DrawData, x: f64, y: f64, w: f64, h: f64) -> DragMode {
    let week_offset = data.week_offset;
    let hit = hit_test_event(x, y, w, h, week_offset, &data.schedules);
    if let Some((id, _name, _days, col, start_min, end_min, imported, _repeating, _stype, _rs)) =
        hit
    {
        if imported {
            return DragMode::None;
        }

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
            let hour_h = (h - HEADER_H) / (END_HOUR - START_HOUR) as f64;
            let click_offset_min = ((y - y_start) / hour_h * 60.0) as i32;
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
    } else {
        DragMode::Create {
            col: 0,
            start_min: 0,
            end_min: 0,
        }
    }
}

fn apply_drag_update(data: &mut DrawData, off_x: f64, off_y: f64, w: f64, h: f64) {
    match data.drag_mode.clone() {
        DragMode::Create { .. } => {
            if let Some((sx, sy)) = data.drag_start {
                let cx = sx + off_x;
                let cy = sy + off_y;
                if let Some((col, s, e)) = create_drag_range(sx, sy, cx, cy, w, h) {
                    data.drag_mode = DragMode::Create {
                        col,
                        start_min: s,
                        end_min: e,
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
                let (new_col, new_start, new_end) =
                    move_drag_target(cx, cy, w, h, duration_min, click_offset_min);
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
                if let Some((new_start, new_end)) =
                    resize_drag_target(sx, cy, w, h, start_min, end_min, from_top)
                {
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
}

fn apply_drag_end(
    data: &mut DrawData,
    off_x: f64,
    off_y: f64,
    w: f64,
    h: f64,
) -> Vec<ScheduleInput> {
    let dist = (off_x * off_x + off_y * off_y).sqrt();
    let mode = std::mem::replace(&mut data.drag_mode, DragMode::None);
    let start_pos = data.drag_start.take();

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
                if let Some(sched) = data.schedules.iter_mut().find(|s| s.id == *id) {
                    if sched.specific_date.is_some() {
                        let new_date = week_monday_for_offset(week_offset)
                            + chrono::Duration::days(*col as i64);
                        let date_str = new_date.format("%Y-%m-%d").to_string();
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
                if let Some(sched) = data.schedules.iter_mut().find(|s| s.id == *id) {
                    sched.days = vec![*col as u8];
                    sched.start_min = *start_min;
                    sched.end_min = *end_min;
                }
            }
            _ => {}
        }
    }

    if dist <= 10.0 {
        if let Some((x, y)) = start_pos {
            return vec![ScheduleInput::ClickAt(x, y, w, h)];
        }
        return Vec::new();
    }

    match mode {
        DragMode::Create {
            col,
            start_min,
            end_min,
        } => {
            if end_min > start_min + 14 {
                vec![ScheduleInput::ShowCreateDialog {
                    col,
                    start_min,
                    end_min,
                }]
            } else {
                Vec::new()
            }
        }
        DragMode::Move {
            id,
            col,
            start_min,
            end_min,
            ..
        } => vec![ScheduleInput::CommitDragMove {
            id,
            col,
            start_min,
            end_min,
            specific_date: new_specific_date,
        }],
        DragMode::Resize {
            id,
            col,
            start_min,
            end_min,
            ..
        } => vec![ScheduleInput::CommitDragResize {
            id,
            col,
            start_min,
            end_min,
        }],
        DragMode::None => Vec::new(),
    }
}

fn cursor_name_for_position(data: &DrawData, x: f64, y: f64, w: f64, h: f64) -> &'static str {
    let col_w = (w - MARGIN_LEFT - MARGIN_RIGHT) / 7.0;

    'cursor: {
        for sched in &data.schedules {
            if !sched.enabled || sched.imported {
                continue;
            }

            let cols: Vec<usize> = if let Some(ds) = &sched.specific_date {
                if let Ok(date) = chrono::NaiveDate::parse_from_str(ds, "%Y-%m-%d") {
                    let week_mon = week_monday_for_offset(data.week_offset);
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
    }
}

pub(super) fn install_controllers(
    drawing_area: &gtk4::DrawingArea,
    draw_data: Rc<RefCell<DrawData>>,
    sender: ComponentSender<ScheduleSection>,
) {
    install_drag_controller(drawing_area, draw_data.clone(), sender);
    install_motion_controller(drawing_area, draw_data);
}

fn install_drag_controller(
    drawing_area: &gtk4::DrawingArea,
    draw_data: Rc<RefCell<DrawData>>,
    sender: ComponentSender<ScheduleSection>,
) {
    let drag = gtk4::GestureDrag::new();

    {
        let dd = draw_data.clone();
        let da = drawing_area.clone();
        drag.connect_drag_begin(move |_, x, y| {
            let mut data = dd.borrow_mut();
            data.drag_start = Some((x, y));
            let w = da.width() as f64;
            let h = da.allocated_height() as f64;
            data.drag_mode = begin_drag_mode(&data, x, y, w, h);
        });
    }

    {
        let dd = draw_data.clone();
        let da = drawing_area.clone();
        drag.connect_drag_update(move |_, off_x, off_y| {
            let mut data = dd.borrow_mut();
            let w = da.width() as f64;
            let h = da.allocated_height() as f64;
            apply_drag_update(&mut data, off_x, off_y, w, h);
            drop(data);
            da.queue_draw();
        });
    }

    {
        let dd = draw_data;
        let da = drawing_area.clone();
        let s = sender.clone();
        drag.connect_drag_end(move |_, off_x, off_y| {
            let mut data = dd.borrow_mut();
            let w = da.width() as f64;
            let h = da.allocated_height() as f64;
            let inputs = apply_drag_end(&mut data, off_x, off_y, w, h);
            drop(data);
            da.queue_draw();
            for input in inputs {
                s.input(input);
            }
        });
    }

    drawing_area.add_controller(drag);
}

#[cfg(test)]
#[path = "controllers_tests.rs"]
mod tests;

fn install_motion_controller(drawing_area: &gtk4::DrawingArea, draw_data: Rc<RefCell<DrawData>>) {
    let motion = gtk4::EventControllerMotion::new();
    {
        let dd = draw_data;
        let da = drawing_area.clone();
        motion.connect_motion(move |_, x, y| {
            let data = dd.borrow();
            let w = da.width() as f64;
            let h = da.allocated_height() as f64;
            let cursor = cursor_name_for_position(&data, x, y, w, h);
            da.set_cursor_from_name(Some(cursor));
        });
    }
    drawing_area.add_controller(motion);
}
