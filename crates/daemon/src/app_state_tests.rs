use super::*;
use chrono::{NaiveDate, NaiveTime, Weekday};
use shared::models::{Schedule, ScheduleType};

fn sample_schedule(id: Uuid, name: &str, imported: bool, imported_repeating: bool) -> Schedule {
    Schedule {
        id,
        name: name.to_string(),
        days: vec![Weekday::Mon],
        start: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
        end: NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
        rule_set_id: Uuid::nil(),
        enabled: true,
        imported,
        imported_repeating,
        specific_date: Some(NaiveDate::from_ymd_opt(2026, 3, 16).unwrap()),
        schedule_type: ScheduleType::Focus,
    }
}

#[test]
fn default_rule_set_follows_add_remove() {
    let state = AppState::new(Config::default());
    let rs1 = RuleSet::new("Default");
    let rs1_id = rs1.id;
    let rs2 = RuleSet::new("Work");
    let rs2_id = rs2.id;

    state.add_rule_set(rs1);
    assert_eq!(state.effective_default_rule_set_id(), rs1_id);

    state.add_rule_set(rs2);
    assert_eq!(state.effective_default_rule_set_id(), rs1_id);

    state.remove_rule_set(rs1_id);
    assert_eq!(state.effective_default_rule_set_id(), rs2_id);
}

#[test]
fn set_default_rule_set_rejects_unknown_id() {
    let state = AppState::new(Config::default());
    assert!(!state.set_default_rule_set(Uuid::new_v4()));
}

#[test]
fn add_remove_url_deduplicates() {
    let state = AppState::new(Config::default());
    let rs = RuleSet::new("Dev");
    let id = rs.id;
    state.add_rule_set(rs);

    assert!(state.add_url_to_rule_set(id, "github.com".to_string()));
    assert!(state.add_url_to_rule_set(id, "github.com".to_string()));
    let rule_sets = state.list_rule_sets();
    let urls = &rule_sets.iter().find(|r| r.id == id).unwrap().allowed_urls;
    assert_eq!(urls, &vec!["github.com".to_string()]);

    assert!(state.remove_url_from_rule_set(id, "github.com"));
    let rule_sets = state.list_rule_sets();
    let urls = &rule_sets.iter().find(|r| r.id == id).unwrap().allowed_urls;
    assert!(urls.is_empty());
}

#[test]
fn apply_calendar_schedules_replaces_only_imported_entries() {
    let mut cfg = Config::default();
    let manual = sample_schedule(Uuid::new_v4(), "Manual", false, false);
    let old_imported = sample_schedule(Uuid::new_v4(), "Old Imported", true, false);
    cfg.schedules = vec![manual.clone(), old_imported];

    let state = AppState::new(cfg);
    let new_imported = sample_schedule(Uuid::new_v4(), "New Imported", true, true);
    state.apply_calendar_schedules(vec![new_imported.clone()]);

    let schedules = state.list_schedules();
    assert_eq!(schedules.len(), 2);
    assert!(schedules.iter().any(|s| s.id == manual.id));
    assert!(schedules.iter().any(|s| s.id == new_imported.id));
    assert!(!schedules.iter().any(|s| s.name == "Old Imported"));
}

#[test]
fn import_rules_are_case_insensitive_and_deduplicated() {
    let state = AppState::new(Config::default());
    state.add_import_rule("Deep Work".to_string(), ScheduleType::Focus);
    state.add_import_rule("deep work".to_string(), ScheduleType::Focus);
    state.add_import_rule("Deep Work".to_string(), ScheduleType::Break);

    let rules = state.list_import_rules();
    assert_eq!(rules.len(), 2);
    assert!(rules
        .iter()
        .any(|r| r.keyword == "deep work" && r.schedule_type == ScheduleType::Focus));
    assert!(rules
        .iter()
        .any(|r| r.keyword == "deep work" && r.schedule_type == ScheduleType::Break));
}

#[test]
fn pending_oauth_state_is_consumed() {
    let state = AppState::new(Config::default());
    state.set_pending_oauth_state("state-1".into(), "client".into(), "secret".into());

    assert!(state.take_pending_oauth("wrong-state").is_none());
    assert!(state.take_pending_oauth("state-1").is_none());

    state.set_pending_oauth_state("state-2".into(), "client2".into(), "secret2".into());
    let creds = state.take_pending_oauth("state-2");
    assert_eq!(creds, Some(("client2".to_string(), "secret2".to_string())));
}

#[test]
fn google_calendar_tokens_lifecycle() {
    let state = AppState::new(Config::default());
    state.set_google_calendar_tokens(
        "cid".into(),
        "csecret".into(),
        "access-1".into(),
        "refresh-1".into(),
        100,
    );

    let cfg = state.google_calendar_config().unwrap();
    assert_eq!(cfg.access_token.as_deref(), Some("access-1"));
    assert_eq!(cfg.refresh_token.as_deref(), Some("refresh-1"));
    assert_eq!(cfg.token_expiry_secs, Some(100));

    state.update_google_tokens("access-2".into(), 200);
    let cfg = state.google_calendar_config().unwrap();
    assert_eq!(cfg.access_token.as_deref(), Some("access-2"));
    assert_eq!(cfg.token_expiry_secs, Some(200));

    state.revoke_google_calendar();
    assert!(state.google_calendar_config().is_none());
}
