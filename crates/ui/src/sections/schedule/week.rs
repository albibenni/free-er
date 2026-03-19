use chrono::{Datelike, Duration, Local};

pub(super) const MIN_WEEK_OFFSET: i32 = -1;
pub(super) const MAX_WEEK_OFFSET: i32 = 1;

pub(super) fn clamp_week_offset(offset: i32) -> i32 {
    offset.clamp(MIN_WEEK_OFFSET, MAX_WEEK_OFFSET)
}

pub(super) fn week_monday_for_offset(offset: i32) -> chrono::NaiveDate {
    let today = Local::now().date_naive();
    let days_from_mon = today.weekday().num_days_from_monday() as i64;
    let this_monday = today - Duration::days(days_from_mon);
    this_monday + Duration::weeks(offset as i64)
}

pub(super) fn week_label_text(offset: i32) -> String {
    let week_monday = week_monday_for_offset(offset);
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
