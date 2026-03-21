use super::*;
use chrono::{Datelike, NaiveDate, NaiveTime, Weekday};
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

// ── new coverage ──────────────────────────────────────────────────────────────

#[test]
fn new_creates_state_with_provided_config() {
    let mut cfg = Config::default();
    cfg.strict_mode = true;
    let state = AppState::new(cfg);
    assert!(state.config().strict_mode);
}

#[test]
fn start_and_stop_focus() {
    let state = AppState::new(Config::default());
    let rs = RuleSet::new("Work");
    let id = rs.id;
    state.add_rule_set(rs);

    state.start_focus(id);
    let snap = state.snapshot();
    assert!(snap.focus_active);
    assert_eq!(snap.active_rule_set_name.as_deref(), Some("Work"));

    state.stop_focus();
    let snap = state.snapshot();
    assert!(!snap.focus_active);
    assert!(snap.active_rule_set_name.is_none());
}

#[test]
fn active_rule_set_returns_none_when_no_focus() {
    let state = AppState::new(Config::default());
    assert!(state.active_rule_set().is_none());
}

#[test]
fn active_rule_set_returns_some_when_focus_active() {
    let state = AppState::new(Config::default());
    let rs = RuleSet::new("Dev");
    let id = rs.id;
    state.add_rule_set(rs);
    state.start_focus(id);
    assert!(state.active_rule_set().is_some());
}

#[test]
fn start_pomodoro_sets_focus_and_pomodoro() {
    let state = AppState::new(Config::default());
    state.start_pomodoro(1500, 300, None);
    let snap = state.snapshot();
    assert!(snap.focus_active);
    assert!(snap.pomodoro_active);
    assert_eq!(snap.pomodoro_phase.as_ref().map(|p| format!("{p:?}")), Some("Focus".into()));
}

#[test]
fn stop_pomodoro_clears_pomodoro_and_focus() {
    let state = AppState::new(Config::default());
    state.start_pomodoro(1500, 300, None);
    state.stop_pomodoro();
    let snap = state.snapshot();
    assert!(!snap.pomodoro_active);
    // Focus is also cleared so the schedule loop can cleanly resume.
    assert!(!snap.focus_active);
}

#[test]
fn tick_advances_expired_pomodoro() {
    use std::time::Duration;
    let state = AppState::new(Config::default());
    state.start_pomodoro(1, 300, None); // 1-second focus phase

    // Force expiry by manipulating the timer's started_at through tick
    // Wait for it to expire naturally (1 second)
    std::thread::sleep(Duration::from_millis(1100));
    state.tick();

    let snap = state.snapshot();
    // After tick, phase should have advanced to Break
    assert_eq!(
        snap.pomodoro_phase.as_ref().map(|p| format!("{p:?}")),
        Some("Break".into())
    );
}

#[test]
fn tick_does_nothing_when_no_pomodoro() {
    let state = AppState::new(Config::default());
    state.tick(); // should not panic
}

#[test]
fn tick_does_nothing_when_pomodoro_not_expired() {
    let state = AppState::new(Config::default());
    state.start_pomodoro(3600, 300, None); // long phase
    state.tick(); // phase should not advance
    let snap = state.snapshot();
    assert_eq!(
        snap.pomodoro_phase.as_ref().map(|p| format!("{p:?}")),
        Some("Focus".into())
    );
}

#[test]
fn skip_break_returns_true_when_not_strict() {
    let state = AppState::new(Config::default());
    assert!(state.skip_break()); // no pomodoro → still returns true
}

#[test]
fn skip_break_returns_false_when_strict_breaks() {
    let mut cfg = Config::default();
    cfg.pomodoro.strict_breaks = true;
    let state = AppState::new(cfg);
    assert!(!state.skip_break());
}

#[test]
fn skip_break_during_break_phase_advances_to_focus() {
    use std::time::Duration;
    let state = AppState::new(Config::default());
    state.start_pomodoro(1, 3600, None); // 1-second focus

    std::thread::sleep(Duration::from_millis(1100));
    state.tick(); // advance to Break phase

    let snap = state.snapshot();
    assert_eq!(
        snap.pomodoro_phase.as_ref().map(|p| format!("{p:?}")),
        Some("Break".into())
    );

    assert!(state.skip_break());
    let snap = state.snapshot();
    assert_eq!(
        snap.pomodoro_phase.as_ref().map(|p| format!("{p:?}")),
        Some("Focus".into())
    );
}

#[test]
fn schedule_crud_operations() {
    let state = AppState::new(Config::default());
    let s = sample_schedule(Uuid::new_v4(), "Morning", false, false);
    let sid = s.id;

    state.add_schedule(s);
    assert_eq!(state.list_schedules().len(), 1);

    state.update_schedule(
        sid,
        "Evening".into(),
        vec![Weekday::Fri],
        NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
        NaiveTime::from_hms_opt(20, 0, 0).unwrap(),
        None,
        None,
        ScheduleType::Break,
    );
    let schedules = state.list_schedules();
    assert_eq!(schedules[0].name, "Evening");
    assert_eq!(schedules[0].schedule_type, ScheduleType::Break);

    state.remove_schedule(sid);
    assert!(state.list_schedules().is_empty());
}

#[test]
fn list_rule_sets_returns_all() {
    let state = AppState::new(Config::default());
    state.add_rule_set(RuleSet::new("A"));
    state.add_rule_set(RuleSet::new("B"));
    assert_eq!(state.list_rule_sets().len(), 2);
}

#[test]
fn add_url_to_rule_set_returns_false_when_not_found() {
    let state = AppState::new(Config::default());
    assert!(!state.add_url_to_rule_set(Uuid::new_v4(), "x.com".into()));
}

#[test]
fn remove_url_from_rule_set_returns_false_when_not_found() {
    let state = AppState::new(Config::default());
    assert!(!state.remove_url_from_rule_set(Uuid::new_v4(), "x.com"));
}

#[test]
fn set_strict_mode_and_allow_new_tab() {
    let state = AppState::new(Config::default());
    state.set_strict_mode(true);
    assert!(state.config().strict_mode);
    state.set_allow_new_tab(false);
    assert!(!state.config().allow_new_tab);
}

#[test]
fn caldav_config_is_none_by_default_then_set() {
    let state = AppState::new(Config::default());
    assert!(state.caldav_config().is_none());
    state.set_caldav("https://cal.example.com".into(), "user".into(), "pass".into());
    let cfg = state.caldav_config().unwrap();
    assert_eq!(cfg.url, "https://cal.example.com");
    assert_eq!(cfg.username.as_deref(), Some("user"));
    assert_eq!(cfg.password.as_deref(), Some("pass"));
}

#[test]
fn set_default_rule_set_valid_id_returns_true() {
    let state = AppState::new(Config::default());
    let rs = RuleSet::new("X");
    let id = rs.id;
    state.add_rule_set(rs);
    assert!(state.set_default_rule_set(id));
    assert_eq!(state.effective_default_rule_set_id(), id);
}

#[test]
fn effective_default_rule_set_id_returns_nil_when_empty() {
    let state = AppState::new(Config::default());
    assert_eq!(state.effective_default_rule_set_id(), Uuid::nil());
}

#[test]
fn effective_default_rule_set_falls_back_to_first_when_default_removed() {
    let state = AppState::new(Config::default());
    let rs1 = RuleSet::new("A");
    let id1 = rs1.id;
    let rs2 = RuleSet::new("B");
    let id2 = rs2.id;
    state.add_rule_set(rs1);
    state.add_rule_set(rs2);
    state.set_default_rule_set(id2);
    // Remove the default (id2 → falls back to first = id1)
    state.remove_rule_set(id2);
    assert_eq!(state.effective_default_rule_set_id(), id1);
}

#[test]
fn effective_default_rule_set_id_filters_stale_default() {
    let state = AppState::new(Config::default());
    let rs = RuleSet::new("Only");
    let id = rs.id;
    state.add_rule_set(rs);
    state.set_default_rule_set(id);
    // Manually set a stale default (not in rule_sets) via Config
    // We test this by setting and removing the rule set
    state.remove_rule_set(id);
    // Now default_rule_set_id still points to id which no longer exists
    // effective_default_rule_set_id should return nil since rule_sets is empty
    assert_eq!(state.effective_default_rule_set_id(), Uuid::nil());
}

#[test]
fn snapshot_returns_correct_google_calendar_connected_flag() {
    let state = AppState::new(Config::default());
    assert!(!state.snapshot().google_calendar_connected);

    state.set_google_calendar_tokens("id".into(), "s".into(), "tok".into(), "ref".into(), 9999);
    assert!(state.snapshot().google_calendar_connected);
}

#[test]
fn snapshot_google_connected_false_when_access_token_missing() {
    let mut cfg = Config::default();
    cfg.google_calendar = Some(shared::models::GoogleCalendarConfig {
        client_id: "id".into(),
        client_secret: "s".into(),
        access_token: None,
        refresh_token: None,
        token_expiry_secs: None,
    });
    let state = AppState::new(cfg);
    assert!(!state.snapshot().google_calendar_connected);
}

#[test]
fn apply_schedule_starts_focus_when_active_focus_schedule() {
    use chrono::{Local, NaiveTime};
    let now = Local::now();
    let today = now.date_naive().weekday();
    let _time = now.naive_local().time();

    // Create a focus schedule that's active right now
    let start = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
    let end = NaiveTime::from_hms_opt(23, 59, 59).unwrap();

    let state = AppState::new(Config::default());
    let rs = RuleSet::new("Sched");
    let rs_id = rs.id;
    state.add_rule_set(rs);

    let s = Schedule {
        id: Uuid::new_v4(),
        name: "Always On".into(),
        days: vec![today],
        start,
        end,
        rule_set_id: rs_id,
        enabled: true,
        imported: false,
        imported_repeating: false,
        specific_date: None,
        schedule_type: ScheduleType::Focus,
    };
    state.add_schedule(s);
    state.apply_schedule();

    let snap = state.snapshot();
    assert!(snap.focus_active);
    assert_eq!(snap.active_rule_set_name.as_deref(), Some("Sched"));
}

#[test]
fn apply_schedule_does_not_change_manual_focus() {
    let state = AppState::new(Config::default());
    let rs = RuleSet::new("Manual");
    let id = rs.id;
    state.add_rule_set(rs);
    state.start_focus(id); // manually started, schedule_activated=false

    // No schedules → apply_schedule does nothing since !schedule_activated
    state.apply_schedule();
    assert!(state.snapshot().focus_active);
}

#[test]
fn apply_schedule_stops_schedule_activated_focus_when_no_active_schedule() {
    let state = AppState::new(Config::default());
    let rs = RuleSet::new("R");
    let rs_id = rs.id;
    state.add_rule_set(rs);

    // Simulate a schedule-activated focus by using a far-future specific_date schedule
    let s = Schedule {
        id: Uuid::new_v4(),
        name: "Past".into(),
        days: vec![chrono::Weekday::Mon],
        start: NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
        end: NaiveTime::from_hms_opt(0, 1, 0).unwrap(), // 1 minute only
        rule_set_id: rs_id,
        enabled: true,
        imported: false,
        imported_repeating: false,
        specific_date: Some(NaiveDate::from_ymd_opt(2000, 1, 1).unwrap()), // past date
        schedule_type: ScheduleType::Focus,
    };
    state.add_schedule(s);

    // Force schedule_activated state via apply_schedule with an always-active schedule first
    // then remove/disable it. Simplest: directly test the "schedule_activated=true + no active" path.
    // We do this by setting up a schedule active right now, applying it, then disabling it.
    use chrono::Local;
    let now = Local::now();
    let today = now.date_naive().weekday();
    let always_on = Schedule {
        id: Uuid::new_v4(),
        name: "AlwaysOn".into(),
        days: vec![today],
        start: NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
        end: NaiveTime::from_hms_opt(23, 59, 59).unwrap(),
        rule_set_id: rs_id,
        enabled: true,
        imported: false,
        imported_repeating: false,
        specific_date: None,
        schedule_type: ScheduleType::Focus,
    };
    let always_id = always_on.id;
    state.add_schedule(always_on);
    state.apply_schedule(); // activates focus with schedule_activated=true

    assert!(state.snapshot().focus_active);

    // Now remove the active schedule and call apply_schedule again
    state.remove_schedule(always_id);
    state.apply_schedule(); // should stop focus since schedule_activated=true but no active schedule

    assert!(!state.snapshot().focus_active);
}

#[test]
fn apply_schedule_break_wins_over_focus_when_both_active() {
    use chrono::Local;
    let now = Local::now();
    let today = now.date_naive().weekday();
    let start = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
    let end = NaiveTime::from_hms_opt(23, 59, 59).unwrap();

    let state = AppState::new(Config::default());
    let rs = RuleSet::new("R");
    let rs_id = rs.id;
    state.add_rule_set(rs);

    let focus_sched = Schedule {
        id: Uuid::new_v4(),
        name: "Focus".into(),
        days: vec![today],
        start,
        end,
        rule_set_id: rs_id,
        enabled: true,
        imported: false,
        imported_repeating: false,
        specific_date: None,
        schedule_type: ScheduleType::Focus,
    };
    let break_sched = Schedule {
        id: Uuid::new_v4(),
        name: "Break".into(),
        days: vec![today],
        start,
        end,
        rule_set_id: rs_id,
        enabled: true,
        imported: false,
        imported_repeating: false,
        specific_date: None,
        schedule_type: ScheduleType::Break,
    };
    state.add_schedule(focus_sched);
    state.add_schedule(break_sched);

    // First activate focus via schedule
    state.apply_schedule(); // break wins → focus should not start / be stopped
    assert!(!state.snapshot().focus_active);
}

#[test]
fn apply_schedule_break_stops_schedule_activated_focus() {
    use chrono::Local;
    let now = Local::now();
    let today = now.date_naive().weekday();

    let state = AppState::new(Config::default());
    let rs = RuleSet::new("R");
    let rs_id = rs.id;
    state.add_rule_set(rs);

    let focus_sched_id = Uuid::new_v4();
    let focus_sched = Schedule {
        id: focus_sched_id,
        name: "Focus".into(),
        days: vec![today],
        start: NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
        end: NaiveTime::from_hms_opt(23, 59, 59).unwrap(),
        rule_set_id: rs_id,
        enabled: true,
        imported: false,
        imported_repeating: false,
        specific_date: None,
        schedule_type: ScheduleType::Focus,
    };
    state.add_schedule(focus_sched);
    state.apply_schedule(); // focus activated by schedule
    assert!(state.snapshot().focus_active);

    // Now add a break schedule and re-apply
    let break_sched = Schedule {
        id: Uuid::new_v4(),
        name: "Break".into(),
        days: vec![today],
        start: NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
        end: NaiveTime::from_hms_opt(23, 59, 59).unwrap(),
        rule_set_id: rs_id,
        enabled: true,
        imported: false,
        imported_repeating: false,
        specific_date: None,
        schedule_type: ScheduleType::Break,
    };
    state.add_schedule(break_sched);
    state.apply_schedule(); // break wins → stops schedule-activated focus
    assert!(!state.snapshot().focus_active);
}

#[test]
fn update_google_tokens_no_op_when_no_google_config() {
    let state = AppState::new(Config::default());
    state.update_google_tokens("tok".into(), 999); // should not panic
    assert!(state.google_calendar_config().is_none());
}

#[test]
fn remove_import_rule_removes_matching_entry() {
    let state = AppState::new(Config::default());
    state.add_import_rule("standup".into(), ScheduleType::Focus);
    assert_eq!(state.list_import_rules().len(), 1);
    state.remove_import_rule("standup", &ScheduleType::Focus);
    assert!(state.list_import_rules().is_empty());
}

#[test]
fn snapshot_returns_no_seconds_remaining_when_no_pomodoro() {
    let state = AppState::new(Config::default());
    let snap = state.snapshot();
    assert!(!snap.pomodoro_active);
    assert!(snap.seconds_remaining.is_none());
    assert!(snap.pomodoro_phase.is_none());
}

#[test]
fn pomodoro_takes_priority_over_schedule() {
    use chrono::Local;
    let now = Local::now();
    let today = now.date_naive().weekday();

    let state = AppState::new(Config::default());
    let rs = RuleSet::new("SchedRS");
    let rs_id = rs.id;
    let pom_rs = RuleSet::new("PomRS");
    let pom_rs_id = pom_rs.id;
    state.add_rule_set(rs);
    state.add_rule_set(pom_rs);

    // Activate schedule first
    let sched = Schedule {
        id: Uuid::new_v4(),
        name: "Always".into(),
        days: vec![today],
        start: NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
        end: NaiveTime::from_hms_opt(23, 59, 59).unwrap(),
        rule_set_id: rs_id,
        enabled: true,
        imported: false,
        imported_repeating: false,
        specific_date: None,
        schedule_type: ScheduleType::Focus,
    };
    state.add_schedule(sched);
    state.apply_schedule();
    assert_eq!(state.snapshot().active_rule_set_name.as_deref(), Some("SchedRS"));

    // Start pomodoro: should override schedule's rule set
    state.start_pomodoro(3600, 300, Some(pom_rs_id));
    assert_eq!(state.snapshot().active_rule_set_name.as_deref(), Some("PomRS"));

    // apply_schedule while pomodoro is running: should be a no-op
    state.apply_schedule();
    assert_eq!(state.snapshot().active_rule_set_name.as_deref(), Some("PomRS"));

    // Stop pomodoro: focus resets; next apply_schedule restores the schedule
    state.stop_pomodoro();
    assert!(!state.snapshot().focus_active);
    state.apply_schedule();
    assert_eq!(state.snapshot().active_rule_set_name.as_deref(), Some("SchedRS"));
}

#[test]
fn tick_sets_focus_inactive_during_break_phase() {
    use std::time::Duration;
    let state = AppState::new(Config::default());
    state.start_pomodoro(1, 3600, None); // 1-second focus

    std::thread::sleep(Duration::from_millis(1100));
    state.tick(); // advance to Break phase

    let snap = state.snapshot();
    assert_eq!(snap.pomodoro_phase.as_ref().map(|p| format!("{p:?}")), Some("Break".into()));
    // During break, blocking should be inactive
    assert!(!snap.focus_active);
}
