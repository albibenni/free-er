use crate::sections::{
    allowed_lists::AllowedListsInput, calendar_rules::CalendarRulesInput, focus::FocusInput,
    pomodoro::PomodoroInput, schedule::ScheduleInput, settings::SettingsInput,
};
use relm4::{ComponentController, ComponentSender};
use shared::ipc::{DaemonEvent, RuleSetSummary};

use super::{App, AppMsg};

/// Dispatch a push event from the daemon subscription to the appropriate child components.
pub(super) fn handle_event(app: &App, event: DaemonEvent, sender: ComponentSender<App>) {
    let focus_sender = app.focus.sender().clone();
    let pom_sender = app.pomodoro.sender().clone();
    let lists_sender = app.allowed_lists.sender().clone();
    let settings_sender = app.settings.sender().clone();
    let schedule_sender = app.schedule.sender().clone();
    let cal_sender = app.calendar_rules.sender().clone();

    match event {
        DaemonEvent::InitialSnapshot {
            status,
            rule_sets,
            schedules,
            import_rules,
        } => {
            focus_sender.emit(FocusInput::StatusUpdated {
                active: status.focus_active,
                rule_set: status.active_rule_set_name,
            });
            pom_sender.emit(PomodoroInput::StatusUpdated {
                phase: status.pomodoro_phase.map(|p| format!("{p:?}")),
                seconds_remaining: status.seconds_remaining,
            });
            settings_sender.emit(SettingsInput::GoogleStatusUpdated(
                status.google_calendar_connected,
            ));
            settings_sender.emit(SettingsInput::AllowNewTabUpdated(status.allow_new_tab));
            settings_sender.emit(SettingsInput::AccentColorUpdated(status.accent_color.clone()));
            pom_sender.emit(PomodoroInput::AccentColorUpdated(status.accent_color.clone()));
            sender.input(AppMsg::ApplyAccentCss(status.accent_color));
            if let Some(id) = status.default_rule_set_id {
                sender.input(AppMsg::SetDefaultRuleSet(id));
            }
            dispatch_rule_sets(
                rule_sets,
                &lists_sender,
                &pom_sender,
                &schedule_sender,
                &settings_sender,
                &sender,
            );
            schedule_sender.emit(ScheduleInput::SchedulesUpdated(schedules));
            cal_sender.emit(CalendarRulesInput::RulesUpdated(import_rules));
        }

        DaemonEvent::FocusChanged { active, rule_set_name } => {
            focus_sender.emit(FocusInput::StatusUpdated {
                active,
                rule_set: rule_set_name,
            });
        }

        DaemonEvent::PomodoroTick { phase, seconds_remaining } => {
            pom_sender.emit(PomodoroInput::StatusUpdated {
                phase: phase.map(|p| format!("{p:?}")),
                seconds_remaining,
            });
        }

        DaemonEvent::ConfigChanged {
            strict_mode: _,
            allow_new_tab,
            accent_color,
            google_calendar_connected,
            default_rule_set_id,
        } => {
            settings_sender.emit(SettingsInput::GoogleStatusUpdated(google_calendar_connected));
            settings_sender.emit(SettingsInput::AllowNewTabUpdated(allow_new_tab));
            settings_sender.emit(SettingsInput::AccentColorUpdated(accent_color.clone()));
            pom_sender.emit(PomodoroInput::AccentColorUpdated(accent_color.clone()));
            sender.input(AppMsg::ApplyAccentCss(accent_color));
            if let Some(id) = default_rule_set_id {
                sender.input(AppMsg::SetDefaultRuleSet(id));
            }
        }

        DaemonEvent::RuleSetsChanged { rule_sets } => {
            dispatch_rule_sets(
                rule_sets,
                &lists_sender,
                &pom_sender,
                &schedule_sender,
                &settings_sender,
                &sender,
            );
        }

        DaemonEvent::SchedulesChanged { schedules } => {
            sender.input(AppMsg::SchedulesUpdated(schedules));
        }

        DaemonEvent::ImportRulesChanged { rules } => {
            cal_sender.emit(CalendarRulesInput::RulesUpdated(rules));
        }
    }
}

fn dispatch_rule_sets(
    rule_sets: Vec<RuleSetSummary>,
    lists_sender: &relm4::Sender<AllowedListsInput>,
    pom_sender: &relm4::Sender<PomodoroInput>,
    schedule_sender: &relm4::Sender<ScheduleInput>,
    settings_sender: &relm4::Sender<SettingsInput>,
    _tick_sender: &ComponentSender<App>,
) {
    use crate::sections::allowed_lists::AllowedListsInput as AL;
    use crate::sections::pomodoro::PomodoroInput as PI;
    use crate::sections::schedule::ScheduleInput as SI;

    lists_sender.emit(AL::RuleSetsUpdated(rule_sets.clone()));
    pom_sender.emit(PI::RuleSetsUpdated(rule_sets.clone()));
    schedule_sender.emit(SI::RuleSetsUpdated(rule_sets.clone()));

    let all_urls: Vec<String> = rule_sets.iter().flat_map(|s| s.allowed_urls.clone()).collect();
    settings_sender.emit(SettingsInput::QuickUrlsUpdated(all_urls));

    // Don't override the default_rule_set_id from here — it comes from ConfigChanged/InitialSnapshot.
}
