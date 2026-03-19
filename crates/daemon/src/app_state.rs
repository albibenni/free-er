use crate::pomodoro::PomodoroTimer;
use shared::models::{Config, RuleSet};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Debug, Default)]
struct Inner {
    focus_active: bool,
    active_rule_set_id: Option<Uuid>,
    pomodoro: Option<PomodoroTimer>,
    config: Config,
    // Ephemeral OAuth2 state (not persisted)
    pending_oauth_state: Option<String>,
    pending_google_client_id: Option<String>,
    pending_google_client_secret: Option<String>,
    /// True when the current focus session was started automatically by a schedule.
    schedule_activated: bool,
}

#[derive(Debug, Clone)]
pub struct AppState(Arc<Mutex<Inner>>);

impl AppState {
    pub fn new(config: Config) -> Self {
        Self(Arc::new(Mutex::new(Inner {
            config,
            ..Default::default()
        })))
    }

    pub fn start_focus(&self, rule_set_id: Uuid) {
        let mut inner = self.0.lock().unwrap();
        inner.focus_active = true;
        inner.active_rule_set_id = Some(rule_set_id);
        inner.schedule_activated = false;
    }

    pub fn stop_focus(&self) {
        let mut inner = self.0.lock().unwrap();
        inner.focus_active = false;
        inner.active_rule_set_id = None;
        inner.pomodoro = None;
        inner.schedule_activated = false;
    }

    pub fn start_pomodoro(&self, focus_secs: u64, break_secs: u64, rule_set_id: Option<Uuid>) {
        let mut inner = self.0.lock().unwrap();
        inner.focus_active = true;
        inner.active_rule_set_id = rule_set_id;
        inner.pomodoro = Some(PomodoroTimer::new(focus_secs, break_secs));
    }

    pub fn stop_pomodoro(&self) {
        let mut inner = self.0.lock().unwrap();
        inner.pomodoro = None;
    }

    pub fn active_rule_set(&self) -> Option<RuleSet> {
        let inner = self.0.lock().unwrap();
        let id = inner.active_rule_set_id?;
        inner.config.rule_sets.iter().find(|r| r.id == id).cloned()
    }

    #[allow(dead_code)]
    pub fn config(&self) -> Config {
        self.0.lock().unwrap().config.clone()
    }

    pub fn add_rule_set(&self, rule_set: RuleSet) {
        let mut inner = self.0.lock().unwrap();
        let id = rule_set.id;
        inner.config.rule_sets.push(rule_set);
        if inner.config.default_rule_set_id.is_none() {
            inner.config.default_rule_set_id = Some(id);
        }
    }

    /// Called every second by the background tick loop.
    /// Advances the pomodoro phase when the current phase expires.
    pub fn tick(&self) {
        let mut inner = self.0.lock().unwrap();
        if let Some(pom) = &mut inner.pomodoro {
            if pom.is_expired() {
                pom.advance();
                // When moving back to Focus, ensure focus_active stays true.
                inner.focus_active = true;
            }
        }
    }

    /// Skip the current break and return to Focus immediately.
    /// Returns false if strict_breaks is enabled.
    pub fn skip_break(&self) -> bool {
        let mut inner = self.0.lock().unwrap();
        if inner.config.pomodoro.strict_breaks {
            return false;
        }
        if let Some(pom) = &mut inner.pomodoro {
            if pom.phase == crate::pomodoro::Phase::Break {
                pom.advance();
                inner.focus_active = true;
            }
        }
        true
    }

    pub fn remove_rule_set(&self, id: Uuid) {
        let mut inner = self.0.lock().unwrap();
        inner.config.rule_sets.retain(|r| r.id != id);
        if inner.config.default_rule_set_id == Some(id) {
            inner.config.default_rule_set_id = inner.config.rule_sets.first().map(|r| r.id);
        }
    }

    pub fn set_default_rule_set(&self, id: Uuid) -> bool {
        let mut inner = self.0.lock().unwrap();
        if inner.config.rule_sets.iter().any(|r| r.id == id) {
            inner.config.default_rule_set_id = Some(id);
            true
        } else {
            false
        }
    }

    pub fn effective_default_rule_set_id(&self) -> Uuid {
        let inner = self.0.lock().unwrap();
        inner
            .config
            .default_rule_set_id
            .filter(|id| inner.config.rule_sets.iter().any(|r| r.id == *id))
            .or_else(|| inner.config.rule_sets.first().map(|r| r.id))
            .unwrap_or_else(Uuid::nil)
    }

    pub fn list_rule_sets(&self) -> Vec<shared::models::RuleSet> {
        self.0.lock().unwrap().config.rule_sets.clone()
    }

    pub fn list_schedules(&self) -> Vec<shared::models::Schedule> {
        self.0.lock().unwrap().config.schedules.clone()
    }

    pub fn add_schedule(&self, schedule: shared::models::Schedule) {
        self.0.lock().unwrap().config.schedules.push(schedule);
    }

    pub fn remove_schedule(&self, id: Uuid) {
        self.0.lock().unwrap().config.schedules.retain(|s| s.id != id);
    }

    pub fn update_schedule(
        &self,
        id: Uuid,
        name: String,
        days: Vec<chrono::Weekday>,
        start: chrono::NaiveTime,
        end: chrono::NaiveTime,
        rule_set_id: Option<Uuid>,
        new_specific_date: Option<chrono::NaiveDate>,
        schedule_type: shared::models::ScheduleType,
    ) {
        let mut inner = self.0.lock().unwrap();
        if let Some(s) = inner.config.schedules.iter_mut().find(|s| s.id == id) {
            s.name = name;
            s.days = days;
            s.start = start;
            s.end = end;
            // Always apply allowed-list changes; `None` means clear selection.
            s.rule_set_id = rule_set_id.unwrap_or_else(Uuid::nil);
            s.specific_date = new_specific_date;
            s.schedule_type = schedule_type;
        }
    }

    /// Check active schedules and start/stop focus accordingly.
    /// Called periodically by the background scheduler loop.
    pub fn apply_schedule(&self) {
        let mut inner = self.0.lock().unwrap();

        // Find the first active Focus schedule
        let active_focus = inner.config.schedules.iter()
            .filter(|s| s.schedule_type == shared::models::ScheduleType::Focus && s.is_active_now())
            .map(|s| s.rule_set_id)
            .next();

        // Find any active Break schedule
        let has_break = inner.config.schedules.iter()
            .any(|s| s.schedule_type == shared::models::ScheduleType::Break && s.is_active_now());

        if has_break {
            // Break wins: stop focus if we started it.
            if inner.schedule_activated {
                inner.focus_active = false;
                inner.active_rule_set_id = None;
                inner.schedule_activated = false;
            }
        } else if let Some(rule_set_id) = active_focus {
            // Start focus (or update rule set) if not already schedule-activated with same rule set.
            if !inner.focus_active || inner.active_rule_set_id != Some(rule_set_id) {
                inner.focus_active = true;
                inner.active_rule_set_id = Some(rule_set_id);
                inner.schedule_activated = true;
            }
        } else if inner.schedule_activated {
            // No active schedule — stop if we were the ones who started it.
            inner.focus_active = false;
            inner.active_rule_set_id = None;
            inner.schedule_activated = false;
        }
    }

    pub fn remove_url_from_rule_set(&self, rule_set_id: Uuid, url: &str) -> bool {
        let mut inner = self.0.lock().unwrap();
        if let Some(rs) = inner.config.rule_sets.iter_mut().find(|r| r.id == rule_set_id) {
            rs.allowed_urls.retain(|u| u != url);
            true
        } else {
            false
        }
    }

    pub fn add_url_to_rule_set(&self, rule_set_id: Uuid, url: String) -> bool {
        let mut inner = self.0.lock().unwrap();
        if let Some(rs) = inner.config.rule_sets.iter_mut().find(|r| r.id == rule_set_id) {
            if !rs.allowed_urls.contains(&url) {
                rs.allowed_urls.push(url);
            }
            true
        } else {
            false
        }
    }

    /// Replace calendar-imported schedules with a fresh set.
    /// Non-imported (manually created) schedules are left untouched.
    pub fn apply_calendar_schedules(&self, imported: Vec<shared::models::Schedule>) {
        let mut inner = self.0.lock().unwrap();
        // Replace the imported slice atomically so stale imported entries
        // (including those outside the visible week window) are eliminated.
        inner.config.schedules.retain(|s| !s.imported);
        inner.config.schedules.extend(imported);
    }

    pub fn caldav_config(&self) -> Option<shared::models::CalDavConfig> {
        self.0.lock().unwrap().config.caldav.clone()
    }

    pub fn list_import_rules(&self) -> Vec<shared::models::CalendarImportRule> {
        self.0.lock().unwrap().config.import_rules.clone()
    }

    pub fn add_import_rule(&self, keyword: String, schedule_type: shared::models::ScheduleType) {
        let keyword = keyword.to_lowercase();
        let mut inner = self.0.lock().unwrap();
        // Avoid duplicates (comparison is already on lowercased keyword)
        let exists = inner.config.import_rules.iter().any(|r| {
            r.keyword == keyword && r.schedule_type == schedule_type
        });
        if !exists {
            inner.config.import_rules.push(shared::models::CalendarImportRule {
                keyword,
                schedule_type,
                rule_set_id: None,
            });
        }
    }

    pub fn remove_import_rule(&self, keyword: &str, schedule_type: &shared::models::ScheduleType) {
        let keyword = keyword.to_lowercase();
        let mut inner = self.0.lock().unwrap();
        inner.config.import_rules.retain(|r| {
            !(r.keyword == keyword && &r.schedule_type == schedule_type)
        });
    }

    pub fn set_strict_mode(&self, enabled: bool) {
        self.0.lock().unwrap().config.strict_mode = enabled;
    }

    pub fn set_allow_new_tab(&self, enabled: bool) {
        self.0.lock().unwrap().config.allow_new_tab = enabled;
    }

    // ── Google Calendar OAuth2 ────────────────────────────────────────────────

    pub fn set_pending_oauth_state(&self, state_token: String, client_id: String, client_secret: String) {
        let mut inner = self.0.lock().unwrap();
        inner.pending_oauth_state = Some(state_token);
        inner.pending_google_client_id = Some(client_id);
        inner.pending_google_client_secret = Some(client_secret);
    }

    /// Validate and consume the CSRF token. Returns the stored credentials if valid.
    pub fn take_pending_oauth(&self, state_token: &str) -> Option<(String, String)> {
        let mut inner = self.0.lock().unwrap();
        let stored = inner.pending_oauth_state.take()?;
        if stored != state_token {
            return None;
        }
        match (inner.pending_google_client_id.take(), inner.pending_google_client_secret.take()) {
            (Some(id), Some(secret)) => Some((id, secret)),
            _ => None,
        }
    }

    pub fn set_google_calendar_tokens(
        &self,
        client_id: String,
        client_secret: String,
        access_token: String,
        refresh_token: String,
        expiry_secs: i64,
    ) {
        let mut inner = self.0.lock().unwrap();
        inner.config.google_calendar = Some(shared::models::GoogleCalendarConfig {
            client_id,
            client_secret,
            access_token: Some(access_token),
            refresh_token: Some(refresh_token),
            token_expiry_secs: Some(expiry_secs),
        });
    }

    pub fn update_google_tokens(&self, access_token: String, expiry_secs: i64) {
        let mut inner = self.0.lock().unwrap();
        if let Some(cfg) = &mut inner.config.google_calendar {
            cfg.access_token = Some(access_token);
            cfg.token_expiry_secs = Some(expiry_secs);
        }
    }

    pub fn revoke_google_calendar(&self) {
        self.0.lock().unwrap().config.google_calendar = None;
    }

    pub fn google_calendar_config(&self) -> Option<shared::models::GoogleCalendarConfig> {
        self.0.lock().unwrap().config.google_calendar.clone()
    }

    pub fn set_caldav(&self, url: String, username: String, password: String) {
        let mut inner = self.0.lock().unwrap();
        inner.config.caldav = Some(shared::models::CalDavConfig {
            url,
            username: Some(username),
            password: Some(password),
        });
    }

    pub fn snapshot(&self) -> StateSnapshot {
        let inner = self.0.lock().unwrap();
        let active_rule_set_name = inner
            .active_rule_set_id
            .and_then(|id| inner.config.rule_sets.iter().find(|r| r.id == id))
            .map(|r| r.name.clone());

        let (pomodoro_active, pomodoro_phase, seconds_remaining) =
            if let Some(pom) = &inner.pomodoro {
                (true, Some(pom.phase.clone()), Some(pom.seconds_remaining()))
            } else {
                (false, None, None)
            };

        let google_calendar_connected = inner.config.google_calendar
            .as_ref()
            .map(|c| c.access_token.is_some())
            .unwrap_or(false);

        StateSnapshot {
            focus_active: inner.focus_active,
            strict_mode: inner.config.strict_mode,
            allow_new_tab: inner.config.allow_new_tab,
            active_rule_set_name,
            pomodoro_active,
            pomodoro_phase,
            seconds_remaining,
            google_calendar_connected,
            default_rule_set_id: inner.config.default_rule_set_id,
        }
    }
}

pub struct StateSnapshot {
    pub focus_active: bool,
    pub strict_mode: bool,
    pub allow_new_tab: bool,
    pub active_rule_set_name: Option<String>,
    pub pomodoro_active: bool,
    pub pomodoro_phase: Option<crate::pomodoro::Phase>,
    pub seconds_remaining: Option<u64>,
    pub google_calendar_connected: bool,
    pub default_rule_set_id: Option<Uuid>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, NaiveTime, Weekday};
    use shared::models::{Schedule, ScheduleType};

    fn sample_schedule(
        id: Uuid,
        name: &str,
        imported: bool,
        imported_repeating: bool,
    ) -> Schedule {
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
        assert!(rules.iter().any(|r| r.keyword == "deep work" && r.schedule_type == ScheduleType::Focus));
        assert!(rules.iter().any(|r| r.keyword == "deep work" && r.schedule_type == ScheduleType::Break));
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
}
