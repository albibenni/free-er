use crate::pomodoro::PomodoroTimer;
use shared::{
    ipc::{
        DaemonEvent, ImportRuleSummary, PomodoroPhase, RuleSetSummary, ScheduleSummary,
        StatusResponse,
    },
    models::{Config, RuleSet},
};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use tracing::warn;
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
        self.0.lock().unwrap_or_else(|e| {
            warn!("AppState mutex was poisoned — recovering from panic in critical section");
            e.into_inner()
        })
    }

    /// Subscribe to push events from the daemon.
    pub fn subscribe(&self) -> broadcast::Receiver<DaemonEvent> {
        self.lock().event_tx.subscribe()
    }

    /// Send an event to all subscribers. Caller must already hold the sender.
    fn emit(tx: &broadcast::Sender<DaemonEvent>, event: DaemonEvent) {
        let _ = tx.send(event);
    }

    // ── Event payload builders (static, operate on an already-held Inner) ─────

    fn focus_event(inner: &Inner) -> DaemonEvent {
        let rule_set_name = inner
            .active_rule_set_id
            .and_then(|id| inner.config.rule_sets.iter().find(|r| r.id == id))
            .map(|r| r.name.clone());
        DaemonEvent::FocusChanged {
            active: inner.focus_active,
            rule_set_name,
        }
    }

    fn config_event(inner: &Inner) -> DaemonEvent {
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

    fn rule_set_summaries(inner: &Inner) -> Vec<RuleSetSummary> {
        inner
            .config
            .rule_sets
            .iter()
            .map(|rs| RuleSetSummary {
                id: rs.id,
                name: rs.name.clone(),
                allowed_urls: rs.allowed_urls.clone(),
            })
            .collect()
    }

    fn schedule_summaries(inner: &Inner) -> Vec<ScheduleSummary> {
        use chrono::Timelike;
        inner
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
            .collect()
    }

    fn rule_sets_event(inner: &Inner) -> DaemonEvent {
        DaemonEvent::RuleSetsChanged {
            rule_sets: Self::rule_set_summaries(inner),
        }
    }

    fn schedules_event(inner: &Inner) -> DaemonEvent {
        DaemonEvent::SchedulesChanged {
            schedules: Self::schedule_summaries(inner),
        }
    }

    fn import_rules_event(inner: &Inner) -> DaemonEvent {
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
        let (tx, ev) = {
            let mut inner = self.lock();
            inner.focus_active = true;
            inner.active_rule_set_id = Some(rule_set_id);
            inner.schedule_activated = false;
            (inner.event_tx.clone(), Self::focus_event(&inner))
        };
        Self::emit(&tx, ev);
    }

    pub fn stop_focus(&self) {
        let (tx, focus_ev) = {
            let mut inner = self.lock();
            inner.focus_active = false;
            inner.active_rule_set_id = None;
            inner.pomodoro = None;
            inner.schedule_activated = false;
            (inner.event_tx.clone(), Self::focus_event(&inner))
        };
        Self::emit(&tx, focus_ev);
        Self::emit(
            &tx,
            DaemonEvent::PomodoroTick {
                phase: None,
                seconds_remaining: None,
            },
        );
    }

    pub fn start_pomodoro(&self, focus_secs: u64, break_secs: u64, rule_set_id: Option<Uuid>) {
        let (tx, focus_ev, pom_ev) = {
            let mut inner = self.lock();
            inner.focus_active = true;
            inner.active_rule_set_id = rule_set_id;
            let pom = PomodoroTimer::new(focus_secs, break_secs);
            let secs = pom.seconds_remaining();
            inner.pomodoro = Some(pom);
            let focus_ev = Self::focus_event(&inner);
            let pom_ev = DaemonEvent::PomodoroTick {
                phase: Some(PomodoroPhase::Focus),
                seconds_remaining: Some(secs),
            };
            (inner.event_tx.clone(), focus_ev, pom_ev)
        };
        Self::emit(&tx, focus_ev);
        Self::emit(&tx, pom_ev);
    }

    pub fn stop_pomodoro(&self) {
        let (tx, focus_ev) = {
            let mut inner = self.lock();
            inner.pomodoro = None;
            inner.focus_active = false;
            inner.active_rule_set_id = None;
            (inner.event_tx.clone(), Self::focus_event(&inner))
        };
        Self::emit(&tx, focus_ev);
        Self::emit(
            &tx,
            DaemonEvent::PomodoroTick {
                phase: None,
                seconds_remaining: None,
            },
        );
    }

    pub fn active_rule_set(&self) -> Option<RuleSet> {
        let inner = self.lock();
        let id = inner.active_rule_set_id?;
        inner.config.rule_sets.iter().find(|r| r.id == id).cloned()
    }

    pub fn config(&self) -> Config {
        self.lock().config.clone()
    }

    pub fn add_rule_set(&self, rule_set: RuleSet) {
        let (tx, ev1, ev2) = {
            let mut inner = self.lock();
            let id = rule_set.id;
            inner.config.rule_sets.push(rule_set);
            if inner.config.default_rule_set_id.is_none() {
                inner.config.default_rule_set_id = Some(id);
            }
            (
                inner.event_tx.clone(),
                Self::rule_sets_event(&inner),
                Self::config_event(&inner),
            )
        };
        Self::emit(&tx, ev1);
        Self::emit(&tx, ev2);
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
                        crate::pomodoro::Phase::Focus => PomodoroPhase::Focus,
                        crate::pomodoro::Phase::Break => PomodoroPhase::Break,
                    });
                    let secs = Some(pom.seconds_remaining());
                    (true, is_focus, phase, secs)
                }
                None => (false, false, None, None),
            };
            if is_active {
                inner.focus_active = is_focus;
            }
            is_active.then_some((inner.event_tx.clone(), phase, secs))
        }; // lock dropped here
        if let Some((tx, phase, seconds_remaining)) = pom_state {
            Self::emit(&tx, DaemonEvent::PomodoroTick { phase, seconds_remaining });
        }
    }

    /// Skip the current break and return to Focus immediately.
    /// Returns false if strict_breaks is enabled.
    pub fn skip_break(&self) -> bool {
        let (tx, ev) = {
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
            (inner.event_tx.clone(), Self::focus_event(&inner))
        };
        Self::emit(&tx, ev);
        true
    }

    pub fn set_open_tabs(&self, tabs: Vec<(String, String)>) {
        self.lock().open_tabs = tabs;
    }

    pub fn get_open_tabs(&self) -> Vec<(String, String)> {
        self.lock().open_tabs.clone()
    }

    pub fn remove_rule_set(&self, id: Uuid) {
        let (tx, ev1, ev2) = {
            let mut inner = self.lock();
            inner.config.rule_sets.retain(|r| r.id != id);
            if inner.config.default_rule_set_id == Some(id) {
                inner.config.default_rule_set_id =
                    inner.config.rule_sets.first().map(|r| r.id);
            }
            (
                inner.event_tx.clone(),
                Self::rule_sets_event(&inner),
                Self::config_event(&inner),
            )
        };
        Self::emit(&tx, ev1);
        Self::emit(&tx, ev2);
    }

    pub fn set_default_rule_set(&self, id: Uuid) -> bool {
        let result = {
            let mut inner = self.lock();
            if inner.config.rule_sets.iter().any(|r| r.id == id) {
                inner.config.default_rule_set_id = Some(id);
                let ev = Self::config_event(&inner);
                Some((inner.event_tx.clone(), ev))
            } else {
                None
            }
        };
        if let Some((tx, ev)) = result {
            Self::emit(&tx, ev);
            true
        } else {
            false
        }
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
        let (tx, ev) = {
            let mut inner = self.lock();
            inner.config.schedules.push(schedule);
            (inner.event_tx.clone(), Self::schedules_event(&inner))
        };
        Self::emit(&tx, ev);
    }

    pub fn remove_schedule(&self, id: Uuid) {
        let (tx, ev) = {
            let mut inner = self.lock();
            inner.config.schedules.retain(|s| s.id != id);
            (inner.event_tx.clone(), Self::schedules_event(&inner))
        };
        Self::emit(&tx, ev);
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
        let (tx, ev) = {
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
            (inner.event_tx.clone(), Self::schedules_event(&inner))
        };
        Self::emit(&tx, ev);
    }

    /// Check active schedules and start/stop focus accordingly.
    /// Called periodically by the background scheduler loop.
    pub fn apply_schedule(&self) {
        let result = {
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
                                // If no specific list was assigned (nil sentinel), fall back
                                // to the effective default rule set so blocking still works.
                                let eid = if s.rule_set_id.is_nil() {
                                    inner
                                        .config
                                        .default_rule_set_id
                                        .filter(|id| {
                                            inner.config.rule_sets.iter().any(|r| r.id == *id)
                                        })
                                        .or_else(|| inner.config.rule_sets.first().map(|r| r.id))
                                        .unwrap_or_else(Uuid::nil)
                                } else {
                                    s.rule_set_id
                                };
                                (Some(eid), brk)
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

            if inner.focus_active != prev_active || inner.active_rule_set_id != prev_rule_set {
                let ev = Self::focus_event(&inner);
                Some((inner.event_tx.clone(), ev))
            } else {
                None
            }
        }; // lock dropped

        if let Some((tx, ev)) = result {
            Self::emit(&tx, ev);
        }
    }

    pub fn remove_url_from_rule_set(&self, rule_set_id: Uuid, url: &str) -> bool {
        let result = {
            let mut inner = self.lock();
            if let Some(rs) = inner
                .config
                .rule_sets
                .iter_mut()
                .find(|r| r.id == rule_set_id)
            {
                rs.allowed_urls.retain(|u| u != url);
                let ev = Self::rule_sets_event(&inner);
                Some((inner.event_tx.clone(), ev))
            } else {
                None
            }
        };
        if let Some((tx, ev)) = result {
            Self::emit(&tx, ev);
            true
        } else {
            false
        }
    }

    pub fn add_url_to_rule_set(&self, rule_set_id: Uuid, url: String) -> bool {
        let result = {
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
                let ev = Self::rule_sets_event(&inner);
                Some((inner.event_tx.clone(), ev))
            } else {
                None
            }
        };
        if let Some((tx, ev)) = result {
            Self::emit(&tx, ev);
            true
        } else {
            false
        }
    }

    /// Replace calendar-imported schedules with a fresh set.
    pub fn apply_calendar_schedules(&self, imported: Vec<shared::models::Schedule>) {
        let (tx, ev) = {
            let mut inner = self.lock();
            inner.config.schedules.retain(|s| !s.imported);
            inner.config.schedules.extend(imported);
            (inner.event_tx.clone(), Self::schedules_event(&inner))
        };
        Self::emit(&tx, ev);
    }

    pub fn caldav_config(&self) -> Option<shared::models::CalDavConfig> {
        self.lock().config.caldav.clone()
    }

    pub fn list_import_rules(&self) -> Vec<shared::models::CalendarImportRule> {
        self.lock().config.import_rules.clone()
    }

    pub fn add_import_rule(&self, keyword: String, schedule_type: shared::models::ScheduleType) {
        let keyword = keyword.to_lowercase();
        let (tx, ev) = {
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
            (inner.event_tx.clone(), Self::import_rules_event(&inner))
        };
        Self::emit(&tx, ev);
    }

    pub fn remove_import_rule(
        &self,
        keyword: &str,
        schedule_type: &shared::models::ScheduleType,
    ) {
        let keyword = keyword.to_lowercase();
        let (tx, ev) = {
            let mut inner = self.lock();
            inner
                .config
                .import_rules
                .retain(|r| !(r.keyword == keyword && &r.schedule_type == schedule_type));
            (inner.event_tx.clone(), Self::import_rules_event(&inner))
        };
        Self::emit(&tx, ev);
    }

    pub fn set_strict_mode(&self, enabled: bool) {
        let (tx, ev) = {
            let mut inner = self.lock();
            inner.config.strict_mode = enabled;
            (inner.event_tx.clone(), Self::config_event(&inner))
        };
        Self::emit(&tx, ev);
    }

    pub fn set_allow_new_tab(&self, enabled: bool) {
        let (tx, ev) = {
            let mut inner = self.lock();
            inner.config.allow_new_tab = enabled;
            (inner.event_tx.clone(), Self::config_event(&inner))
        };
        Self::emit(&tx, ev);
    }

    pub fn set_accent_color(&self, hex: String) {
        let (tx, ev) = {
            let mut inner = self.lock();
            inner.config.accent_color = hex;
            (inner.event_tx.clone(), Self::config_event(&inner))
        };
        Self::emit(&tx, ev);
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
        let (tx, ev) = {
            let mut inner = self.lock();
            inner.config.google_calendar = Some(shared::models::GoogleCalendarConfig {
                client_id,
                client_secret,
                access_token: Some(access_token),
                refresh_token: Some(refresh_token),
                token_expiry_secs: Some(expiry_secs),
            });
            (inner.event_tx.clone(), Self::config_event(&inner))
        };
        Self::emit(&tx, ev);
    }

    pub fn update_google_tokens(&self, access_token: String, expiry_secs: i64) {
        let (tx, ev) = {
            let mut inner = self.lock();
            if let Some(cfg) = &mut inner.config.google_calendar {
                cfg.access_token = Some(access_token);
                cfg.token_expiry_secs = Some(expiry_secs);
            }
            (inner.event_tx.clone(), Self::config_event(&inner))
        };
        Self::emit(&tx, ev);
    }

    pub fn revoke_google_calendar(&self) {
        let (tx, ev) = {
            let mut inner = self.lock();
            inner.config.google_calendar = None;
            (inner.event_tx.clone(), Self::config_event(&inner))
        };
        Self::emit(&tx, ev);
    }

    pub fn google_calendar_config(&self) -> Option<shared::models::GoogleCalendarConfig> {
        self.lock().config.google_calendar.clone()
    }

    pub fn set_caldav(&self, url: String, username: String, password: String) {
        let (tx, ev) = {
            let mut inner = self.lock();
            inner.config.caldav = Some(shared::models::CalDavConfig {
                url,
                username: Some(username),
                password: Some(password),
            });
            (inner.event_tx.clone(), Self::config_event(&inner))
        };
        Self::emit(&tx, ev);
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

    /// Build a full InitialSnapshot event in a single lock acquisition.
    pub fn build_snapshot_event(&self) -> DaemonEvent {
        let inner = self.lock();

        let active_rule_set_name = inner
            .active_rule_set_id
            .and_then(|id| inner.config.rule_sets.iter().find(|r| r.id == id))
            .map(|r| r.name.clone());

        let (pomodoro_active, pomodoro_phase, seconds_remaining) =
            if let Some(pom) = &inner.pomodoro {
                (
                    true,
                    Some(match pom.phase {
                        crate::pomodoro::Phase::Focus => PomodoroPhase::Focus,
                        crate::pomodoro::Phase::Break => PomodoroPhase::Break,
                    }),
                    Some(pom.seconds_remaining()),
                )
            } else {
                (false, None, None)
            };

        let status = StatusResponse {
            focus_active: inner.focus_active,
            strict_mode: inner.config.strict_mode,
            allow_new_tab: inner.config.allow_new_tab,
            active_rule_set_name,
            pomodoro_active,
            pomodoro_phase,
            seconds_remaining,
            google_calendar_connected: inner
                .config
                .google_calendar
                .as_ref()
                .map(|c| c.access_token.is_some())
                .unwrap_or(false),
            caldav_url: inner.config.caldav.as_ref().map(|c| c.url.clone()),
            default_rule_set_id: inner.config.default_rule_set_id,
            accent_color: inner.config.accent_color.clone(),
        };

        let rule_sets = Self::rule_set_summaries(&inner);
        let schedules = Self::schedule_summaries(&inner);

        let import_rules = inner
            .config
            .import_rules
            .iter()
            .map(|r| ImportRuleSummary {
                keyword: r.keyword.clone(),
                schedule_type: r.schedule_type.clone(),
            })
            .collect();

        DaemonEvent::InitialSnapshot {
            status,
            rule_sets,
            schedules,
            import_rules,
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
