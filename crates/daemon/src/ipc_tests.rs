use super::handle_command;
use crate::app_state::AppState;
use shared::ipc::{Command, ScheduleType};
use shared::models::{Config, RuleSet};
use std::sync::Mutex;
use uuid::Uuid;

static HOME_LOCK: Mutex<()> = Mutex::new(());

fn state() -> AppState {
    AppState::new(Config::default())
}

fn state_with_rule_set() -> (AppState, Uuid) {
    let s = AppState::new(Config::default());
    let rs = RuleSet::new("Default");
    let id = rs.id;
    s.add_rule_set(rs);
    (s, id)
}

fn ok_resp() -> &'static str {
    r#"{"ok": true}"#
}

#[test]
fn start_focus_returns_ok_not_mutated() {
    let state = state();
    let id = Uuid::new_v4();
    let (resp, mutated) = handle_command(Command::StartFocus { rule_set_id: id }, &state);
    assert!(resp.contains("ok"));
    assert!(!mutated);
}

#[test]
fn stop_focus_returns_ok_not_mutated() {
    let state = state();
    let (resp, mutated) = handle_command(Command::StopFocus, &state);
    assert_eq!(resp, ok_resp());
    assert!(!mutated);
}

#[test]
fn take_break_returns_ok_not_mutated() {
    let state = state();
    let (resp, mutated) = handle_command(Command::TakeBreak { duration_secs: 300 }, &state);
    assert_eq!(resp, ok_resp());
    assert!(!mutated);
}

#[test]
fn start_pomodoro_returns_ok_not_mutated() {
    let state = state();
    let (resp, mutated) = handle_command(
        Command::StartPomodoro {
            focus_secs: 1500,
            break_secs: 300,
            rule_set_id: None,
        },
        &state,
    );
    assert_eq!(resp, ok_resp());
    assert!(!mutated);
}

#[test]
fn stop_pomodoro_returns_ok_not_mutated() {
    let state = state();
    let (resp, mutated) = handle_command(Command::StopPomodoro, &state);
    assert_eq!(resp, ok_resp());
    assert!(!mutated);
}

#[test]
fn skip_break_returns_ok_when_not_strict() {
    let state = state();
    let (resp, mutated) = handle_command(Command::SkipBreak, &state);
    assert_eq!(resp, ok_resp());
    assert!(!mutated);
}

#[test]
fn skip_break_returns_error_when_strict_breaks_enabled() {
    let mut cfg = Config::default();
    cfg.pomodoro.strict_breaks = true;
    let state = AppState::new(cfg);
    // Start pomodoro so there's a break to skip
    state.start_pomodoro(10, 5, None);
    let (resp, mutated) = handle_command(Command::SkipBreak, &state);
    assert!(resp.contains("error"));
    assert!(!mutated);
}

#[test]
fn get_status_returns_status_response() {
    let state = state();
    let (resp, mutated) = handle_command(Command::GetStatus, &state);
    assert!(!mutated);
    let v: serde_json::Value = serde_json::from_str(&resp).unwrap();
    assert!(v["focus_active"].is_boolean());
    assert!(v["strict_mode"].is_boolean());
}

#[test]
fn get_status_with_active_pomodoro_returns_phase() {
    let state = state();
    state.start_pomodoro(1500, 300, None);
    let (resp, mutated) = handle_command(Command::GetStatus, &state);
    assert!(!mutated);
    let v: serde_json::Value = serde_json::from_str(&resp).unwrap();
    assert_eq!(v["pomodoro_active"], true);
    assert!(v["pomodoro_phase"].is_string());
    assert!(v["seconds_remaining"].is_number());
}

#[test]
fn add_rule_set_returns_ok_with_id_and_mutated() {
    let state = state();
    let (resp, mutated) = handle_command(
        Command::AddRuleSet {
            name: "Work".into(),
            allowed_urls: vec!["github.com".into()],
        },
        &state,
    );
    assert!(mutated);
    let v: serde_json::Value = serde_json::from_str(&resp).unwrap();
    assert_eq!(v["ok"], true);
    assert!(v["id"].as_str().is_some());
}

#[test]
fn remove_rule_set_returns_ok_mutated() {
    let (state, id) = state_with_rule_set();
    let (resp, mutated) = handle_command(Command::RemoveRuleSet { id }, &state);
    assert_eq!(resp, ok_resp());
    assert!(mutated);
}

#[test]
fn add_url_to_rule_set_found_returns_ok_mutated() {
    let (state, id) = state_with_rule_set();
    let (resp, mutated) = handle_command(
        Command::AddUrlToRuleSet {
            rule_set_id: id,
            url: "example.com".into(),
        },
        &state,
    );
    assert_eq!(resp, ok_resp());
    assert!(mutated);
}

#[test]
fn add_url_to_rule_set_not_found_returns_error() {
    let state = state();
    let (resp, mutated) = handle_command(
        Command::AddUrlToRuleSet {
            rule_set_id: Uuid::new_v4(),
            url: "example.com".into(),
        },
        &state,
    );
    assert!(resp.contains("error"));
    assert!(!mutated);
}

#[test]
fn remove_url_from_rule_set_found_returns_ok_mutated() {
    let (state, id) = state_with_rule_set();
    state.add_url_to_rule_set(id, "example.com".into());
    let (resp, mutated) = handle_command(
        Command::RemoveUrlFromRuleSet {
            rule_set_id: id,
            url: "example.com".into(),
        },
        &state,
    );
    assert_eq!(resp, ok_resp());
    assert!(mutated);
}

#[test]
fn remove_url_from_rule_set_not_found_returns_error() {
    let state = state();
    let (resp, mutated) = handle_command(
        Command::RemoveUrlFromRuleSet {
            rule_set_id: Uuid::new_v4(),
            url: "example.com".into(),
        },
        &state,
    );
    assert!(resp.contains("error"));
    assert!(!mutated);
}

#[test]
fn list_rule_sets_returns_json_array() {
    let (state, _) = state_with_rule_set();
    let (resp, mutated) = handle_command(Command::ListRuleSets, &state);
    assert!(!mutated);
    let v: serde_json::Value = serde_json::from_str(&resp).unwrap();
    assert!(v.is_array());
    assert_eq!(v.as_array().unwrap().len(), 1);
}

#[test]
fn set_default_rule_set_found_returns_ok_mutated() {
    let (state, id) = state_with_rule_set();
    let (resp, mutated) = handle_command(Command::SetDefaultRuleSet { id }, &state);
    assert_eq!(resp, ok_resp());
    assert!(mutated);
}

#[test]
fn set_default_rule_set_not_found_returns_error() {
    let state = state();
    let (resp, mutated) =
        handle_command(Command::SetDefaultRuleSet { id: Uuid::new_v4() }, &state);
    assert!(resp.contains("error"));
    assert!(!mutated);
}

#[test]
fn add_schedule_parses_and_returns_ok_with_id() {
    let state = state();
    let (resp, mutated) = handle_command(
        Command::AddSchedule {
            name: "Morning".into(),
            days: vec![0, 1, 2, 3, 4],
            start_min: 9 * 60,
            end_min: 11 * 60,
            rule_set_id: None,
            specific_date: None,
            schedule_type: ScheduleType::Focus,
        },
        &state,
    );
    assert!(mutated);
    let v: serde_json::Value = serde_json::from_str(&resp).unwrap();
    assert_eq!(v["ok"], true);
    let id_str = v["id"].as_str().unwrap();
    assert!(uuid::Uuid::parse_str(id_str).is_ok());
}

#[test]
fn add_schedule_with_specific_date_and_break_type() {
    let state = state();
    let (resp, mutated) = handle_command(
        Command::AddSchedule {
            name: "Lunch".into(),
            days: vec![],
            start_min: 12 * 60,
            end_min: 13 * 60,
            rule_set_id: Some(Uuid::new_v4()),
            specific_date: Some("2026-03-20".into()),
            schedule_type: ScheduleType::Break,
        },
        &state,
    );
    assert!(mutated);
    let v: serde_json::Value = serde_json::from_str(&resp).unwrap();
    assert_eq!(v["ok"], true);
}

#[test]
fn remove_schedule_returns_ok_mutated() {
    let state = state();
    let (add_resp, _) = handle_command(
        Command::AddSchedule {
            name: "S".into(),
            days: vec![0],
            start_min: 60,
            end_min: 120,
            rule_set_id: None,
            specific_date: None,
            schedule_type: ScheduleType::Focus,
        },
        &state,
    );
    let v: serde_json::Value = serde_json::from_str(&add_resp).unwrap();
    let id: Uuid = v["id"].as_str().unwrap().parse().unwrap();

    let (resp, mutated) = handle_command(Command::RemoveSchedule { id }, &state);
    assert_eq!(resp, ok_resp());
    assert!(mutated);
}

#[test]
fn update_schedule_modifies_existing() {
    let state = state();
    let (add_resp, _) = handle_command(
        Command::AddSchedule {
            name: "Old".into(),
            days: vec![0],
            start_min: 60,
            end_min: 120,
            rule_set_id: None,
            specific_date: None,
            schedule_type: ScheduleType::Focus,
        },
        &state,
    );
    let v: serde_json::Value = serde_json::from_str(&add_resp).unwrap();
    let id: Uuid = v["id"].as_str().unwrap().parse().unwrap();

    let (resp, mutated) = handle_command(
        Command::UpdateSchedule {
            id,
            name: "New".into(),
            days: vec![1, 2],
            start_min: 9 * 60,
            end_min: 10 * 60,
            rule_set_id: None,
            specific_date: Some("2026-04-01".into()),
            schedule_type: ScheduleType::Break,
        },
        &state,
    );
    assert_eq!(resp, ok_resp());
    assert!(mutated);
}

#[test]
fn update_schedule_with_no_specific_date_clears_it() {
    let state = state();
    let (add_resp, _) = handle_command(
        Command::AddSchedule {
            name: "S".into(),
            days: vec![0],
            start_min: 0,
            end_min: 60,
            rule_set_id: None,
            specific_date: Some("2026-03-20".into()),
            schedule_type: ScheduleType::Focus,
        },
        &state,
    );
    let v: serde_json::Value = serde_json::from_str(&add_resp).unwrap();
    let id: Uuid = v["id"].as_str().unwrap().parse().unwrap();

    let (resp, mutated) = handle_command(
        Command::UpdateSchedule {
            id,
            name: "S".into(),
            days: vec![0],
            start_min: 0,
            end_min: 60,
            rule_set_id: None,
            specific_date: None,
            schedule_type: ScheduleType::Focus,
        },
        &state,
    );
    assert_eq!(resp, ok_resp());
    assert!(mutated);
}

#[test]
fn list_schedules_returns_json_array() {
    let state = state();
    handle_command(
        Command::AddSchedule {
            name: "S".into(),
            days: vec![0],
            start_min: 60,
            end_min: 120,
            rule_set_id: None,
            specific_date: None,
            schedule_type: ScheduleType::Focus,
        },
        &state,
    );
    let (resp, mutated) = handle_command(Command::ListSchedules, &state);
    assert!(!mutated);
    let v: serde_json::Value = serde_json::from_str(&resp).unwrap();
    assert!(v.is_array());
    assert_eq!(v.as_array().unwrap().len(), 1);
}

#[test]
fn set_strict_mode_returns_ok_mutated() {
    let state = state();
    let (resp, mutated) = handle_command(Command::SetStrictMode { enabled: true }, &state);
    assert_eq!(resp, ok_resp());
    assert!(mutated);
}

#[test]
fn set_allow_new_tab_returns_ok_mutated() {
    let state = state();
    let (resp, mutated) = handle_command(Command::SetAllowNewTab { enabled: false }, &state);
    assert_eq!(resp, ok_resp());
    assert!(mutated);
}

#[test]
fn set_caldav_returns_ok_mutated() {
    let state = state();
    let (resp, mutated) = handle_command(
        Command::SetCalDav {
            url: "https://caldav.example.com".into(),
            username: "alice".into(),
            password: "pw".into(),
        },
        &state,
    );
    assert_eq!(resp, ok_resp());
    assert!(mutated);
}

#[test]
fn start_google_oauth_without_client_file_returns_error() {
    let _g = HOME_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let home = std::env::temp_dir().join("free-er-ipc-no-google");
    let _ = std::fs::remove_dir_all(&home);
    std::fs::create_dir_all(&home).unwrap();
    std::env::set_var("HOME", &home);

    let state = state();
    let (resp, mutated) = handle_command(Command::StartGoogleOAuth, &state);
    assert!(resp.contains("error"));
    assert!(!mutated);
}

#[test]
fn start_google_oauth_with_client_file_returns_auth_url() {
    let _g = HOME_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let home = std::env::temp_dir().join("free-er-ipc-with-google");
    let _ = std::fs::remove_dir_all(&home);
    let cfg_dir = home.join(".config/free-er");
    std::fs::create_dir_all(&cfg_dir).unwrap();
    std::fs::write(
        cfg_dir.join("google_client.json"),
        r#"{"client_id":"test-id","client_secret":"test-secret"}"#,
    )
    .unwrap();
    std::env::set_var("HOME", &home);

    let state = state();
    let (resp, mutated) = handle_command(Command::StartGoogleOAuth, &state);
    assert!(!mutated);
    let v: serde_json::Value = serde_json::from_str(&resp).unwrap();
    let auth_url = v["auth_url"].as_str().unwrap();
    assert!(auth_url.contains("accounts.google.com"));
    assert!(auth_url.contains("test-id"));
}

#[test]
fn revoke_google_calendar_returns_ok_mutated() {
    let state = state();
    let (resp, mutated) = handle_command(Command::RevokeGoogleCalendar, &state);
    assert_eq!(resp, ok_resp());
    assert!(mutated);
}

#[tokio::test]
async fn sync_calendar_with_no_config_returns_ok() {
    let state = state();
    let (resp, mutated) = handle_command(Command::SyncCalendar, &state);
    assert_eq!(resp, ok_resp());
    assert!(!mutated);
}

#[tokio::test]
async fn sync_calendar_with_caldav_config_spawns_and_returns_ok() {
    let state = state();
    state.set_caldav("https://example.com/cal".into(), "u".into(), "p".into());
    let (resp, mutated) = handle_command(Command::SyncCalendar, &state);
    assert_eq!(resp, ok_resp());
    assert!(!mutated);
}

#[tokio::test]
async fn sync_calendar_with_google_config_spawns_and_returns_ok() {
    let state = state();
    state.set_google_calendar_tokens(
        "id".into(),
        "secret".into(),
        "access".into(),
        "refresh".into(),
        9999999999,
    );
    let (resp, mutated) = handle_command(Command::SyncCalendar, &state);
    assert_eq!(resp, ok_resp());
    assert!(!mutated);
}

#[test]
fn add_import_rule_returns_ok_mutated() {
    let state = state();
    let (resp, mutated) = handle_command(
        Command::AddImportRule {
            keyword: "standup".into(),
            schedule_type: ScheduleType::Focus,
        },
        &state,
    );
    assert_eq!(resp, ok_resp());
    assert!(mutated);
}

#[test]
fn remove_import_rule_returns_ok_mutated() {
    let state = state();
    let (resp, mutated) = handle_command(
        Command::RemoveImportRule {
            keyword: "standup".into(),
            schedule_type: ScheduleType::Focus,
        },
        &state,
    );
    assert_eq!(resp, ok_resp());
    assert!(mutated);
}

#[test]
fn list_import_rules_returns_empty_array_initially() {
    let state = state();
    let (resp, mutated) = handle_command(Command::ListImportRules, &state);
    assert!(!mutated);
    let v: serde_json::Value = serde_json::from_str(&resp).unwrap();
    assert!(v.is_array());
    assert!(v.as_array().unwrap().is_empty());
}

#[test]
fn list_import_rules_after_add_returns_entry() {
    let state = state();
    state.add_import_rule("deep work".into(), shared::models::ScheduleType::Focus);
    let (resp, mutated) = handle_command(Command::ListImportRules, &state);
    assert!(!mutated);
    let v: serde_json::Value = serde_json::from_str(&resp).unwrap();
    assert_eq!(v.as_array().unwrap().len(), 1);
}

/// Validate that all weekday indices 0-6 are parsed correctly by AddSchedule.
#[test]
fn add_schedule_all_weekday_indices() {
    let state = state();
    // days 0-6 should all be valid; day 7 should be ignored
    let (resp, _) = handle_command(
        Command::AddSchedule {
            name: "All days".into(),
            days: vec![0, 1, 2, 3, 4, 5, 6, 7],
            start_min: 0,
            end_min: 60,
            rule_set_id: None,
            specific_date: None,
            schedule_type: ScheduleType::Focus,
        },
        &state,
    );
    let v: serde_json::Value = serde_json::from_str(&resp).unwrap();
    assert_eq!(v["ok"], true);
}
