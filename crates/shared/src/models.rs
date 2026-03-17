use chrono::{Datelike, NaiveTime, Weekday};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A named list of allowed URL patterns (wildcards supported).
/// During a focus session, only URLs matching this list are accessible.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSet {
    pub id: Uuid,
    pub name: String,
    /// Wildcard URL patterns, e.g. "github.com", "*.rust-lang.org"
    pub allowed_urls: Vec<String>,
}

impl RuleSet {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            allowed_urls: Vec::new(),
        }
    }
}

/// Whether a schedule window is a focus session or a break.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum ScheduleType {
    #[default]
    Focus,
    Break,
}

/// A recurring weekly schedule that activates a focus session automatically.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    pub id: Uuid,
    pub name: String,
    pub days: Vec<Weekday>,
    pub start: NaiveTime,
    pub end: NaiveTime,
    pub rule_set_id: Uuid,
    pub enabled: bool,
    #[serde(default)]
    pub imported: bool,
    /// If set, this is a one-time event on a specific date (not a recurring weekly schedule).
    #[serde(default)]
    pub specific_date: Option<chrono::NaiveDate>,
    #[serde(default)]
    pub schedule_type: ScheduleType,
}

impl Schedule {
    /// Returns true if this schedule is active at the given weekday + time.
    pub fn is_active(&self, day: Weekday, time: NaiveTime) -> bool {
        self.enabled && self.days.contains(&day) && time >= self.start && time < self.end
    }

    /// Returns true if this schedule is active right now (respects specific_date).
    pub fn is_active_now(&self) -> bool {
        if !self.enabled {
            return false;
        }
        let now = chrono::Local::now();
        let today = now.date_naive();
        let time = now.time();
        let day_matches = if let Some(specific) = self.specific_date {
            specific == today
        } else {
            self.days.contains(&today.weekday())
        };
        day_matches && time >= self.start && time < self.end
    }
}

/// Configuration for the Pomodoro timer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PomodoroConfig {
    pub focus_secs: u64,
    pub break_secs: u64,
    /// If true, the user cannot manually end a break early.
    pub strict_breaks: bool,
}

impl Default for PomodoroConfig {
    fn default() -> Self {
        Self {
            focus_secs: 25 * 60,
            break_secs: 5 * 60,
            strict_breaks: false,
        }
    }
}

/// A keyword rule: if a calendar event title contains `keyword`,
/// it is imported as a focus session using `rule_set_id`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarImportRule {
    pub keyword: String,
    pub rule_set_id: Uuid,
}

/// CalDAV / remote .ics source configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CalDavConfig {
    /// Full URL to the .ics feed or CalDAV calendar.
    pub url: String,
    pub username: Option<String>,
    pub password: Option<String>,
    /// How to map event titles → rule sets.
    pub import_rules: Vec<CalendarImportRule>,
}

/// OAuth2 credentials and tokens for Google Calendar integration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GoogleCalendarConfig {
    pub client_id: String,
    pub client_secret: String,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    /// Unix timestamp (seconds) at which the access_token expires.
    pub token_expiry_secs: Option<i64>,
    /// How to map event titles → rule sets (shared with CalDAV import_rules).
    pub import_rules: Vec<CalendarImportRule>,
}

/// Top-level persisted config written to ~/.config/free-er/config.json
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub rule_sets: Vec<RuleSet>,
    pub schedules: Vec<Schedule>,
    pub pomodoro: PomodoroConfig,
    pub caldav: Option<CalDavConfig>,
    pub google_calendar: Option<GoogleCalendarConfig>,
    /// If true, focus cannot be stopped manually while a schedule is active.
    pub strict_mode: bool,
}
