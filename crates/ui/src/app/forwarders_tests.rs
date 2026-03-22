use super::*;
use shared::ipc::ScheduleType;
use uuid::Uuid;

#[test]
fn maps_focus_outputs() {
    assert!(matches!(
        map_focus_output(FocusOutput::SkipBreak),
        AppMsg::SkipBreak
    ));
    assert!(matches!(
        map_focus_output(FocusOutput::TakeBreak { break_secs: 300 }),
        AppMsg::TakeBreak { break_secs: 300 }
    ));
}

#[test]
fn maps_pomodoro_outputs() {
    let id = Uuid::new_v4();
    assert!(matches!(map_pomodoro_output(PomodoroOutput::Stop), AppMsg::StopPomodoro));
    let msg = map_pomodoro_output(PomodoroOutput::Start {
        focus_secs: 25,
        break_secs: 5,
        rule_set_id: Some(id),
    });
    match msg {
        AppMsg::StartPomodoro {
            focus_secs,
            break_secs,
            rule_set_id,
        } => {
            assert_eq!(focus_secs, 25);
            assert_eq!(break_secs, 5);
            assert_eq!(rule_set_id, Some(id));
        }
        _ => panic!("unexpected app message"),
    }
}

#[test]
fn maps_allowed_lists_outputs() {
    let id = Uuid::new_v4();
    assert!(matches!(
        map_allowed_lists_output(AllowedListsOutput::CreateRuleSet("x".to_string())),
        AppMsg::CreateRuleSet(_)
    ));
    assert!(matches!(
        map_allowed_lists_output(AllowedListsOutput::DeleteRuleSet(id)),
        AppMsg::DeleteRuleSet(x) if x == id
    ));
    assert!(matches!(
        map_allowed_lists_output(AllowedListsOutput::SetDefaultRuleSet(id)),
        AppMsg::ChooseDefaultRuleSet(x) if x == id
    ));
    assert!(matches!(
        map_allowed_lists_output(AllowedListsOutput::AddUrl {
            rule_set_id: id,
            url: "https://a".to_string(),
        }),
        AppMsg::AddUrlToList { rule_set_id, url } if rule_set_id == id && url == "https://a"
    ));
    assert!(matches!(
        map_allowed_lists_output(AllowedListsOutput::RemoveUrl {
            rule_set_id: id,
            url: "https://a".to_string(),
        }),
        AppMsg::RemoveUrlFromList { rule_set_id, url } if rule_set_id == id && url == "https://a"
    ));
}

#[test]
fn maps_schedule_outputs() {
    let id = Uuid::new_v4();
    let msg = map_schedule_output(ScheduleOutput::CreateSchedule {
        name: "Daily".to_string(),
        days: vec![1, 2],
        start_min: 60,
        end_min: 120,
        specific_date: Some("2026-03-20".to_string()),
        rule_set_id: Some(id),
        schedule_type: ScheduleType::Focus,
    });
    assert!(matches!(msg, AppMsg::CreateSchedule { .. }));

    let msg = map_schedule_output(ScheduleOutput::UpdateSchedule {
        id,
        name: "Updated".to_string(),
        days: vec![3],
        start_min: 120,
        end_min: 180,
        rule_set_id: None,
        specific_date: None,
        schedule_type: ScheduleType::Break,
    });
    assert!(matches!(msg, AppMsg::UpdateSchedule { id: x, .. } if x == id));

    assert!(matches!(
        map_schedule_output(ScheduleOutput::DeleteSchedule(id)),
        AppMsg::DeleteSchedule(x) if x == id
    ));
    assert!(matches!(
        map_schedule_output(ScheduleOutput::ResyncCalendar),
        AppMsg::ResyncCalendar
    ));
}

#[test]
fn maps_calendar_rules_outputs() {
    let add = map_calendar_rules_output(CalendarRulesOutput::AddRule {
        keyword: "meeting".to_string(),
        schedule_type: ScheduleType::Focus,
    });
    assert!(matches!(add, AppMsg::AddImportRule { .. }));

    let remove = map_calendar_rules_output(CalendarRulesOutput::RemoveRule {
        keyword: "lunch".to_string(),
        schedule_type: ScheduleType::Break,
    });
    assert!(matches!(remove, AppMsg::RemoveImportRule { .. }));
}

#[test]
fn maps_settings_outputs() {
    assert!(matches!(
        map_settings_output(SettingsOutput::StrictModeChanged(true)),
        AppMsg::StrictModeChanged(true)
    ));
    assert!(matches!(
        map_settings_output(SettingsOutput::AllowNewTabChanged(false)),
        AppMsg::AllowNewTabChanged(false)
    ));
    assert!(matches!(
        map_settings_output(SettingsOutput::AiSitesToggled(true)),
        AppMsg::AiSitesToggled(true)
    ));
    assert!(matches!(
        map_settings_output(SettingsOutput::SearchEnginesToggled(false)),
        AppMsg::SearchEnginesToggled(false)
    ));
    assert!(matches!(
        map_settings_output(SettingsOutput::QuickUrlToggled {
            url: "https://z",
            enabled: true
        }),
        AppMsg::AddUrl(url) if url == "https://z"
    ));
    assert!(matches!(
        map_settings_output(SettingsOutput::QuickUrlToggled {
            url: "https://z",
            enabled: false
        }),
        AppMsg::RemoveUrl(url) if url == "https://z"
    ));
    assert!(matches!(
        map_settings_output(SettingsOutput::CalDavSaved {
            url: "u".to_string(),
            user: "a".to_string(),
            pass: "p".to_string()
        }),
        AppMsg::SaveCalDav { .. }
    ));
    assert!(matches!(
        map_settings_output(SettingsOutput::ConnectGoogleRequested),
        AppMsg::ConnectGoogle
    ));
    assert!(matches!(
        map_settings_output(SettingsOutput::DisconnectGoogleRequested),
        AppMsg::DisconnectGoogle
    ));
}
