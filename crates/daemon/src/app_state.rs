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
    }

    pub fn stop_focus(&self) {
        let mut inner = self.0.lock().unwrap();
        inner.focus_active = false;
        inner.active_rule_set_id = None;
        inner.pomodoro = None;
    }

    pub fn start_pomodoro(&self, focus_secs: u64, break_secs: u64) {
        let mut inner = self.0.lock().unwrap();
        inner.focus_active = true;
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

    pub fn config(&self) -> Config {
        self.0.lock().unwrap().config.clone()
    }

    pub fn add_rule_set(&self, rule_set: RuleSet) {
        self.0.lock().unwrap().config.rule_sets.push(rule_set);
    }

    pub fn remove_rule_set(&self, id: Uuid) {
        let mut inner = self.0.lock().unwrap();
        inner.config.rule_sets.retain(|r| r.id != id);
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

        StateSnapshot {
            focus_active: inner.focus_active,
            strict_mode: inner.config.strict_mode,
            active_rule_set_name,
            pomodoro_active,
            pomodoro_phase,
            seconds_remaining,
        }
    }
}

pub struct StateSnapshot {
    pub focus_active: bool,
    pub strict_mode: bool,
    pub active_rule_set_name: Option<String>,
    pub pomodoro_active: bool,
    pub pomodoro_phase: Option<crate::pomodoro::Phase>,
    pub seconds_remaining: Option<u64>,
}
