use super::*;
use chrono::Datelike;

#[test]
fn clamps_week_offset_to_bounds() {
    assert_eq!(clamp_week_offset(-99), MIN_WEEK_OFFSET);
    assert_eq!(clamp_week_offset(99), MAX_WEEK_OFFSET);
    assert_eq!(clamp_week_offset(0), 0);
}

#[test]
fn monday_offset_moves_by_whole_weeks() {
    let this = week_monday_for_offset(0);
    let prev = week_monday_for_offset(-1);
    let next = week_monday_for_offset(1);
    assert_eq!((this - prev).num_days(), 7);
    assert_eq!((next - this).num_days(), 7);
    assert_eq!(this.weekday(), chrono::Weekday::Mon);
}

#[test]
fn week_label_is_non_empty() {
    let lbl = week_label_text(0);
    assert!(!lbl.trim().is_empty());
}

#[test]
fn week_label_formats_single_month_range() {
    let monday = chrono::NaiveDate::from_ymd_opt(2026, 3, 16).unwrap();
    assert_eq!(format_week_label(monday), "Mar 16–22");
}

#[test]
fn week_label_formats_cross_month_range() {
    let monday = chrono::NaiveDate::from_ymd_opt(2026, 3, 30).unwrap();
    assert_eq!(format_week_label(monday), "Mar 30 – Apr 5");
}
