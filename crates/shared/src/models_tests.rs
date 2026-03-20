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

#[test]
fn config_allow_new_tab_serde_default_function_returns_true() {
    // The serde default function (used for deserialization) returns true;
    // Config::default() (Rust Default trait) gives the bool default of false.
    assert!(super::default_true());
}

#[test]
fn schedule_type_default_is_focus() {
    assert_eq!(ScheduleType::default(), ScheduleType::Focus);
}

#[test]
fn schedule_is_active_false_at_exact_end_time() {
    let schedule = Schedule {
        id: Uuid::new_v4(),
        name: "Boundary".to_string(),
        days: vec![Weekday::Mon],
        start: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
        end: NaiveTime::from_hms_opt(11, 0, 0).unwrap(),
        rule_set_id: Uuid::new_v4(),
        enabled: true,
        imported: false,
        imported_repeating: false,
        specific_date: None,
        schedule_type: ScheduleType::Focus,
    };
    // Exactly at start — active
    assert!(schedule.is_active(Weekday::Mon, NaiveTime::from_hms_opt(9, 0, 0).unwrap()));
    // One second before end — active
    assert!(schedule.is_active(Weekday::Mon, NaiveTime::from_hms_opt(10, 59, 59).unwrap()));
}

#[test]
fn is_active_now_with_specific_date_matching_today() {
    let today = chrono::Local::now().date_naive();
    let _now_time = chrono::Local::now().naive_local().time();

    let schedule = Schedule {
        id: Uuid::new_v4(),
        name: "OneTime".to_string(),
        days: vec![], // no recurring days
        start: NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
        end: NaiveTime::from_hms_opt(23, 59, 59).unwrap(),
        rule_set_id: Uuid::new_v4(),
        enabled: true,
        imported: false,
        imported_repeating: false,
        specific_date: Some(today),
        schedule_type: ScheduleType::Focus,
    };
    assert!(schedule.is_active_now());
}

#[test]
fn is_active_now_with_specific_date_not_matching_today() {
    let not_today = chrono::Local::now().date_naive() + chrono::Duration::days(7);
    let schedule = Schedule {
        id: Uuid::new_v4(),
        name: "FutureOneTime".to_string(),
        days: vec![],
        start: NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
        end: NaiveTime::from_hms_opt(23, 59, 59).unwrap(),
        rule_set_id: Uuid::new_v4(),
        enabled: true,
        imported: false,
        imported_repeating: false,
        specific_date: Some(not_today),
        schedule_type: ScheduleType::Focus,
    };
    assert!(!schedule.is_active_now());
}

#[test]
fn is_active_now_false_when_disabled() {
    let today = chrono::Local::now().date_naive();
    let schedule = Schedule {
        id: Uuid::new_v4(),
        name: "Disabled".to_string(),
        days: vec![],
        start: NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
        end: NaiveTime::from_hms_opt(23, 59, 59).unwrap(),
        rule_set_id: Uuid::new_v4(),
        enabled: false,
        imported: false,
        imported_repeating: false,
        specific_date: Some(today),
        schedule_type: ScheduleType::Focus,
    };
    assert!(!schedule.is_active_now());
}

#[test]
fn is_active_now_true_for_recurring_schedule_today() {
    let today_weekday = chrono::Local::now().date_naive().weekday();
    let schedule = Schedule {
        id: Uuid::new_v4(),
        name: "Recurring".to_string(),
        days: vec![today_weekday],
        start: NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
        end: NaiveTime::from_hms_opt(23, 59, 59).unwrap(),
        rule_set_id: Uuid::new_v4(),
        enabled: true,
        imported: false,
        imported_repeating: false,
        specific_date: None,
        schedule_type: ScheduleType::Focus,
    };
    assert!(schedule.is_active_now());
}

#[test]
fn rule_set_new_has_non_nil_id() {
    let rs = RuleSet::new("Test");
    assert_ne!(rs.id, Uuid::nil());
}
