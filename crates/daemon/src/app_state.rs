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
        self.0.lock().unwrap().config.rule_sets.push(rule_set);
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
        new_specific_date: Option<chrono::NaiveDate>, // Some → overwrite; None → leave unchanged

        schedule_type: shared::models::ScheduleType,
    ) {
        let mut inner = self.0.lock().unwrap();
        if let Some(s) = inner.config.schedules.iter_mut().find(|s| s.id == id) {
            s.name = name;
            s.days = days;
            s.start = start;
            s.end = end;
            if let Some(rsid) = rule_set_id {
                s.rule_set_id = rsid;
            }
            if let Some(date) = new_specific_date {
                s.specific_date = Some(date);
            }
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

        if let Some(rule_set_id) = active_focus {
            // Start focus (or update rule set) if not already schedule-activated with same rule set
            if !inner.focus_active || inner.active_rule_set_id != Some(rule_set_id) {
                inner.focus_active = true;
                inner.active_rule_set_id = Some(rule_set_id);
                inner.schedule_activated = true;
            }
        } else if has_break || !inner.schedule_activated {
            // Only auto-stop if we were the ones who started it
            if inner.schedule_activated {
                inner.focus_active = false;
                inner.active_rule_set_id = None;
                inner.schedule_activated = false;
            }
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
        // Remove previously imported schedules (identified by a naming convention
        // or, in the future, a dedicated `source` field). For now we replace all
        // schedules that share a name with an incoming one.
        let incoming_names: std::collections::HashSet<_> =
            imported.iter().map(|s| s.name.clone()).collect();
        inner
            .config
            .schedules
            .retain(|s| !incoming_names.contains(&s.name));
        inner.config.schedules.extend(imported);
    }

    pub fn caldav_config(&self) -> Option<shared::models::CalDavConfig> {
        self.0.lock().unwrap().config.caldav.clone()
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
        let import_rules = inner.config.google_calendar
            .as_ref()
            .map(|c| c.import_rules.clone())
            .unwrap_or_default();
        inner.config.google_calendar = Some(shared::models::GoogleCalendarConfig {
            client_id,
            client_secret,
            access_token: Some(access_token),
            refresh_token: Some(refresh_token),
            token_expiry_secs: Some(expiry_secs),
            import_rules,
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
            import_rules: inner
                .config
                .caldav
                .as_ref()
                .map(|c| c.import_rules.clone())
                .unwrap_or_default(),
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
}
