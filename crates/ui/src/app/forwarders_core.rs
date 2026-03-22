use relm4::prelude::*;

use crate::sections::{
    allowed_lists::{AllowedListsOutput, AllowedListsSection},
    calendar_rules::{CalendarRulesOutput, CalendarRulesSection},
    focus::{FocusOutput, FocusSection},
    pomodoro::{PomodoroOutput, PomodoroSection},
    schedule::{ScheduleOutput, ScheduleSection},
    settings::{SettingsOutput, SettingsSection},
};

use super::{App, AppMsg};

fn map_focus_output(out: FocusOutput) -> AppMsg {
    match out {
        FocusOutput::SkipBreak => AppMsg::SkipBreak,
        FocusOutput::TakeBreak { break_secs } => AppMsg::TakeBreak { break_secs },
    }
}

fn map_pomodoro_output(out: PomodoroOutput) -> AppMsg {
    match out {
        PomodoroOutput::Start {
            focus_secs,
            break_secs,
            rule_set_id,
        } => AppMsg::StartPomodoro {
            focus_secs,
            break_secs,
            rule_set_id,
        },
        PomodoroOutput::Stop => AppMsg::StopPomodoro,
    }
}

fn map_allowed_lists_output(out: AllowedListsOutput) -> AppMsg {
    match out {
        AllowedListsOutput::AddUrl { rule_set_id, url } => AppMsg::AddUrlToList { rule_set_id, url },
        AllowedListsOutput::RemoveUrl { rule_set_id, url } => {
            AppMsg::RemoveUrlFromList { rule_set_id, url }
        }
        AllowedListsOutput::CreateRuleSet(name) => AppMsg::CreateRuleSet(name),
        AllowedListsOutput::DeleteRuleSet(id) => AppMsg::DeleteRuleSet(id),
        AllowedListsOutput::SetDefaultRuleSet(id) => AppMsg::ChooseDefaultRuleSet(id),
        AllowedListsOutput::RequestOpenTabs => AppMsg::FetchOpenTabs,
    }
}

fn map_schedule_output(out: ScheduleOutput) -> AppMsg {
    match out {
        ScheduleOutput::CreateSchedule {
            name,
            days,
            start_min,
            end_min,
            specific_date,
            rule_set_id,
            schedule_type,
        } => AppMsg::CreateSchedule {
            name,
            days,
            start_min,
            end_min,
            specific_date,
            rule_set_id,
            schedule_type,
        },
        ScheduleOutput::UpdateSchedule {
            id,
            name,
            days,
            start_min,
            end_min,
            rule_set_id,
            specific_date,
            schedule_type,
        } => AppMsg::UpdateSchedule {
            id,
            name,
            days,
            start_min,
            end_min,
            rule_set_id,
            specific_date,
            schedule_type,
        },
        ScheduleOutput::DeleteSchedule(id) => AppMsg::DeleteSchedule(id),
        ScheduleOutput::ResyncCalendar => AppMsg::ResyncCalendar,
    }
}

fn map_calendar_rules_output(out: CalendarRulesOutput) -> AppMsg {
    match out {
        CalendarRulesOutput::SaveCalDav { url, user, pass } => {
            AppMsg::SaveCalDav { url, user, pass }
        }
        CalendarRulesOutput::ConnectGoogleRequested => AppMsg::ConnectGoogle,
        CalendarRulesOutput::DisconnectGoogleRequested => AppMsg::DisconnectGoogle,
        CalendarRulesOutput::AddRule {
            keyword,
            schedule_type,
        } => AppMsg::AddImportRule {
            keyword,
            schedule_type,
        },
        CalendarRulesOutput::RemoveRule {
            keyword,
            schedule_type,
        } => AppMsg::RemoveImportRule {
            keyword,
            schedule_type,
        },
    }
}

fn map_settings_output(out: SettingsOutput) -> AppMsg {
    match out {
        SettingsOutput::StrictModeChanged(v) => AppMsg::StrictModeChanged(v),
        SettingsOutput::AllowNewTabChanged(v) => AppMsg::AllowNewTabChanged(v),
        SettingsOutput::AiSitesToggled(v) => AppMsg::AiSitesToggled(v),
        SettingsOutput::SearchEnginesToggled(v) => AppMsg::SearchEnginesToggled(v),
        SettingsOutput::LocalhostToggled(v) => AppMsg::LocalhostToggled(v),
        SettingsOutput::QuickUrlToggled { url, enabled } => {
            if enabled {
                AppMsg::AddUrl(url.to_string())
            } else {
                AppMsg::RemoveUrl(url.to_string())
            }
        }
        SettingsOutput::CalDavSaved { url, user, pass } => AppMsg::SaveCalDav { url, user, pass },
        SettingsOutput::ConnectGoogleRequested => AppMsg::ConnectGoogle,
        SettingsOutput::DisconnectGoogleRequested => AppMsg::DisconnectGoogle,
        SettingsOutput::AccentColorChanged(hex) => AppMsg::AccentColorChanged(hex),
    }
}

pub(super) fn launch_focus(sender: &ComponentSender<App>) -> Controller<FocusSection> {
    FocusSection::builder()
        .launch(())
        .forward(sender.input_sender(), map_focus_output)
}

pub(super) fn launch_pomodoro(sender: &ComponentSender<App>) -> Controller<PomodoroSection> {
    PomodoroSection::builder()
        .launch(())
        .forward(sender.input_sender(), map_pomodoro_output)
}

pub(super) fn launch_allowed_lists(
    sender: &ComponentSender<App>,
) -> Controller<AllowedListsSection> {
    AllowedListsSection::builder()
        .launch(())
        .forward(sender.input_sender(), map_allowed_lists_output)
}

pub(super) fn launch_schedule(sender: &ComponentSender<App>) -> Controller<ScheduleSection> {
    ScheduleSection::builder()
        .launch(())
        .forward(sender.input_sender(), map_schedule_output)
}

pub(super) fn launch_calendar_rules(
    sender: &ComponentSender<App>,
) -> Controller<CalendarRulesSection> {
    CalendarRulesSection::builder()
        .launch(())
        .forward(sender.input_sender(), map_calendar_rules_output)
}

pub(super) fn launch_settings(sender: &ComponentSender<App>) -> Controller<SettingsSection> {
    SettingsSection::builder()
        .launch(false)
        .forward(sender.input_sender(), map_settings_output)
}

#[cfg(test)]
#[path = "forwarders_tests.rs"]
mod tests;
