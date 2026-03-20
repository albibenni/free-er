use super::*;
use shared::models::{CalendarImportRule, ScheduleType};

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

#[test]
fn drops_events_before_window_start() {
    let default_id = Uuid::new_v4();
    let far_past = chrono::Local::now().date_naive() - chrono::Duration::weeks(8);
    let sample_ics = sample_ics_for_date(far_past);
    let schedules = parse_schedules(&sample_ics, &[], default_id);
    assert!(schedules.is_empty());
}

#[test]
fn parses_date_only_ics_events() {
    let default_id = Uuid::new_v4();
    let today = chrono::Local::now().date_naive();
    let d = today.format("%Y%m%d");
    // Date-only format (no time component)
    let ics = format!(
        "BEGIN:VCALENDAR\r\n\
BEGIN:VEVENT\r\n\
SUMMARY:All Day Task\r\n\
DTSTART:{d}\r\n\
DTEND:{d}\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n"
    );
    let schedules = parse_schedules(&ics, &[], default_id);
    assert_eq!(schedules.len(), 1);
    assert_eq!(schedules[0].name, "All Day Task");
}

#[test]
fn ics_event_with_rrule_sets_imported_repeating() {
    let default_id = Uuid::new_v4();
    let today = chrono::Local::now().date_naive();
    let d = today.format("%Y%m%d");
    let ics = format!(
        "BEGIN:VCALENDAR\r\n\
BEGIN:VEVENT\r\n\
SUMMARY:Weekly Standup\r\n\
DTSTART:{d}T100000Z\r\n\
DTEND:{d}T110000Z\r\n\
RRULE:FREQ=WEEKLY;BYDAY=MO\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n"
    );
    let schedules = parse_schedules(&ics, &[], default_id);
    assert_eq!(schedules.len(), 1);
    assert!(schedules[0].imported_repeating);
}

#[test]
fn event_without_summary_is_skipped() {
    let default_id = Uuid::new_v4();
    let today = chrono::Local::now().date_naive();
    let d = today.format("%Y%m%d");
    let ics = format!(
        "BEGIN:VCALENDAR\r\n\
BEGIN:VEVENT\r\n\
DTSTART:{d}T100000Z\r\n\
DTEND:{d}T110000Z\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n"
    );
    let schedules = parse_schedules(&ics, &[], default_id);
    assert!(schedules.is_empty());
}

#[test]
fn resolve_rule_defaults_to_focus_with_default_id_when_no_matching_rule() {
    let default_id = Uuid::new_v4();
    let (stype, rs_id) = resolve_rule("No match here", &[], default_id);
    assert_eq!(stype, ScheduleType::Focus);
    assert_eq!(rs_id, default_id);
}

#[test]
fn resolve_rule_uses_override_rule_set_id_for_focus() {
    use shared::models::CalendarImportRule;
    let default_id = Uuid::new_v4();
    let override_id = Uuid::new_v4();
    let rules = vec![CalendarImportRule {
        keyword: "work".into(),
        schedule_type: ScheduleType::Focus,
        rule_set_id: Some(override_id),
    }];
    let (stype, rs_id) = resolve_rule("Deep Work Session", &rules, default_id);
    assert_eq!(stype, ScheduleType::Focus);
    assert_eq!(rs_id, override_id);
}

#[test]
fn resolve_rule_break_uses_nil_uuid() {
    use shared::models::CalendarImportRule;
    let default_id = Uuid::new_v4();
    let rules = vec![CalendarImportRule {
        keyword: "lunch".into(),
        schedule_type: ScheduleType::Break,
        rule_set_id: None,
    }];
    let (stype, rs_id) = resolve_rule("Lunch Break", &rules, default_id);
    assert_eq!(stype, ScheduleType::Break);
    assert_eq!(rs_id, Uuid::nil());
}

#[test]
fn google_event_to_schedule_with_datetime_format() {
    let default_id = Uuid::new_v4();
    let today = chrono::Local::now().date_naive();
    let start = format!("{today}T09:00:00Z");
    let end = format!("{today}T11:00:00Z");

    let event = serde_json::json!({
        "summary": "Team sync",
        "start": { "dateTime": start },
        "end": { "dateTime": end }
    });

    let (window_start, window_end) = schedule_window_bounds();
    let result = google_event_to_schedule(&event, &[], window_start, window_end, default_id);
    assert!(result.is_some());
    let s = result.unwrap();
    assert_eq!(s.name, "Team sync");
    assert!(!s.imported_repeating);
}

#[test]
fn google_event_to_schedule_with_date_only_format() {
    let default_id = Uuid::new_v4();
    let today = chrono::Local::now().date_naive();
    let d = today.format("%Y-%m-%d").to_string();

    let event = serde_json::json!({
        "summary": "All Day Event",
        "start": { "date": &d },
        "end": { "date": &d }
    });

    let (window_start, window_end) = schedule_window_bounds();
    let result = google_event_to_schedule(&event, &[], window_start, window_end, default_id);
    assert!(result.is_some());
}

#[test]
fn google_event_to_schedule_missing_summary_returns_none() {
    let default_id = Uuid::new_v4();
    let today = chrono::Local::now().date_naive();
    let start = format!("{today}T09:00:00Z");
    let end = format!("{today}T11:00:00Z");

    let event = serde_json::json!({
        "start": { "dateTime": start },
        "end": { "dateTime": end }
    });

    let (window_start, window_end) = schedule_window_bounds();
    let result = google_event_to_schedule(&event, &[], window_start, window_end, default_id);
    assert!(result.is_none());
}

#[test]
fn google_event_to_schedule_outside_window_returns_none() {
    let default_id = Uuid::new_v4();
    let far_future = (chrono::Local::now().date_naive() + chrono::Duration::weeks(8))
        .format("%Y-%m-%d")
        .to_string();

    let event = serde_json::json!({
        "summary": "Future Event",
        "start": { "date": &far_future },
        "end": { "date": &far_future }
    });

    let (window_start, window_end) = schedule_window_bounds();
    let result = google_event_to_schedule(&event, &[], window_start, window_end, default_id);
    assert!(result.is_none());
}

#[test]
fn google_event_to_schedule_with_recurring_event_id_sets_imported_repeating() {
    let default_id = Uuid::new_v4();
    let today = chrono::Local::now().date_naive();
    let start = format!("{today}T09:00:00Z");
    let end = format!("{today}T11:00:00Z");

    let event = serde_json::json!({
        "summary": "Recurring",
        "start": { "dateTime": start },
        "end": { "dateTime": end },
        "recurringEventId": "some-recurrence-id"
    });

    let (window_start, window_end) = schedule_window_bounds();
    let result = google_event_to_schedule(&event, &[], window_start, window_end, default_id);
    assert!(result.is_some());
    assert!(result.unwrap().imported_repeating);
}

#[test]
fn google_event_missing_start_returns_none() {
    let default_id = Uuid::new_v4();
    let today = chrono::Local::now().date_naive();
    let end = format!("{today}T11:00:00Z");

    let event = serde_json::json!({
        "summary": "No Start",
        "end": { "dateTime": end }
    });

    let (window_start, window_end) = schedule_window_bounds();
    let result = google_event_to_schedule(&event, &[], window_start, window_end, default_id);
    assert!(result.is_none());
}
