use crate::pomodoro::PomodoroTimer;
use shared::{
    ipc::{DaemonEvent, ImportRuleSummary, RuleSetSummary, ScheduleSummary},
    models::{Config, RuleSet},
};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use uuid::Uuid;

#[derive(Debug)]
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
    /// Open browser tabs pushed by the extension — ephemeral, never persisted.
    open_tabs: Vec<(String, String)>,
    event_tx: broadcast::Sender<DaemonEvent>,
}

#[derive(Debug, Clone)]
pub struct AppState(Arc<Mutex<Inner>>);

impl AppState {
    pub fn new(config: Config) -> Self {
        let (event_tx, _) = broadcast::channel(256);
        Self(Arc::new(Mutex::new(Inner {
            config,
            event_tx,
            focus_active: false,
            active_rule_set_id: None,
            pomodoro: None,
            pending_oauth_state: None,
            pending_google_client_id: None,
            pending_google_client_secret: None,
            schedule_activated: false,
            open_tabs: Vec::new(),
        })))
    }

    /// Acquire the inner lock, recovering gracefully if it is poisoned.
    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.0.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Subscribe to push events from the daemon.
    pub fn subscribe(&self) -> broadcast::Receiver<DaemonEvent> {
        self.lock().event_tx.subscribe()
    }

    /// Emit an event to all subscribers. Safe to call after dropping the Inner lock.
    fn emit(&self, event: DaemonEvent) {
        let tx = self.lock().event_tx.clone();
        let _ = tx.send(event);
    }

    // ── Event payload builders (each acquires the lock briefly) ──────────────

    fn focus_event(&self) -> DaemonEvent {
        let inner = self.lock();
        let rule_set_name = inner
            .active_rule_set_id
            .and_then(|id| inner.config.rule_sets.iter().find(|r| r.id == id))
            .map(|r| r.name.clone());
        DaemonEvent::FocusChanged {
            active: inner.focus_active,
            rule_set_name,
        }
    }

    fn config_event(&self) -> DaemonEvent {
        let inner = self.lock();
        DaemonEvent::ConfigChanged {
            strict_mode: inner.config.strict_mode,
            allow_new_tab: inner.config.allow_new_tab,
            accent_color: inner.config.accent_color.clone(),
            google_calendar_connected: inner
                .config
                .google_calendar
                .as_ref()
                .map(|c| c.access_token.is_some())
                .unwrap_or(false),
            caldav_url: inner.config.caldav.as_ref().map(|c| c.url.clone()),
            default_rule_set_id: inner.config.default_rule_set_id,
        }
    }

    fn rule_sets_event(&self) -> DaemonEvent {
        let inner = self.lock();
        let rule_sets = inner
            .config
            .rule_sets
            .iter()
            .map(|rs| RuleSetSummary {
                id: rs.id,
                name: rs.name.clone(),
                allowed_urls: rs.allowed_urls.clone(),
            })
            .collect();
        DaemonEvent::RuleSetsChanged { rule_sets }
    }

    fn schedules_event(&self) -> DaemonEvent {
        use chrono::Timelike;
        let inner = self.lock();
        let schedules = inner
            .config
            .schedules
            .iter()
            .map(|s| ScheduleSummary {
                id: s.id,
                name: s.name.clone(),
                days: s
                    .days
                    .iter()
                    .map(|d| d.num_days_from_monday() as u8)
                    .collect(),
                start_min: s.start.hour() * 60 + s.start.minute(),
                end_min: s.end.hour() * 60 + s.end.minute(),
                enabled: s.enabled,
                imported: s.imported,
                imported_repeating: s.imported_repeating,
                specific_date: s.specific_date.map(|d| d.format("%Y-%m-%d").to_string()),
                schedule_type: s.schedule_type.clone(),
                rule_set_id: s.rule_set_id,
            })
            .collect();
        DaemonEvent::SchedulesChanged { schedules }
    }

    fn import_rules_event(&self) -> DaemonEvent {
        let inner = self.lock();
        let rules = inner
            .config
            .import_rules
            .iter()
            .map(|r| ImportRuleSummary {
                keyword: r.keyword.clone(),
                schedule_type: r.schedule_type.clone(),
            })
            .collect();
        DaemonEvent::ImportRulesChanged { rules }
    }

    // ── Mutations ─────────────────────────────────────────────────────────────

    pub fn start_focus(&self, rule_set_id: Uuid) {
        {
            let mut inner = self.lock();
            inner.focus_active = true;
            inner.active_rule_set_id = Some(rule_set_id);
            inner.schedule_activated = false;
        }
        self.emit(self.focus_event());
    }

    pub fn stop_focus(&self) {
        {
            let mut inner = self.lock();
            inner.focus_active = false;
            inner.active_rule_set_id = None;
            inner.pomodoro = None;
            inner.schedule_activated = false;
        }
        self.emit(self.focus_event());
        self.emit(DaemonEvent::PomodoroTick {
            phase: None,
            seconds_remaining: None,
        });
    }

    pub fn start_pomodoro(&self, focus_secs: u64, break_secs: u64, rule_set_id: Option<Uuid>) {
        let pom_state = {
            let mut inner = self.lock();
            inner.focus_active = true;
            inner.active_rule_set_id = rule_set_id;
            let pom = PomodoroTimer::new(focus_secs, break_secs);
            let secs = pom.seconds_remaining();
            inner.pomodoro = Some(pom);
            (Some(shared::ipc::PomodoroPhase::Focus), Some(secs))
        };
        self.emit(self.focus_event());
        self.emit(DaemonEvent::PomodoroTick {
            phase: pom_state.0,
            seconds_remaining: pom_state.1,
        });
    }

    pub fn stop_pomodoro(&self) {
        {
            let mut inner = self.lock();
            inner.pomodoro = None;
            inner.focus_active = false;
            inner.active_rule_set_id = None;
        }
        self.emit(self.focus_event());
        self.emit(DaemonEvent::PomodoroTick {
            phase: None,
            seconds_remaining: None,
        });
    }

    pub fn active_rule_set(&self) -> Option<RuleSet> {
        let inner = self.lock();
        let id = inner.active_rule_set_id?;
        inner.config.rule_sets.iter().find(|r| r.id == id).cloned()
    }

    #[allow(dead_code)]
    pub fn config(&self) -> Config {
        self.lock().config.clone()
    }

    pub fn add_rule_set(&self, rule_set: RuleSet) {
        {
            let mut inner = self.lock();
            let id = rule_set.id;
            inner.config.rule_sets.push(rule_set);
            if inner.config.default_rule_set_id.is_none() {
                inner.config.default_rule_set_id = Some(id);
            }
        }
        self.emit(self.rule_sets_event());
        self.emit(self.config_event());
    }

    /// Called every second by the background tick loop.
    /// Advances the pomodoro phase when the current phase expires.
    pub fn tick(&self) {
        let pom_state = {
            let mut inner = self.lock();
            // Extract values from pom before touching other inner fields (borrow discipline).
            let (is_active, is_focus, phase, secs) = match &mut inner.pomodoro {
                Some(pom) => {
                    if pom.is_expired() {
                        pom.advance();
                    }
                    let is_focus = pom.phase == crate::pomodoro::Phase::Focus;
                    let phase = Some(match pom.phase {
                        crate::pomodoro::Phase::Focus => shared::ipc::PomodoroPhase::Focus,
                        crate::pomodoro::Phase::Break => shared::ipc::PomodoroPhase::Break,
                    });
                    let secs = Some(pom.seconds_remaining());
                    (true, is_focus, phase, secs)
                }
                None => (false, false, None, None),
            };
            if is_active {
                inner.focus_active = is_focus;
            }
            is_active.then_some((phase, secs))
        }; // lock dropped here
        if let Some((phase, seconds_remaining)) = pom_state {
            self.emit(DaemonEvent::PomodoroTick {
                phase,
                seconds_remaining,
            });
        }
    }

    /// Skip the current break and return to Focus immediately.
    /// Returns false if strict_breaks is enabled.
    pub fn skip_break(&self) -> bool {
        let result = {
            let mut inner = self.lock();
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
        };
        if result {
            self.emit(self.focus_event());
        }
        result
    }

    pub fn set_open_tabs(&self, tabs: Vec<(String, String)>) {
        self.lock().open_tabs = tabs;
    }

    pub fn get_open_tabs(&self) -> Vec<(String, String)> {
        self.lock().open_tabs.clone()
    }

    pub fn remove_rule_set(&self, id: Uuid) {
        {
            let mut inner = self.lock();
            inner.config.rule_sets.retain(|r| r.id != id);
            if inner.config.default_rule_set_id == Some(id) {
                inner.config.default_rule_set_id =
                    inner.config.rule_sets.first().map(|r| r.id);
            }
        }
        self.emit(self.rule_sets_event());
        self.emit(self.config_event());
    }

    pub fn set_default_rule_set(&self, id: Uuid) -> bool {
        let changed = {
            let mut inner = self.lock();
            if inner.config.rule_sets.iter().any(|r| r.id == id) {
                inner.config.default_rule_set_id = Some(id);
                true
            } else {
                false
            }
        };
        if changed {
            self.emit(self.config_event());
        }
        changed
    }

    pub fn effective_default_rule_set_id(&self) -> Uuid {
        let inner = self.lock();
        inner
            .config
            .default_rule_set_id
            .filter(|id| inner.config.rule_sets.iter().any(|r| r.id == *id))
            .or_else(|| inner.config.rule_sets.first().map(|r| r.id))
            .unwrap_or_else(Uuid::nil)
    }

    pub fn list_rule_sets(&self) -> Vec<shared::models::RuleSet> {
        self.lock().config.rule_sets.clone()
    }

    pub fn list_schedules(&self) -> Vec<shared::models::Schedule> {
        self.lock().config.schedules.clone()
    }

    pub fn add_schedule(&self, schedule: shared::models::Schedule) {
        self.lock().config.schedules.push(schedule);
        self.emit(self.schedules_event());
    }

    pub fn remove_schedule(&self, id: Uuid) {
        self.lock().config.schedules.retain(|s| s.id != id);
        self.emit(self.schedules_event());
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
        {
            let mut inner = self.lock();
            if let Some(s) = inner.config.schedules.iter_mut().find(|s| s.id == id) {
                s.name = name;
                s.days = days;
                s.start = start;
                s.end = end;
                s.rule_set_id = rule_set_id.unwrap_or_else(Uuid::nil);
                s.specific_date = new_specific_date;
                s.schedule_type = schedule_type;
            }
        }
        self.emit(self.schedules_event());
    }

    /// Check active schedules and start/stop focus accordingly.
    /// Called periodically by the background scheduler loop.
    pub fn apply_schedule(&self) {
        let focus_changed = {
            let mut inner = self.lock();

            if inner.pomodoro.is_some() {
                return;
            }

            let prev_active = inner.focus_active;
            let prev_rule_set = inner.active_rule_set_id;

            let (active_focus, has_break) =
                inner
                    .config
                    .schedules
                    .iter()
                    .filter(|s| s.is_active_now())
                    .fold((None, false), |(focus, brk), s| {
                        match s.schedule_type {
                            shared::models::ScheduleType::Focus if focus.is_none() => {
                                (Some(s.rule_set_id), brk)
                            }
                            shared::models::ScheduleType::Break => (focus, true),
                            _ => (focus, brk),
                        }
                    });

            if has_break {
                if inner.schedule_activated {
                    inner.focus_active = false;
                    inner.active_rule_set_id = None;
                    inner.schedule_activated = false;
                }
            } else if let Some(rule_set_id) = active_focus {
                if !inner.focus_active || inner.active_rule_set_id != Some(rule_set_id) {
                    inner.focus_active = true;
                    inner.active_rule_set_id = Some(rule_set_id);
                    inner.schedule_activated = true;
                }
            } else if inner.schedule_activated {
                inner.focus_active = false;
                inner.active_rule_set_id = None;
                inner.schedule_activated = false;
            }

            inner.focus_active != prev_active || inner.active_rule_set_id != prev_rule_set
        }; // lock dropped

        if focus_changed {
            self.emit(self.focus_event());
        }
    }

    pub fn remove_url_from_rule_set(&self, rule_set_id: Uuid, url: &str) -> bool {
        let changed = {
            let mut inner = self.lock();
            if let Some(rs) = inner
                .config
                .rule_sets
                .iter_mut()
                .find(|r| r.id == rule_set_id)
            {
                rs.allowed_urls.retain(|u| u != url);
                true
            } else {
                false
            }
        };
        if changed {
            self.emit(self.rule_sets_event());
        }
        changed
    }

    pub fn add_url_to_rule_set(&self, rule_set_id: Uuid, url: String) -> bool {
        let changed = {
            let mut inner = self.lock();
            if let Some(rs) = inner
                .config
                .rule_sets
                .iter_mut()
                .find(|r| r.id == rule_set_id)
            {
                if !rs.allowed_urls.contains(&url) {
                    rs.allowed_urls.push(url);
                }
                true
            } else {
                false
            }
        };
        if changed {
            self.emit(self.rule_sets_event());
        }
        changed
    }

    /// Replace calendar-imported schedules with a fresh set.
    pub fn apply_calendar_schedules(&self, imported: Vec<shared::models::Schedule>) {
        {
            let mut inner = self.lock();
            inner.config.schedules.retain(|s| !s.imported);
            inner.config.schedules.extend(imported);
        }
        self.emit(self.schedules_event());
    }

    pub fn caldav_config(&self) -> Option<shared::models::CalDavConfig> {
        self.lock().config.caldav.clone()
    }

    pub fn list_import_rules(&self) -> Vec<shared::models::CalendarImportRule> {
        self.lock().config.import_rules.clone()
    }

    pub fn add_import_rule(&self, keyword: String, schedule_type: shared::models::ScheduleType) {
        let keyword = keyword.to_lowercase();
        {
            let mut inner = self.lock();
            let exists = inner
                .config
                .import_rules
                .iter()
                .any(|r| r.keyword == keyword && r.schedule_type == schedule_type);
            if !exists {
                inner
                    .config
                    .import_rules
                    .push(shared::models::CalendarImportRule {
                        keyword,
                        schedule_type,
                        rule_set_id: None,
                    });
            }
        }
        self.emit(self.import_rules_event());
    }

    pub fn remove_import_rule(
        &self,
        keyword: &str,
        schedule_type: &shared::models::ScheduleType,
    ) {
        let keyword = keyword.to_lowercase();
        {
            let mut inner = self.lock();
            inner
                .config
                .import_rules
                .retain(|r| !(r.keyword == keyword && &r.schedule_type == schedule_type));
        }
        self.emit(self.import_rules_event());
    }

    pub fn set_strict_mode(&self, enabled: bool) {
        self.lock().config.strict_mode = enabled;
        self.emit(self.config_event());
    }

    pub fn set_allow_new_tab(&self, enabled: bool) {
        self.lock().config.allow_new_tab = enabled;
        self.emit(self.config_event());
    }

    pub fn set_accent_color(&self, hex: String) {
        self.lock().config.accent_color = hex;
        self.emit(self.config_event());
    }

    // ── Google Calendar OAuth2 ────────────────────────────────────────────────

    pub fn set_pending_oauth_state(
        &self,
        state_token: String,
        client_id: String,
        client_secret: String,
    ) {
        let mut inner = self.lock();
        inner.pending_oauth_state = Some(state_token);
        inner.pending_google_client_id = Some(client_id);
        inner.pending_google_client_secret = Some(client_secret);
    }

    /// Validate and consume the CSRF token. Returns the stored credentials if valid.
    pub fn take_pending_oauth(&self, state_token: &str) -> Option<(String, String)> {
        let mut inner = self.lock();
        let stored = inner.pending_oauth_state.take()?;
        if stored != state_token {
            return None;
        }
        match (
            inner.pending_google_client_id.take(),
            inner.pending_google_client_secret.take(),
        ) {
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
        {
            let mut inner = self.lock();
            inner.config.google_calendar = Some(shared::models::GoogleCalendarConfig {
                client_id,
                client_secret,
                access_token: Some(access_token),
                refresh_token: Some(refresh_token),
                token_expiry_secs: Some(expiry_secs),
            });
        }
        self.emit(self.config_event());
    }

    pub fn update_google_tokens(&self, access_token: String, expiry_secs: i64) {
        {
            let mut inner = self.lock();
            if let Some(cfg) = &mut inner.config.google_calendar {
                cfg.access_token = Some(access_token);
                cfg.token_expiry_secs = Some(expiry_secs);
            }
        }
        self.emit(self.config_event());
    }

    pub fn revoke_google_calendar(&self) {
        self.lock().config.google_calendar = None;
        self.emit(self.config_event());
    }

    pub fn google_calendar_config(&self) -> Option<shared::models::GoogleCalendarConfig> {
        self.lock().config.google_calendar.clone()
    }

    pub fn set_caldav(&self, url: String, username: String, password: String) {
        {
            let mut inner = self.lock();
            inner.config.caldav = Some(shared::models::CalDavConfig {
                url,
                username: Some(username),
                password: Some(password),
            });
        }
        self.emit(self.config_event());
    }

    pub fn snapshot(&self) -> StateSnapshot {
        let inner = self.lock();
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

        let google_calendar_connected = inner
            .config
            .google_calendar
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
            caldav_url: inner.config.caldav.as_ref().map(|c| c.url.clone()),
            default_rule_set_id: inner.config.default_rule_set_id,
            accent_color: inner.config.accent_color.clone(),
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
    pub caldav_url: Option<String>,
    pub default_rule_set_id: Option<Uuid>,
    pub accent_color: String,
}

#[cfg(test)]
#[path = "app_state_tests.rs"]
mod tests;
