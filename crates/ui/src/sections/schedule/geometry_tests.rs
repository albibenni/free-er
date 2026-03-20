use super::*;
use chrono::{Duration, NaiveDate};
use shared::ipc::ScheduleType;
use uuid::Uuid;

fn sched(
    days: Vec<u8>,
    start_min: u32,
    end_min: u32,
    specific_date: Option<String>,
) -> ScheduleSummary {
    ScheduleSummary {
        id: Uuid::new_v4(),
        name: "x".into(),
        days,
        start_min,
        end_min,
        enabled: true,
        imported: false,
        imported_repeating: false,
        specific_date,
        schedule_type: ScheduleType::Focus,
        rule_set_id: Uuid::nil(),
    }
}

#[test]
fn snap15_rounds_to_nearest_quarter() {
    assert_eq!(snap15(0), 0);
    assert_eq!(snap15(7), 0);
    assert_eq!(snap15(8), 15);
    assert_eq!(snap15(22), 15);
    assert_eq!(snap15(53), 60);
}

#[test]
fn clamp_hour_frac_limits_to_range() {
    assert_eq!(clamp_hour_frac(START_HOUR as f64 - 3.0), 0.0);
    assert_eq!(clamp_hour_frac(END_HOUR as f64 + 3.0), 1.0);
    assert!((clamp_hour_frac((START_HOUR + END_HOUR) as f64 / 2.0) - 0.5).abs() < 0.05);
}

#[test]
fn pixel_to_day_time_rejects_out_of_grid() {
    assert_eq!(pixel_to_day_time(0.0, 0.0, 700.0, 900.0), None);
    assert_eq!(
        pixel_to_day_time(MARGIN_LEFT + 1.0, HEADER_H - 1.0, 700.0, 900.0),
        None
    );
}

#[test]
fn pixel_to_day_time_maps_inside_grid() {
    let got = pixel_to_day_time(MARGIN_LEFT + 5.0, HEADER_H + 5.0, 700.0, 900.0);
    assert!(got.is_some());
    let (col, mins) = got.unwrap();
    assert!(col <= 6);
    assert!((START_HOUR * 60..=END_HOUR * 60).contains(&mins));
}

#[test]
fn event_columns_for_weekly_days_uses_days() {
    let week_monday = NaiveDate::from_ymd_opt(2026, 3, 16).unwrap();
    let s = sched(vec![0, 2, 4], 9 * 60, 10 * 60, None);
    assert_eq!(event_columns(&s, week_monday), vec![0, 2, 4]);
}

#[test]
fn event_columns_for_specific_date_in_week_returns_single_col() {
    let week_monday = NaiveDate::from_ymd_opt(2026, 3, 16).unwrap();
    let date = (week_monday + Duration::days(3))
        .format("%Y-%m-%d")
        .to_string();
    let s = sched(vec![], 9 * 60, 10 * 60, Some(date));
    assert_eq!(event_columns(&s, week_monday), vec![3]);
}

#[test]
fn event_columns_for_specific_date_outside_week_returns_empty() {
    let week_monday = NaiveDate::from_ymd_opt(2026, 3, 16).unwrap();
    let date = (week_monday + Duration::days(10))
        .format("%Y-%m-%d")
        .to_string();
    let s = sched(vec![], 9 * 60, 10 * 60, Some(date));
    assert!(event_columns(&s, week_monday).is_empty());
}

#[test]
fn compute_layout_splits_overlapping_events() {
    let week_monday = NaiveDate::from_ymd_opt(2026, 3, 16).unwrap();
    let a = sched(vec![0], 9 * 60, 10 * 60, None);
    let b = sched(vec![0], 9 * 60 + 30, 10 * 60 + 30, None);
    let layouts = compute_layout(&[a, b], week_monday);
    assert_eq!(layouts.len(), 2);
    assert!(layouts.iter().all(|l| l.total_slots == 2));
}

#[test]
fn find_overlap_groups_splits_non_overlapping_events() {
    let week_monday = NaiveDate::from_ymd_opt(2026, 3, 16).unwrap();
    let a = sched(vec![0], 9 * 60, 10 * 60, None);
    let b = sched(vec![0], 11 * 60, 12 * 60, None);
    let schedules = vec![a, b];
    let groups = find_overlap_groups(&[0, 1], &schedules);
    assert_eq!(groups, vec![vec![0], vec![1]]);

    // Sanity: layout still places both on the same day with one slot each.
    let layout = compute_layout(&schedules, week_monday);
    assert_eq!(layout.len(), 2);
    assert!(layout.iter().all(|l| l.total_slots == 1));
}

#[test]
fn hit_test_event_returns_matching_schedule_inside_block() {
    let week_monday = NaiveDate::from_ymd_opt(2026, 3, 16).unwrap();
    let s = sched(vec![0], 9 * 60, 10 * 60, None);
    let id = s.id;
    let schedules = vec![s];
    let layout = compute_layout(&schedules, week_monday);
    let l = &layout[0];

    let w = 700.0;
    let h = 900.0;
    let col_w = (w - MARGIN_LEFT - MARGIN_RIGHT) / 7.0;
    let slot_w = col_w / l.total_slots as f64;
    let x = MARGIN_LEFT + l.col as f64 * col_w + l.slot as f64 * slot_w + slot_w / 2.0;

    let sf = clamp_hour_frac(schedules[0].start_min as f64 / 60.0);
    let ef = clamp_hour_frac(schedules[0].end_min as f64 / 60.0);
    let ys = HEADER_H + sf * (h - HEADER_H);
    let ye = HEADER_H + ef * (h - HEADER_H);
    let y = (ys + ye) / 2.0;

    let hit = hit_test_event(x, y, w, h, 0, &schedules);
    assert!(hit.is_some());
    assert_eq!(hit.unwrap().0, id);
}

#[test]
fn hit_test_event_ignores_disabled_schedule() {
    let mut s = sched(vec![0], 9 * 60, 10 * 60, None);
    s.enabled = false;
    let hit = hit_test_event(100.0, 200.0, 700.0, 900.0, 0, &[s]);
    assert!(hit.is_none());
}

#[test]
fn event_columns_with_invalid_specific_date_is_empty() {
    let week_monday = NaiveDate::from_ymd_opt(2026, 3, 16).unwrap();
    let s = sched(vec![], 9 * 60, 10 * 60, Some("invalid-date".to_string()));
    assert!(event_columns(&s, week_monday).is_empty());
}

#[test]
fn pixel_to_day_time_rejects_right_side_overflow() {
    let w = 700.0;
    let h = 900.0;
    assert_eq!(pixel_to_day_time(w + 1.0, HEADER_H + 10.0, w, h), None);
}
