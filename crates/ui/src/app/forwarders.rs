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

pub(super) fn launch_focus(sender: &ComponentSender<App>) -> Controller<FocusSection> {
    FocusSection::builder()
        .launch(())
        .forward(sender.input_sender(), |out| match out {
            FocusOutput::StartFocus => AppMsg::StartFocus,
            FocusOutput::StopFocus => AppMsg::StopFocus,
            FocusOutput::SkipBreak => AppMsg::SkipBreak,
        })
}

pub(super) fn launch_pomodoro(sender: &ComponentSender<App>) -> Controller<PomodoroSection> {
    PomodoroSection::builder()
        .launch(())
        .forward(sender.input_sender(), |out| match out {
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
        })
}

pub(super) fn launch_allowed_lists(sender: &ComponentSender<App>) -> Controller<AllowedListsSection> {
    AllowedListsSection::builder()
        .launch(())
        .forward(sender.input_sender(), |out| match out {
            AllowedListsOutput::AddUrl { rule_set_id, url } => {
                AppMsg::AddUrlToList { rule_set_id, url }
            }
            AllowedListsOutput::RemoveUrl { rule_set_id, url } => {
                AppMsg::RemoveUrlFromList { rule_set_id, url }
            }
            AllowedListsOutput::CreateRuleSet(name) => AppMsg::CreateRuleSet(name),
            AllowedListsOutput::DeleteRuleSet(id) => AppMsg::DeleteRuleSet(id),
            AllowedListsOutput::SetDefaultRuleSet(id) => AppMsg::ChooseDefaultRuleSet(id),
        })
}

pub(super) fn launch_schedule(sender: &ComponentSender<App>) -> Controller<ScheduleSection> {
    ScheduleSection::builder()
        .launch(())
        .forward(sender.input_sender(), |out| match out {
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
        })
}

pub(super) fn launch_calendar_rules(sender: &ComponentSender<App>) -> Controller<CalendarRulesSection> {
    CalendarRulesSection::builder()
        .launch(())
        .forward(sender.input_sender(), |out| match out {
            CalendarRulesOutput::AddRule { keyword, schedule_type } => {
                AppMsg::AddImportRule { keyword, schedule_type }
            }
            CalendarRulesOutput::RemoveRule { keyword, schedule_type } => {
                AppMsg::RemoveImportRule { keyword, schedule_type }
            }
        })
}

pub(super) fn launch_settings(sender: &ComponentSender<App>) -> Controller<SettingsSection> {
    SettingsSection::builder()
        .launch(false)
        .forward(sender.input_sender(), |out| match out {
            SettingsOutput::StrictModeChanged(v) => AppMsg::StrictModeChanged(v),
            SettingsOutput::AllowNewTabChanged(v) => AppMsg::AllowNewTabChanged(v),
            SettingsOutput::AiSitesToggled(v) => AppMsg::AiSitesToggled(v),
            SettingsOutput::SearchEnginesToggled(v) => AppMsg::SearchEnginesToggled(v),
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
        })
}
