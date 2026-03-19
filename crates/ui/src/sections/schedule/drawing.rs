use chrono::{Datelike, Duration, Local, Timelike};
use gtk4::prelude::*;

use super::draw_data::{DragMode, DrawData};
use super::geometry::{
    clamp_hour_frac, compute_layout, END_HOUR, HEADER_H, MARGIN_LEFT, MARGIN_RIGHT, START_HOUR,
};

// ── Color palette ─────────────────────────────────────────────────────────────

pub(super) const COLORS: &[(f64, f64, f64)] = &[
    (0.26, 0.54, 0.96), // blue
    (0.18, 0.69, 0.51), // teal
    (0.93, 0.42, 0.22), // orange
    (0.62, 0.32, 0.82), // purple
    (0.24, 0.71, 0.29), // green
    (0.95, 0.26, 0.45), // pink
];

// ── Theme ─────────────────────────────────────────────────────────────────────

pub(super) struct Theme {
    pub bg: (f64, f64, f64),
    pub text: (f64, f64, f64),
    pub text_dim: (f64, f64, f64),
    pub text_today: (f64, f64, f64),
    pub grid: (f64, f64, f64),
    pub today_highlight: (f64, f64, f64, f64),
}

impl Theme {
    pub fn from_widget(da: &gtk4::DrawingArea) -> Self {
        let fg = da.style_context().color();
        let lum = 0.299 * fg.red() as f64 + 0.587 * fg.green() as f64 + 0.114 * fg.blue() as f64;
        if use_dark_theme(lum) {
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

fn use_dark_theme(luma: f64) -> bool {
    luma > 0.5
}

// ── Main draw entry point ─────────────────────────────────────────────────────

pub(super) fn draw_calendar(
    da: &gtk4::DrawingArea,
    cr: &gtk4::cairo::Context,
    width: i32,
    height: i32,
    data: &DrawData,
) {
    let t = Theme::from_widget(da);
    let w = width as f64;
    let h = height as f64;

    let col_w = (w - MARGIN_LEFT - MARGIN_RIGHT) / 7.0;
    let hour_h = (h - HEADER_H) / (END_HOUR - START_HOUR) as f64;

    let now = Local::now();
    let today = now.date_naive();
    let days_from_mon = today.weekday().num_days_from_monday() as i64;
    let this_monday = today - Duration::days(days_from_mon);
    let week_monday = this_monday + Duration::weeks(data.week_offset as i64);

    draw_background(cr, &t);
    let today_col = draw_today_highlight(cr, &t, w, h, col_w, data.week_offset, today);
    draw_hour_grid(cr, &t, w, h, hour_h);
    draw_day_headers(cr, &t, col_w, h, week_monday, today_col);
    draw_event_blocks(cr, h, col_w, data, week_monday);
    draw_drag_preview(cr, h, col_w, data);
    draw_now_indicator(cr, h, col_w, data.week_offset, today, now);
}

// ── Background ────────────────────────────────────────────────────────────────

fn draw_background(cr: &gtk4::cairo::Context, t: &Theme) {
    cr.set_source_rgb(t.bg.0, t.bg.1, t.bg.2);
    let _ = cr.paint();
}

// ── Today column highlight ────────────────────────────────────────────────────

fn draw_today_highlight(
    cr: &gtk4::cairo::Context,
    t: &Theme,
    _w: f64,
    h: f64,
    col_w: f64,
    week_offset: i32,
    today: chrono::NaiveDate,
) -> Option<usize> {
    let col = today_col_for_week(week_offset, today)?;
    let x = MARGIN_LEFT + col as f64 * col_w;
    let (r, g, b, a) = t.today_highlight;
    cr.set_source_rgba(r, g, b, a);
    cr.rectangle(x, 0.0, col_w, h);
    let _ = cr.fill();
    Some(col)
}

fn today_col_for_week(week_offset: i32, today: chrono::NaiveDate) -> Option<usize> {
    (week_offset == 0).then_some(today.weekday().num_days_from_monday() as usize)
}

// ── Hour grid + labels ────────────────────────────────────────────────────────

fn draw_hour_grid(cr: &gtk4::cairo::Context, t: &Theme, w: f64, _h: f64, hour_h: f64) {
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
}

// ── Column separators + day headers ──────────────────────────────────────────

fn draw_day_headers(
    cr: &gtk4::cairo::Context,
    t: &Theme,
    col_w: f64,
    h: f64,
    week_monday: chrono::NaiveDate,
    today_col: Option<usize>,
) {
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

    // Reset to normal weight after bold headers
    cr.select_font_face(
        "Sans",
        gtk4::cairo::FontSlant::Normal,
        gtk4::cairo::FontWeight::Normal,
    );
}

// ── Event blocks ──────────────────────────────────────────────────────────────

fn draw_event_blocks(
    cr: &gtk4::cairo::Context,
    h: f64,
    col_w: f64,
    data: &DrawData,
    week_monday: chrono::NaiveDate,
) {
    let layouts = compute_layout(&data.schedules, week_monday);

    for layout in &layouts {
        if layout.hidden {
            continue;
        }
        let sched = match data.schedules.iter().find(|s| s.id == layout.sched_id) {
            Some(s) => s,
            None => continue,
        };
        if !sched.enabled {
            continue;
        }

        let is_resizing = matches!(&data.drag_mode, DragMode::Resize { id, .. } if *id == sched.id);
        if is_resizing {
            continue;
        }
        let is_moving = matches!(&data.drag_mode, DragMode::Move { id, .. } if *id == sched.id);

        let color_idx = event_color_index(&sched.name);
        let (r, g, b) = COLORS[color_idx % COLORS.len()];
        let fill_alpha = if is_moving { 0.25 } else { 0.80 };

        let slot_w = col_w / layout.total_slots as f64;
        let x = MARGIN_LEFT + layout.col as f64 * col_w + layout.slot as f64 * slot_w + 2.0;
        let block_w = slot_w - 4.0;

        let start_frac = clamp_hour_frac(sched.start_min as f64 / 60.0);
        let end_frac = clamp_hour_frac(sched.end_min as f64 / 60.0);
        let y_start = HEADER_H + start_frac * (h - HEADER_H);
        let y_end = HEADER_H + end_frac * (h - HEADER_H);
        let block_h = (y_end - y_start).max(4.0);

        cr.set_source_rgba(r, g, b, fill_alpha);
        rounded_rect(cr, x, y_start, block_w, block_h, 4.0);
        let _ = cr.fill();

        if !sched.imported {
            cr.set_source_rgba(1.0, 1.0, 1.0, 0.5 * fill_alpha);
            cr.set_line_width(1.5);
            rounded_rect(cr, x, y_start, block_w, block_h, 4.0);
            let _ = cr.stroke();
        }

        if !is_moving {
            draw_event_label(cr, sched, x, block_w, y_start, block_h);
        }
    }
}

fn draw_event_label(
    cr: &gtk4::cairo::Context,
    sched: &shared::ipc::ScheduleSummary,
    x: f64,
    block_w: f64,
    y_start: f64,
    block_h: f64,
) {
    if block_h <= 14.0 {
        return;
    }

    const PAD_L: f64 = 6.0;
    const PAD_R: f64 = 6.0;

    cr.save().unwrap();
    cr.rectangle(x, y_start, block_w - PAD_R, block_h);
    let _ = cr.clip();

    const ICON_W: f64 = 13.0;
    const ICON_GAP: f64 = 6.0;
    let icon_total = if sched.imported {
        ICON_W + ICON_GAP
    } else {
        0.0
    };

    // Show start/end times when there's enough room
    let show_times = block_h > 36.0;

    if show_times {
        let start_label = format!("{:02}:{:02}", sched.start_min / 60, sched.start_min % 60);
        let end_label = format!("{:02}:{:02}", sched.end_min / 60, sched.end_min % 60);

        cr.set_font_size(10.0);
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.75);

        cr.move_to(x + PAD_L, y_start + 10.0);
        let _ = cr.show_text(&start_label);

        cr.move_to(x + PAD_L, y_start + block_h - 4.0);
        let _ = cr.show_text(&end_label);
    }

    // Name — centered within the padded area
    cr.set_font_size(13.0);
    cr.set_source_rgb(1.0, 1.0, 1.0);
    let te = cr
        .text_extents(&sched.name)
        .unwrap_or(gtk4::cairo::TextExtents::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0));
    let inner_w = block_w - PAD_L - PAD_R;
    let content_w = icon_total + te.width();
    let text_x = (x + PAD_L + (inner_w - content_w) / 2.0 + icon_total).max(x + PAD_L + icon_total);
    let text_y = y_start + block_h / 2.0 + te.height() / 2.0;

    if sched.imported {
        draw_calendar_icon(cr, text_x - icon_total, text_y - te.height() - 1.0, ICON_W);
    }

    cr.move_to(text_x, text_y);
    let _ = cr.show_text(&sched.name);

    cr.restore().unwrap();
}

// ── Drag preview ──────────────────────────────────────────────────────────────

fn draw_drag_preview(cr: &gtk4::cairo::Context, h: f64, col_w: f64, data: &DrawData) {
    let preview: Option<(usize, u32, u32)> = match &data.drag_mode {
        DragMode::Create {
            col,
            start_min,
            end_min,
        } => Some((*col, *start_min, *end_min)),
        DragMode::Move {
            col,
            start_min,
            end_min,
            ..
        } => Some((*col, *start_min, *end_min)),
        DragMode::Resize {
            col,
            start_min,
            end_min,
            ..
        } => Some((*col, *start_min, *end_min)),
        DragMode::None => None,
    };
    let Some((col, s_min, e_min)) = preview else {
        return;
    };

    let x = MARGIN_LEFT + col as f64 * col_w + 2.0;
    let bw = col_w - 4.0;
    let ys = HEADER_H + clamp_hour_frac(s_min as f64 / 60.0) * (h - HEADER_H);
    let ye = HEADER_H + clamp_hour_frac(e_min as f64 / 60.0) * (h - HEADER_H);
    let bh = (ye - ys).max(4.0);

    let (fill_a, stroke_a) = drag_preview_alphas(&data.drag_mode);

    cr.set_source_rgba(0.26, 0.54, 0.96, fill_a);
    rounded_rect(cr, x, ys, bw, bh, 4.0);
    let _ = cr.fill();
    cr.set_source_rgba(0.26, 0.54, 0.96, stroke_a);
    cr.set_line_width(2.0);
    rounded_rect(cr, x, ys, bw, bh, 4.0);
    let _ = cr.stroke();

    // Draw start / end time labels inside the preview block
    if bh > 20.0 {
        cr.select_font_face(
            "Sans",
            gtk4::cairo::FontSlant::Normal,
            gtk4::cairo::FontWeight::Normal,
        );
        cr.set_font_size(10.0);
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.9);

        let start_label = format!("{:02}:{:02}", s_min / 60, s_min % 60);
        cr.move_to(x + 4.0, ys + 10.0);
        let _ = cr.show_text(&start_label);

        if bh > 28.0 {
            let end_label = format!("{:02}:{:02}", e_min / 60, e_min % 60);
            cr.move_to(x + 4.0, ys + bh - 4.0);
            let _ = cr.show_text(&end_label);
        }
    }
}

fn drag_preview_alphas(mode: &DragMode) -> (f64, f64) {
    match mode {
        DragMode::Create { .. } => (0.35, 0.85),
        _ => (0.55, 0.95),
    }
}

fn event_color_index(name: &str) -> usize {
    name.bytes()
        .fold(0usize, |acc, b| acc.wrapping_add(b as usize))
}

// ── Current-time indicator ────────────────────────────────────────────────────

fn draw_now_indicator(
    cr: &gtk4::cairo::Context,
    h: f64,
    col_w: f64,
    week_offset: i32,
    today: chrono::NaiveDate,
    now: chrono::DateTime<Local>,
) {
    if week_offset != 0 {
        return;
    }
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

    cr.arc(x, y, 4.0, 0.0, std::f64::consts::TAU);
    let _ = cr.fill();
}

// ── Cairo primitives ──────────────────────────────────────────────────────────

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

/// Draw a tiny calendar icon (outline + header bar + dot grid) at (x, y)
/// fitting within a square of `size` pixels.
fn draw_calendar_icon(cr: &gtk4::cairo::Context, x: f64, y: f64, size: f64) {
    let s = size;
    let lw = 1.0_f64;
    cr.set_line_width(lw);

    rounded_rect(cr, x, y, s, s, 1.5);
    let _ = cr.stroke();

    let hh = (s * 0.30).max(2.0);
    cr.rectangle(x + lw / 2.0, y + lw / 2.0, s - lw, hh);
    let _ = cr.fill();

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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use shared::ipc::{ScheduleSummary, ScheduleType};
    use uuid::Uuid;

    fn ensure_gtk() -> Option<std::sync::MutexGuard<'static, ()>> {
        let guard = crate::sections::test_support::GTK_TEST_LOCK.lock().unwrap();
        if gtk4::init().is_ok() {
            Some(guard)
        } else {
            None
        }
    }

    #[test]
    fn dark_theme_threshold() {
        assert!(!use_dark_theme(0.2));
        assert!(!use_dark_theme(0.5));
        assert!(use_dark_theme(0.51));
    }

    #[test]
    fn today_col_only_current_week() {
        let d = NaiveDate::from_ymd_opt(2026, 3, 19).unwrap(); // Thu
        assert_eq!(today_col_for_week(0, d), Some(3));
        assert_eq!(today_col_for_week(1, d), None);
        assert_eq!(today_col_for_week(-1, d), None);
    }

    #[test]
    fn drag_preview_alpha_depends_on_mode() {
        assert_eq!(
            drag_preview_alphas(&DragMode::Create {
                col: 0,
                start_min: 10,
                end_min: 20
            }),
            (0.35, 0.85)
        );
        assert_eq!(drag_preview_alphas(&DragMode::None), (0.55, 0.95));
        assert_eq!(
            drag_preview_alphas(&DragMode::Move {
                id: Uuid::new_v4(),
                col: 0,
                start_min: 10,
                end_min: 20,
                duration_min: 10,
                click_offset_min: 1
            }),
            (0.55, 0.95)
        );
    }

    #[test]
    fn event_color_index_is_stable() {
        assert_eq!(event_color_index("abc"), event_color_index("abc"));
        assert_ne!(event_color_index("abc"), event_color_index("abd"));
        assert_eq!(event_color_index(""), 0);
    }

    #[test]
    #[ignore = "requires GTK runtime stability"]
    fn draw_calendar_smoke_runs_for_drag_modes() {
        let Some(_gtk_guard) = ensure_gtk() else { return; };
        let da = gtk4::DrawingArea::new();
        let surface = gtk4::cairo::ImageSurface::create(gtk4::cairo::Format::ARgb32, 800, 900)
            .unwrap();
        let cr = gtk4::cairo::Context::new(&surface).unwrap();

        let sched = ScheduleSummary {
            id: Uuid::new_v4(),
            name: "Study".into(),
            days: vec![0],
            start_min: 9 * 60,
            end_min: 10 * 60,
            enabled: true,
            imported: false,
            imported_repeating: false,
            specific_date: None,
            schedule_type: ScheduleType::Focus,
            rule_set_id: Uuid::new_v4(),
        };

        let mut data = DrawData {
            schedules: vec![sched.clone()],
            week_offset: 0,
            drag_start: None,
            drag_mode: DragMode::None,
        };
        draw_calendar(&da, &cr, 800, 900, &data);

        data.drag_mode = DragMode::Create {
            col: 0,
            start_min: 9 * 60,
            end_min: 10 * 60,
        };
        draw_calendar(&da, &cr, 800, 900, &data);

        data.drag_mode = DragMode::Move {
            id: sched.id,
            col: 1,
            start_min: 10 * 60,
            end_min: 11 * 60,
            duration_min: 60,
            click_offset_min: 10,
        };
        draw_calendar(&da, &cr, 800, 900, &data);

        data.drag_mode = DragMode::Resize {
            id: sched.id,
            col: 1,
            start_min: 10 * 60,
            end_min: 11 * 60,
            from_top: false,
        };
        draw_calendar(&da, &cr, 800, 900, &data);
    }
}
