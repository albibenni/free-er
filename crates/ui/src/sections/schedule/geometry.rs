use chrono::Datelike;
use shared::ipc::{ScheduleSummary, ScheduleType};

// ── Calendar layout constants ─────────────────────────────────────────────────

pub(super) const MARGIN_LEFT: f64 = 52.0;
pub(super) const HEADER_H: f64 = 40.0;
pub(super) const MARGIN_RIGHT: f64 = 4.0;
pub(super) const START_HOUR: u32 = 6;
pub(super) const END_HOUR: u32 = 23;

// ── Coordinate math ───────────────────────────────────────────────────────────

/// Clamp a fractional hour (e.g. 7.5 = 07:30) to the visible range,
/// returning a fraction [0, 1] within [START_HOUR, END_HOUR].
pub(super) fn clamp_hour_frac(hour_frac: f64) -> f64 {
    let start = START_HOUR as f64;
    let end = END_HOUR as f64;
    ((hour_frac - start) / (end - start)).clamp(0.0, 1.0)
}

/// Round minutes to the nearest 15-minute boundary.
pub(super) fn snap15(m: u32) -> u32 {
    ((m + 7) / 15) * 15
}

/// Convert a pixel position to (column 0-6, minutes-from-midnight).
/// Returns None if the position is outside the grid area.
pub(super) fn pixel_to_day_time(x: f64, y: f64, w: f64, h: f64) -> Option<(usize, u32)> {
    if x < MARGIN_LEFT || y < HEADER_H {
        return None;
    }
    let col_w = (w - MARGIN_LEFT - MARGIN_RIGHT) / 7.0;
    let hour_h = (h - HEADER_H) / (END_HOUR - START_HOUR) as f64;
    let col = ((x - MARGIN_LEFT) / col_w) as usize;
    if col >= 7 {
        return None;
    }
    let hour_frac = (y - HEADER_H) / hour_h;
    let minutes = (START_HOUR as f64 * 60.0 + hour_frac * 60.0) as u32;
    Some((col, minutes.clamp(START_HOUR * 60, END_HOUR * 60)))
}

/// Find the schedule block under (x, y) and return its metadata.
pub(super) fn hit_test_event(
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    week_offset: i32,
    schedules: &[ScheduleSummary],
) -> Option<(uuid::Uuid, String, usize, u32, u32, bool, ScheduleType, uuid::Uuid)> {
    let col_w = (w - MARGIN_LEFT - MARGIN_RIGHT) / 7.0;

    let today = chrono::Local::now().date_naive();
    let dfm = today.weekday().num_days_from_monday() as i64;
    let this_mon = today - chrono::Duration::days(dfm);
    let week_monday = this_mon + chrono::Duration::weeks(week_offset as i64);

    for sched in schedules {
        let cols: Vec<usize> = event_columns(sched, week_monday);
        for col in cols {
            let ex = MARGIN_LEFT + col as f64 * col_w + 2.0;
            let bw = col_w - 4.0;
            let sf = clamp_hour_frac(sched.start_min as f64 / 60.0);
            let ef = clamp_hour_frac(sched.end_min as f64 / 60.0);
            let ys = HEADER_H + sf * (h - HEADER_H);
            let ye = HEADER_H + ef * (h - HEADER_H);
            let bh = (ye - ys).max(4.0);
            if x >= ex && x <= ex + bw && y >= ys && y <= ys + bh {
                return Some((
                    sched.id,
                    sched.name.clone(),
                    col,
                    sched.start_min,
                    sched.end_min,
                    sched.imported,
                    sched.schedule_type.clone(),
                    sched.rule_set_id,
                ));
            }
        }
    }
    None
}

/// Columns (0–6, Mon–Sun) a schedule occupies in the given week.
pub(super) fn event_columns(
    sched: &ScheduleSummary,
    week_monday: chrono::NaiveDate,
) -> Vec<usize> {
    if let Some(ds) = &sched.specific_date {
        if let Ok(date) = chrono::NaiveDate::parse_from_str(ds, "%Y-%m-%d") {
            let off = (date - week_monday).num_days();
            if off >= 0 && off < 7 {
                return vec![off as usize];
            }
        }
        vec![]
    } else {
        sched.days.iter().map(|&d| d as usize).collect()
    }
}
