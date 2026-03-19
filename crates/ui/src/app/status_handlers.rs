use crate::ipc_client;
use crate::sections::{
    allowed_lists::AllowedListsInput, calendar_rules::CalendarRulesInput, focus::FocusInput,
    pomodoro::PomodoroInput, schedule::ScheduleInput, settings::SettingsInput,
};
use relm4::{ComponentController, ComponentSender, Sender};
use tracing::warn;

use super::{App, AppMsg};

pub(super) fn status_tick(app: &App, sender: ComponentSender<App>) {
    let focus_sender = app.focus.sender().clone();
    let pom_sender = app.pomodoro.sender().clone();
    let lists_sender = app.allowed_lists.sender().clone();
    let settings_sender = app.settings.sender().clone();
    let schedule_sender = app.schedule.sender().clone();
    let cal_sender = app.calendar_rules.sender().clone();

    tokio::spawn(async move {
        match ipc_client::get_status().await {
            Ok(status) => {
                focus_sender.emit(FocusInput::StatusUpdated {
                    active: status.focus_active,
                    rule_set: status.active_rule_set_name,
                });
                pom_sender.emit(PomodoroInput::StatusUpdated {
                    phase: status.pomodoro_phase.map(|p| format!("{p:?}")),
                    seconds_remaining: status.seconds_remaining,
                });
                settings_sender
                    .emit(SettingsInput::GoogleStatusUpdated(status.google_calendar_connected));
                settings_sender.emit(SettingsInput::AllowNewTabUpdated(status.allow_new_tab));
            }
            Err(e) => warn!("status poll failed: {e}"),
        }

        push_rule_sets(&lists_sender, &pom_sender, &schedule_sender, &settings_sender, &sender)
            .await;

        match ipc_client::list_schedules().await {
            Ok(schedules) => sender.input(AppMsg::SchedulesUpdated(schedules)),
            Err(e) => warn!("list_schedules failed: {e}"),
        }

        match ipc_client::list_import_rules().await {
            Ok(rules) => cal_sender.emit(CalendarRulesInput::RulesUpdated(rules)),
            Err(e) => warn!("list_import_rules failed: {e}"),
        }
    });
}

pub(super) fn refresh_rule_sets(app: &App, sender: ComponentSender<App>) {
    let lists_sender = app.allowed_lists.sender().clone();
    let pom_sender = app.pomodoro.sender().clone();
    let sched_sender = app.schedule.sender().clone();
    let settings_sender = app.settings.sender().clone();

    tokio::spawn(async move {
        push_rule_sets(&lists_sender, &pom_sender, &sched_sender, &settings_sender, &sender).await;
    });
}

async fn push_rule_sets(
    lists_sender: &Sender<AllowedListsInput>,
    pom_sender: &Sender<PomodoroInput>,
    sched_sender: &Sender<ScheduleInput>,
    settings_sender: &Sender<SettingsInput>,
    tick_sender: &ComponentSender<App>,
) {
    match ipc_client::list_rule_sets().await {
        Ok(sets) => {
            lists_sender.emit(AllowedListsInput::RuleSetsUpdated(sets.clone()));
            pom_sender.emit(PomodoroInput::RuleSetsUpdated(sets.clone()));
            sched_sender.emit(ScheduleInput::RuleSetsUpdated(sets.clone()));
            let all_urls: Vec<String> =
                sets.iter().flat_map(|s| s.allowed_urls.clone()).collect();
            settings_sender.emit(SettingsInput::QuickUrlsUpdated(all_urls));
            if let Some(first_id) = sets.first().map(|s| s.id) {
                tick_sender.input(AppMsg::SetDefaultRuleSet(first_id));
            }
        }
        Err(e) => warn!("list_rule_sets failed: {e}"),
    }
}
