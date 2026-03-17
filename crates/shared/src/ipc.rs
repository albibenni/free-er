use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Commands sent from a client (UI, CLI) to the daemon over the Unix socket.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "cmd")]
pub enum Command {
    StartFocus { rule_set_id: Uuid },
    StopFocus,
    TakeBreak { duration_secs: u64 },
    StartPomodoro { focus_secs: u64, break_secs: u64 },
    StopPomodoro,
    /// Skip the current break and go straight to the next focus phase.
    /// Rejected by the daemon if strict_breaks is enabled.
    SkipBreak,
    GetStatus,
    AddRuleSet { name: String, allowed_urls: Vec<String> },
    RemoveRuleSet { id: Uuid },
    AddUrlToRuleSet { rule_set_id: Uuid, url: String },
    RemoveUrlFromRuleSet { rule_set_id: Uuid, url: String },
    ListRuleSets,
    AddSchedule {
        name: String,
        days: Vec<String>,
        start: String,
        end: String,
        rule_set_id: Uuid,
    },
    RemoveSchedule { id: Uuid },
    SetStrictMode { enabled: bool },
    SetCalDav { url: String, username: String, password: String },
}

/// Returned by ListRuleSets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSetSummary {
    pub id: Uuid,
    pub name: String,
    pub allowed_urls: Vec<String>,
}

/// Current phase of the Pomodoro timer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PomodoroPhase {
    Focus,
    Break,
}

/// Response sent from the daemon back to the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusResponse {
    pub focus_active: bool,
    pub strict_mode: bool,
    pub active_rule_set_name: Option<String>,
    pub pomodoro_active: bool,
    pub pomodoro_phase: Option<PomodoroPhase>,
    pub seconds_remaining: Option<u64>,
}
