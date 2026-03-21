use super::*;
use crate::app::test_support::{lock_ipc, MockDaemon};
use shared::ipc::{Command, ImportRuleSummary, RuleSetSummary, ScheduleSummary, ScheduleType, StatusResponse};
use uuid::Uuid;

fn run_async<F, T>(fut: F) -> T
where
    F: std::future::Future<Output = T>,
{
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(fut)
}

#[test]
fn parses_successful_responses() {
    let _ipc_guard = lock_ipc();
    let rule_set_id = Uuid::new_v4();
    let schedule_id = Uuid::new_v4();
    let daemon = MockDaemon::start(move |cmd| match cmd {
        Command::GetStatus => serde_json::to_string(&StatusResponse {
            focus_active: true,
            strict_mode: false,
            active_rule_set_name: Some("Default".to_string()),
            pomodoro_active: true,
            pomodoro_phase: Some(shared::ipc::PomodoroPhase::Focus),
            seconds_remaining: Some(10),
            google_calendar_connected: true,
            allow_new_tab: true,
            default_rule_set_id: Some(rule_set_id),
            accent_color: "#3584e4".to_string(),
        })
        .unwrap(),
        Command::ListRuleSets => serde_json::to_string(&vec![RuleSetSummary {
            id: rule_set_id,
            name: "Default".to_string(),
            allowed_urls: vec!["https://example.com".to_string()],
        }])
        .unwrap(),
        Command::ListSchedules => serde_json::to_string(&vec![ScheduleSummary {
            id: schedule_id,
            name: "Morning".to_string(),
            days: vec![1],
            start_min: 60,
            end_min: 90,
            enabled: true,
            imported: false,
            imported_repeating: false,
            specific_date: None,
            schedule_type: ScheduleType::Focus,
            rule_set_id,
        }])
        .unwrap(),
        Command::ListImportRules => serde_json::to_string(&vec![ImportRuleSummary {
            keyword: "meeting".to_string(),
            schedule_type: ScheduleType::Break,
        }])
        .unwrap(),
        Command::StartGoogleOAuth => serde_json::json!({ "auth_url": "https://example.com/auth" }).to_string(),
        Command::AddRuleSet { .. } => serde_json::json!({ "id": rule_set_id }).to_string(),
        Command::AddSchedule { .. } => serde_json::json!({ "id": schedule_id }).to_string(),
        _ => "{}".to_string(),
    });

    let raw = run_async(send(&Command::StopFocus)).unwrap();
    assert_eq!(raw, "{}");

    let status = run_async(get_status()).unwrap();
    assert!(status.focus_active);
    assert_eq!(status.default_rule_set_id, Some(rule_set_id));

    let sets = run_async(list_rule_sets()).unwrap();
    assert_eq!(sets.len(), 1);
    assert_eq!(sets[0].id, rule_set_id);

    let schedules = run_async(list_schedules()).unwrap();
    assert_eq!(schedules.len(), 1);
    assert_eq!(schedules[0].id, schedule_id);

    let rules = run_async(list_import_rules()).unwrap();
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].keyword, "meeting");

    let auth_url = run_async(start_google_oauth()).unwrap();
    assert_eq!(auth_url, "https://example.com/auth");

    assert!(run_async(revoke_google_calendar()).is_ok());
    assert!(run_async(sync_calendar()).is_ok());
    assert!(run_async(update_schedule(
        schedule_id,
        "x",
        vec![1],
        1,
        2,
        Some(rule_set_id),
        Some("2026-03-20".to_string()),
        ScheduleType::Focus,
    ))
    .is_ok());
    assert!(run_async(remove_schedule(schedule_id)).is_ok());
    assert!(run_async(add_import_rule("k", ScheduleType::Focus)).is_ok());
    assert!(run_async(remove_import_rule("k", ScheduleType::Break)).is_ok());
    assert!(run_async(remove_rule_set(rule_set_id)).is_ok());
    assert!(run_async(set_default_rule_set(rule_set_id)).is_ok());

    let added_schedule_id = run_async(add_schedule(
        "x",
        vec![2],
        10,
        20,
        None,
        None,
        ScheduleType::Break,
    ))
    .unwrap();
    assert_eq!(added_schedule_id, schedule_id);

    let added_set_id = run_async(add_rule_set("new")).unwrap();
    assert_eq!(added_set_id, rule_set_id);

    let received = daemon.received();
    assert!(!received.is_empty());
}

#[test]
fn reports_errors_for_invalid_or_missing_fields() {
    let _ipc_guard = lock_ipc();

    let daemon = MockDaemon::start(|cmd| match cmd {
        Command::StartGoogleOAuth => serde_json::json!({ "error": "oauth failed" }).to_string(),
        Command::AddRuleSet { .. } => serde_json::json!({ "name": "missing id" }).to_string(),
        Command::AddSchedule { .. } => serde_json::json!({ "name": "missing id" }).to_string(),
        Command::GetStatus => "not-json".to_string(),
        _ => "{}".to_string(),
    });

    assert!(run_async(start_google_oauth()).is_err());
    assert!(run_async(add_rule_set("x")).is_err());
    assert!(run_async(add_schedule(
        "x",
        vec![1],
        1,
        2,
        None,
        None,
        ScheduleType::Focus,
    ))
    .is_err());
    assert!(run_async(get_status()).is_err());

    drop(daemon);
    assert!(run_async(send(&Command::StopFocus)).is_err());
}
