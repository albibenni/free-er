use super::*;
use chrono::NaiveTime;

#[test]
fn rule_set_new_initializes_empty_allowed_urls() {
    let rs = RuleSet::new("Work");
    assert_eq!(rs.name, "Work");
    assert!(rs.allowed_urls.is_empty());
    assert_ne!(rs.id, Uuid::nil());
}

#[test]
fn schedule_is_active_checks_day_and_time_window() {
    let schedule = Schedule {
        id: Uuid::new_v4(),
        name: "Morning".to_string(),
        days: vec![Weekday::Mon, Weekday::Tue],
        start: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
        end: NaiveTime::from_hms_opt(11, 0, 0).unwrap(),
        rule_set_id: Uuid::new_v4(),
        enabled: true,
        imported: false,
        imported_repeating: false,
        specific_date: None,
        schedule_type: ScheduleType::Focus,
    };

    assert!(schedule.is_active(Weekday::Mon, NaiveTime::from_hms_opt(9, 30, 0).unwrap()));
    assert!(!schedule.is_active(Weekday::Wed, NaiveTime::from_hms_opt(9, 30, 0).unwrap()));
    assert!(!schedule.is_active(Weekday::Mon, NaiveTime::from_hms_opt(8, 59, 59).unwrap()));
    assert!(!schedule.is_active(Weekday::Mon, NaiveTime::from_hms_opt(11, 0, 0).unwrap()));
}

#[test]
fn schedule_is_active_respects_enabled_flag() {
    let schedule = Schedule {
        id: Uuid::new_v4(),
        name: "Disabled".to_string(),
        days: vec![Weekday::Mon],
        start: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
        end: NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
        rule_set_id: Uuid::new_v4(),
        enabled: false,
        imported: false,
        imported_repeating: false,
        specific_date: None,
        schedule_type: ScheduleType::Focus,
    };

    assert!(!schedule.is_active(Weekday::Mon, NaiveTime::from_hms_opt(9, 15, 0).unwrap()));
    assert!(!schedule.is_active_now());
}

#[test]
fn config_default_has_expected_pomodoro_values() {
    let cfg = Config::default();
    assert_eq!(cfg.pomodoro.focus_secs, 25 * 60);
    assert_eq!(cfg.pomodoro.break_secs, 5 * 60);
    assert!(!cfg.pomodoro.strict_breaks);
}
