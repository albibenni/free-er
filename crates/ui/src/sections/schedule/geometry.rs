use shared::ipc::{ScheduleSummary, ScheduleType};

use super::week::week_monday_for_offset;

// ── Block layout ──────────────────────────────────────────────────────────────

/// Rendering slot for one schedule in one column.
#[derive(Debug, Clone)]
pub(super) struct BlockLayout {
    pub sched_id: uuid::Uuid,
    pub col: usize,
    /// Horizontal slot index within the column (0 = leftmost).
    pub slot: usize,
    /// Total number of horizontal slots in this overlap group.
    pub total_slots: usize,
    /// Don't render this block (e.g. focus hidden because break wins).
    pub hidden: bool,
}

/// Compute the rendering layout for all schedules in the given week.
///
/// Rules for overlapping events in the same column:
/// - Break + Focus  → break takes full width; focus blocks are hidden.
/// - Focus + Focus (different rule sets) → merged into one full-width block.
/// - Anything else  → side-by-side (column split equally).
pub(super) fn compute_layout(
    schedules: &[ScheduleSummary],
    week_monday: chrono::NaiveDate,
) -> Vec<BlockLayout> {
    let mut layouts: Vec<BlockLayout> = Vec::new();

    for col in 0..7usize {
        // Collect indices of schedules that appear in this column, sorted by start time.
        let mut col_indices: Vec<usize> = (0..schedules.len())
            .filter(|&i| event_columns(&schedules[i], week_monday).contains(&col))
            .collect();
        col_indices.sort_by_key(|&i| schedules[i].start_min);

        for group in find_overlap_groups(&col_indices, schedules) {
            if group.len() == 1 {
                layouts.push(BlockLayout {
                    sched_id: schedules[group[0]].id,
                    col,
                    slot: 0,
                    total_slots: 1,
                    hidden: false,
                });
                continue;
            }

            // All overlapping events are shown side by side regardless of type.
            let n = group.len();
            for (slot, &i) in group.iter().enumerate() {
                layouts.push(BlockLayout {
                    sched_id: schedules[i].id,
                    col,
                    slot,
                    total_slots: n,
                    hidden: false,
                });
            }
        }
    }

    layouts
}

/// Group sorted schedule indices into sets of overlapping intervals.
fn find_overlap_groups(sorted_indices: &[usize], schedules: &[ScheduleSummary]) -> Vec<Vec<usize>> {
    let mut groups: Vec<Vec<usize>> = Vec::new();
    let mut current: Vec<usize> = Vec::new();
    let mut max_end: u32 = 0;

    for &idx in sorted_indices {
        let s = &schedules[idx];
        if current.is_empty() || s.start_min <= max_end {
            current.push(idx);
            max_end = max_end.max(s.end_min);
        } else {
            groups.push(std::mem::take(&mut current));
            current.push(idx);
            max_end = s.end_min;
        }
    }
    if !current.is_empty() {
        groups.push(current);
    }
    groups
}

// ── Calendar layout constants ─────────────────────────────────────────────────

pub(super) const MARGIN_LEFT: f64 = 52.0;
pub(super) const HEADER_H: f64 = 40.0;
pub(super) const MARGIN_RIGHT: f64 = 4.0;
pub(super) const START_HOUR: u32 = 4;
pub(super) const END_HOUR: u32 = 25; // 25 = 1am next day

// ── Coordinate math ───────────────────────────────────────────────────────────

/// Clamp a fractional hour (e.g. 7.5 = 07:30) to the visible range,
/// returning a fraction [0, 1] within [START_HOUR, END_HOUR].
pub(super) fn clamp_hour_frac(hour_frac: f64) -> f64 {
    let start = START_HOUR as f64;
    let end = END_HOUR as f64;
    ((hour_frac - start) / (end - start)).clamp(0.0, 1.0)
}

/// Convert minutes-from-midnight to a viewport fraction, treating hours before
/// START_HOUR as belonging to the next day (hour + 24).
pub(super) fn extended_hour_frac(min: u32) -> f64 {
    let h = min as f64 / 60.0;
    let extended = if h < START_HOUR as f64 { h + 24.0 } else { h };
    clamp_hour_frac(extended)
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
    let minutes = minutes.clamp(START_HOUR * 60, END_HOUR * 60) % (24 * 60);
    Some((col, minutes))
}

/// Find the schedule block under (x, y) and return its metadata.
pub(super) fn hit_test_event(
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    week_offset: i32,
    schedules: &[ScheduleSummary],
) -> Option<(
    uuid::Uuid,
    String,
    Vec<u8>,
    usize,
    u32,
    u32,
    bool,
    bool,
    ScheduleType,
    uuid::Uuid,
)> {
    let col_w = (w - MARGIN_LEFT - MARGIN_RIGHT) / 7.0;

    let week_monday = week_monday_for_offset(week_offset);

    let layouts = compute_layout(schedules, week_monday);

    for layout in &layouts {
        if layout.hidden {
            continue;
        }
        let sched = match schedules.iter().find(|s| s.id == layout.sched_id) {
            Some(s) => s,
            None => continue,
        };
        if !sched.enabled {
            continue;
        }

        let slot_w = col_w / layout.total_slots as f64;
        let ex = MARGIN_LEFT + layout.col as f64 * col_w + layout.slot as f64 * slot_w + 2.0;
        let bw = slot_w - 4.0;
        let sf = extended_hour_frac(sched.start_min);
        let ef = extended_hour_frac(sched.end_min);
        let ys = HEADER_H + sf * (h - HEADER_H);
        let ye = HEADER_H + ef * (h - HEADER_H);
        let bh = (ye - ys).max(4.0);

        if x >= ex && x <= ex + bw && y >= ys && y <= ys + bh {
            return Some((
                sched.id,
                sched.name.clone(),
                sched.days.clone(),
                layout.col,
                sched.start_min,
                sched.end_min,
                sched.imported,
                sched.imported_repeating,
                sched.schedule_type.clone(),
                sched.rule_set_id,
            ));
        }
    }
    None
}

/// Columns (0–6, Mon–Sun) a schedule occupies in the given week.
pub(super) fn event_columns(sched: &ScheduleSummary, week_monday: chrono::NaiveDate) -> Vec<usize> {
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

#[cfg(test)]
#[path = "geometry_tests.rs"]
mod tests;
