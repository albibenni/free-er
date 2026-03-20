use super::*;
use shared::models::CalendarImportRule;

fn sample_ics_for_date(date: chrono::NaiveDate) -> String {
    let d = date.format("%Y%m%d");
    format!(
        "BEGIN:VCALENDAR\r\n\
BEGIN:VEVENT\r\n\
SUMMARY:Deep Work Session\r\n\
DTSTART:{d}T090000Z\r\n\
DTEND:{d}T110000Z\r\n\
END:VEVENT\r\n\
BEGIN:VEVENT\r\n\
SUMMARY:Lunch break\r\n\
DTSTART:{d}T120000Z\r\n\
DTEND:{d}T130000Z\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n"
    )
}

#[test]
fn parses_matching_events() {
    let default_id = Uuid::new_v4();
    let event_date = chrono::Local::now().date_naive();
    let sample_ics = sample_ics_for_date(event_date);
    let import_rules = vec![CalendarImportRule {
        keyword: "work".into(),
        schedule_type: ScheduleType::Focus,
        rule_set_id: None,
    }];
    let schedules = parse_schedules(&sample_ics, &import_rules, default_id);
    assert_eq!(schedules.len(), 2); // both events are imported; rule only affects type
    let work = schedules
        .iter()
        .find(|s| s.name == "Deep Work Session")
        .unwrap();
    assert_eq!(work.rule_set_id, default_id);
    assert_eq!(work.schedule_type, ScheduleType::Focus);
    assert_eq!(work.start, NaiveTime::from_hms_opt(9, 0, 0).unwrap());
    assert_eq!(work.end, NaiveTime::from_hms_opt(11, 0, 0).unwrap());
}

#[test]
fn break_rule_sets_break_type() {
    let default_id = Uuid::new_v4();
    let event_date = chrono::Local::now().date_naive();
    let sample_ics = sample_ics_for_date(event_date);
    let import_rules = vec![CalendarImportRule {
        keyword: "lunch".into(),
        schedule_type: ScheduleType::Break,
        rule_set_id: None,
    }];
    let schedules = parse_schedules(&sample_ics, &import_rules, default_id);
    let lunch = schedules.iter().find(|s| s.name == "Lunch break").unwrap();
    assert_eq!(lunch.schedule_type, ScheduleType::Break);
    assert_eq!(lunch.rule_set_id, Uuid::nil());
}

#[test]
fn drops_events_outside_three_week_window() {
    let default_id = Uuid::new_v4();
    let far_future = chrono::Local::now().date_naive() + chrono::Duration::weeks(8);
    let sample_ics = sample_ics_for_date(far_future);
    let schedules = parse_schedules(&sample_ics, &[], default_id);
    assert!(schedules.is_empty());
}
