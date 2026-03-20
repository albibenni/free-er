use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use crate::models::ScheduleType;

/// Commands sent from a client (UI, CLI) to the daemon over the Unix socket.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "cmd")]
pub enum Command {
    StartFocus {
        rule_set_id: Uuid,
    },
    StopFocus,
    TakeBreak {
        duration_secs: u64,
    },
    StartPomodoro {
        focus_secs: u64,
        break_secs: u64,
        rule_set_id: Option<Uuid>,
    },
    StopPomodoro,
    /// Skip the current break and go straight to the next focus phase.
    /// Rejected by the daemon if strict_breaks is enabled.
    SkipBreak,
    GetStatus,
    AddRuleSet {
        name: String,
        allowed_urls: Vec<String>,
    },
    RemoveRuleSet {
        id: Uuid,
    },
    AddUrlToRuleSet {
        rule_set_id: Uuid,
        url: String,
    },
    RemoveUrlFromRuleSet {
        rule_set_id: Uuid,
        url: String,
    },
    ListRuleSets,
    SetDefaultRuleSet {
        id: Uuid,
    },
    AddSchedule {
        name: String,
        /// Weekday indices 0=Mon..6=Sun
        days: Vec<u8>,
        start_min: u32,
        end_min: u32,
        rule_set_id: Option<Uuid>,
        /// If set (YYYY-MM-DD), the event is pinned to that specific date only.
        specific_date: Option<String>,
        schedule_type: ScheduleType,
    },
    RemoveSchedule {
        id: Uuid,
    },
    UpdateSchedule {
        id: Uuid,
        name: String,
        days: Vec<u8>,
        start_min: u32,
        end_min: u32,
        /// Overwrite the event's allowed-list association. `None` clears it.
        rule_set_id: Option<Uuid>,
        /// Overwrite the event's specific_date. `None` clears it for repeating schedules.
        specific_date: Option<String>,
        schedule_type: ScheduleType,
    },
    ListSchedules,
    SetStrictMode {
        enabled: bool,
    },
    SetAllowNewTab {
        enabled: bool,
    },
    SetCalDav {
        url: String,
        username: String,
        password: String,
    },
    StartGoogleOAuth,
    RevokeGoogleCalendar,
    SyncCalendar,
    /// Add a global calendar import rule.
    AddImportRule {
        keyword: String,
        schedule_type: ScheduleType,
    },
    /// Remove a global calendar import rule by keyword + type.
    RemoveImportRule {
        keyword: String,
        schedule_type: ScheduleType,
    },
    /// List all global calendar import rules.
    ListImportRules,
}

/// Returned by ListSchedules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleSummary {
    pub id: Uuid,
    pub name: String,
    /// Weekday indices: 0=Mon, 1=Tue, …, 6=Sun. Empty for one-time events.
    pub days: Vec<u8>,
    /// Minutes from midnight
    pub start_min: u32,
    pub end_min: u32,
    pub enabled: bool,
    pub imported: bool,
    pub imported_repeating: bool,
    /// If set, this is a one-time event on this specific date (YYYY-MM-DD).
    /// If None, the event repeats weekly on the days in `days`.
    pub specific_date: Option<String>,
    pub schedule_type: ScheduleType,
    /// UUID of the associated rule set (Uuid::nil if none).
    pub rule_set_id: Uuid,
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

/// Returned by ListImportRules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportRuleSummary {
    pub keyword: String,
    pub schedule_type: ScheduleType,
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
    pub google_calendar_connected: bool,
    pub allow_new_tab: bool,
    pub default_rule_set_id: Option<Uuid>,
}
