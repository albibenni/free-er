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

#[cfg(test)]
mod tests {
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
}
