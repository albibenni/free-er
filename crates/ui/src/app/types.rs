use relm4::Controller;
use shared::ipc::{ScheduleSummary, ScheduleType};
use uuid::Uuid;

use crate::sections::{
    allowed_lists::AllowedListsSection, calendar_rules::CalendarRulesSection, focus::FocusSection,
    pomodoro::PomodoroSection, schedule::ScheduleSection, settings::SettingsSection,
};

#[derive(Debug)]
pub enum Page {
    Focus,
    AllowedLists,
    Pomodoro,
    Schedule,
    Calendar,
    Settings,
}

pub struct App {
    pub(super) current_page: Page,
    pub(super) sidebar_open: bool,
    pub(super) focus_active: bool,
    pub(super) pomodoro_active: bool,
    /// ID of the selected default rule set.
    pub(super) default_rule_set_id: Option<Uuid>,
    /// Consecutive status-poll failures — when it reaches the threshold the UI exits.
    pub(super) daemon_failures: u32,
    /// Whether the "daemon gone" dialog is already showing.
    pub(super) daemon_dialog_shown: bool,
    pub(super) focus: Controller<FocusSection>,
    pub(super) pomodoro: Controller<PomodoroSection>,
    pub(super) allowed_lists: Controller<AllowedListsSection>,
    pub(super) schedule: Controller<ScheduleSection>,
    pub(super) calendar_rules: Controller<CalendarRulesSection>,
    pub(super) settings: Controller<SettingsSection>,
}

#[derive(Debug)]
pub enum AppMsg {
    Navigate(Page),
    ToggleSidebar,
    // Focus / Pomodoro session control
    StartFocus { rule_set_id: Option<Uuid> },
    StopFocus,
    SkipBreak,
    StartPomodoro {
        focus_secs: u64,
        break_secs: u64,
        rule_set_id: Option<Uuid>,
    },
    StopPomodoro,
    // URL / rule-set management
    AddUrl(String),
    RemoveUrl(String),
    AddUrlToList {
        rule_set_id: Uuid,
        url: String,
    },
    RemoveUrlFromList {
        rule_set_id: Uuid,
        url: String,
    },
    CreateRuleSet(String),
    DeleteRuleSet(Uuid),
    ChooseDefaultRuleSet(Uuid),
    AiSitesToggled(bool),
    SearchEnginesToggled(bool),
    LocalhostToggled(bool),
    // Settings / integrations
    ConnectGoogle,
    DisconnectGoogle,
    StrictModeChanged(bool),
    AllowNewTabChanged(bool),
    SaveCalDav {
        url: String,
        user: String,
        pass: String,
    },
    // Schedule CRUD
    SchedulesUpdated(Vec<ScheduleSummary>),
    CreateSchedule {
        name: String,
        days: Vec<u8>,
        start_min: u32,
        end_min: u32,
        specific_date: Option<String>,
        rule_set_id: Option<Uuid>,
        schedule_type: ScheduleType,
    },
    UpdateSchedule {
        id: Uuid,
        name: String,
        days: Vec<u8>,
        start_min: u32,
        end_min: u32,
        rule_set_id: Option<Uuid>,
        specific_date: Option<String>,
        schedule_type: ScheduleType,
    },
    DeleteSchedule(Uuid),
    RefreshSchedules,
    ResyncCalendar,
    // Calendar import rules
    AddImportRule {
        keyword: String,
        schedule_type: shared::ipc::ScheduleType,
    },
    RemoveImportRule {
        keyword: String,
        schedule_type: shared::ipc::ScheduleType,
    },
    // Status / refresh
    RefreshRuleSets,
    SetDefaultRuleSet(Uuid),
    AccentColorChanged(String),
    /// Apply accent CSS on the main thread (from status tick — no IPC call).
    ApplyAccentCss(String),
    /// Fetch open browser tabs from daemon and forward to AllowedLists.
    FetchOpenTabs,
    TakeBreak { break_secs: u64 },
    SetFocusActive(bool),
    SetPomodoroActive(bool),
    /// Send Shutdown to the daemon then quit the UI.
    ShutdownDaemon,
    /// A push event arrived from the daemon subscription.
    DaemonEvent(shared::ipc::DaemonEvent),
    /// The subscription task lost its connection — attempt reconnect.
    SubscriptionLost,
}

#[derive(Debug)]
pub enum AppCmdOutput {
    /// A DaemonEvent arrived from the long-running subscription task.
    DaemonEvent(shared::ipc::DaemonEvent),
    /// The subscription socket closed or errored.
    SubscriptionFailed,
}
