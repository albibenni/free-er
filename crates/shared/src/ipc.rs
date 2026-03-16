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
    GetStatus,
    AddRuleSet { name: String, allowed_urls: Vec<String> },
    RemoveRuleSet { id: Uuid },
    AddSchedule {
        name: String,
        days: Vec<String>,
        start: String,
        end: String,
        rule_set_id: Uuid,
    },
    RemoveSchedule { id: Uuid },
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
