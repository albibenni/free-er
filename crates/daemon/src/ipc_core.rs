use crate::app_state::AppState;
use anyhow::Result;
use shared::{
    ipc::{Command, PomodoroPhase, StatusResponse},
    models::RuleSet,
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tracing::{error, info, warn};
use uuid::Uuid;

const SOCKET_PATH: &str = "/tmp/free-er.sock";

pub async fn serve(state: AppState) -> Result<()> {
    // Remove stale socket from a previous run
    let _ = std::fs::remove_file(SOCKET_PATH);

    let listener = UnixListener::bind(SOCKET_PATH)?;
    info!("IPC socket listening at {}", SOCKET_PATH);

    loop {
        let (stream, _) = listener.accept().await?;
        let state = state.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, state).await {
                error!("IPC connection error: {e}");
            }
        });
    }
}

async fn handle_connection(stream: tokio::net::UnixStream, state: AppState) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();

    while let Some(line) = lines.next_line().await? {
        let (response, mutated) = match serde_json::from_str::<Command>(&line) {
            Ok(cmd) => handle_command(cmd, &state),
            Err(e) => (format!("{{\"error\": \"{e}\"}}"), false),
        };
        writer.write_all(response.as_bytes()).await?;
        writer.write_all(b"\n").await?;

        if mutated {
            let config = state.config();
            tokio::spawn(async move {
                if let Err(e) = crate::persistence::save(&config).await {
                    warn!("Failed to persist config: {e}");
                }
            });
        }
    }
    Ok(())
}

fn handle_command(cmd: Command, state: &AppState) -> (String, bool) {
    match cmd {
        Command::StartFocus { rule_set_id } => {
            state.start_focus(rule_set_id);
            ok(false)
        }
        Command::StopFocus => {
            state.stop_focus();
            ok(false)
        }
        Command::TakeBreak { duration_secs } => {
            state.start_pomodoro(duration_secs, 0, None);
            ok(false)
        }
        Command::StartPomodoro {
            focus_secs,
            break_secs,
            rule_set_id,
        } => {
            state.start_pomodoro(focus_secs, break_secs, rule_set_id);
            ok(false)
        }
        Command::StopPomodoro => {
            state.stop_pomodoro();
            ok(false)
        }
        Command::SkipBreak => {
            if state.skip_break() {
                ok(false)
            } else {
                (r#"{"error": "strict breaks are enabled"}"#.into(), false)
            }
        }
        Command::GetStatus => {
            let snap = state.snapshot();
            let resp = StatusResponse {
                focus_active: snap.focus_active,
                strict_mode: snap.strict_mode,
                allow_new_tab: snap.allow_new_tab,
                active_rule_set_name: snap.active_rule_set_name,
                pomodoro_active: snap.pomodoro_active,
                pomodoro_phase: snap.pomodoro_phase.map(|p| match p {
                    crate::pomodoro::Phase::Focus => PomodoroPhase::Focus,
                    crate::pomodoro::Phase::Break => PomodoroPhase::Break,
                }),
                seconds_remaining: snap.seconds_remaining,
                google_calendar_connected: snap.google_calendar_connected,
                default_rule_set_id: snap.default_rule_set_id,
                accent_color: snap.accent_color,
            };
            (
                serde_json::to_string(&resp).unwrap_or_else(|e| format!("{{\"error\": \"{e}\"}}")),
                false,
            )
        }
        Command::AddRuleSet { name, allowed_urls } => {
            let mut rs = RuleSet::new(name);
            rs.allowed_urls = allowed_urls;
            let id = rs.id;
            state.add_rule_set(rs);
            (
                serde_json::json!({ "ok": true, "id": id }).to_string(),
                true,
            )
        }
        Command::RemoveRuleSet { id } => {
            state.remove_rule_set(id);
            ok(true)
        }
        Command::AddUrlToRuleSet { rule_set_id, url } => {
            if state.add_url_to_rule_set(rule_set_id, url) {
                ok(true)
            } else {
                (r#"{"error": "rule set not found"}"#.into(), false)
            }
        }
        Command::RemoveUrlFromRuleSet { rule_set_id, url } => {
            if state.remove_url_from_rule_set(rule_set_id, &url) {
                ok(true)
            } else {
                (r#"{"error": "rule set not found"}"#.into(), false)
            }
        }
        Command::ListRuleSets => {
            let rule_sets: Vec<shared::ipc::RuleSetSummary> = state
                .list_rule_sets()
                .into_iter()
                .map(|rs| shared::ipc::RuleSetSummary {
                    id: rs.id,
                    name: rs.name,
                    allowed_urls: rs.allowed_urls,
                })
                .collect();
            (
                serde_json::to_string(&rule_sets)
                    .unwrap_or_else(|e| format!("{{\"error\": \"{e}\"}}")),
                false,
            )
        }
        Command::SetDefaultRuleSet { id } => {
            if state.set_default_rule_set(id) {
                ok(true)
            } else {
                (r#"{"error": "rule set not found"}"#.into(), false)
            }
        }
        Command::AddSchedule {
            name,
            days,
            start_min,
            end_min,
            rule_set_id,
            specific_date,
            schedule_type,
        } => {
            use chrono::NaiveTime;
            fn wday(d: u8) -> Option<chrono::Weekday> {
                match d {
                    0 => Some(chrono::Weekday::Mon),
                    1 => Some(chrono::Weekday::Tue),
                    2 => Some(chrono::Weekday::Wed),
                    3 => Some(chrono::Weekday::Thu),
                    4 => Some(chrono::Weekday::Fri),
                    5 => Some(chrono::Weekday::Sat),
                    6 => Some(chrono::Weekday::Sun),
                    _ => None,
                }
            }
            let start =
                NaiveTime::from_hms_opt(start_min / 60, start_min % 60, 0).unwrap_or_default();
            let end = NaiveTime::from_hms_opt(end_min / 60, end_min % 60, 0).unwrap_or_default();
            let weekdays = days.iter().filter_map(|&d| wday(d)).collect();
            let parsed_date = specific_date
                .as_deref()
                .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
            let schedule = shared::models::Schedule {
                id: Uuid::new_v4(),
                name,
                days: weekdays,
                start,
                end,
                rule_set_id: rule_set_id.unwrap_or_else(Uuid::nil),
                enabled: true,
                imported: false,
                imported_repeating: false,
                specific_date: parsed_date,
                schedule_type,
            };
            let id = schedule.id;
            state.add_schedule(schedule);
            (
                serde_json::json!({ "ok": true, "id": id }).to_string(),
                true,
            )
        }
        Command::RemoveSchedule { id } => {
            state.remove_schedule(id);
            ok(true)
        }
        Command::UpdateSchedule {
            id,
            name,
            days,
            start_min,
            end_min,
            rule_set_id,
            specific_date,
            schedule_type,
        } => {
            use chrono::NaiveTime;
            fn wday(d: u8) -> Option<chrono::Weekday> {
                match d {
                    0 => Some(chrono::Weekday::Mon),
                    1 => Some(chrono::Weekday::Tue),
                    2 => Some(chrono::Weekday::Wed),
                    3 => Some(chrono::Weekday::Thu),
                    4 => Some(chrono::Weekday::Fri),
                    5 => Some(chrono::Weekday::Sat),
                    6 => Some(chrono::Weekday::Sun),
                    _ => None,
                }
            }
            let start =
                NaiveTime::from_hms_opt(start_min / 60, start_min % 60, 0).unwrap_or_default();
            let end = NaiveTime::from_hms_opt(end_min / 60, end_min % 60, 0).unwrap_or_default();
            let weekdays = days.iter().filter_map(|&d| wday(d)).collect();
            let new_specific_date =
                specific_date.and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok());
            state.update_schedule(
                id,
                name,
                weekdays,
                start,
                end,
                rule_set_id,
                new_specific_date,
                schedule_type,
            );
            ok(true)
        }
        Command::ListSchedules => {
            use chrono::Timelike;
            let summaries: Vec<shared::ipc::ScheduleSummary> = state
                .list_schedules()
                .into_iter()
                .map(|s| shared::ipc::ScheduleSummary {
                    id: s.id,
                    name: s.name,
                    days: s
                        .days
                        .iter()
                        .map(|d| d.num_days_from_monday() as u8)
                        .collect(),
                    start_min: s.start.hour() * 60 + s.start.minute(),
                    end_min: s.end.hour() * 60 + s.end.minute(),
                    enabled: s.enabled,
                    imported: s.imported,
                    imported_repeating: s.imported_repeating,
                    specific_date: s.specific_date.map(|d| d.format("%Y-%m-%d").to_string()),
                    schedule_type: s.schedule_type,
                    rule_set_id: s.rule_set_id,
                })
                .collect();
            (
                serde_json::to_string(&summaries)
                    .unwrap_or_else(|e| format!("{{\"error\": \"{e}\"}}")),
                false,
            )
        }
        Command::SetStrictMode { enabled } => {
            state.set_strict_mode(enabled);
            ok(true)
        }
        Command::SetAllowNewTab { enabled } => {
            state.set_allow_new_tab(enabled);
            ok(true)
        }
        Command::SetAccentColor { hex } => {
            state.set_accent_color(hex);
            ok(true)
        }
        Command::GetOpenTabs => {
            let tabs: Vec<shared::ipc::OpenTab> = state
                .get_open_tabs()
                .into_iter()
                .map(|(url, title)| shared::ipc::OpenTab { url, title })
                .collect();
            (
                serde_json::to_string(&tabs)
                    .unwrap_or_else(|e| format!("{{\"error\": \"{e}\"}}")),
                false,
            )
        }
        Command::SetCalDav {
            url,
            username,
            password,
        } => {
            state.set_caldav(url, username, password);
            ok(true)
        }
        Command::StartGoogleOAuth { .. } => {
            let (client_id, client_secret) = match crate::persistence::load_google_client() {
                Some(c) => c,
                None => {
                    return (
                        r#"{"error":"google_client.json not found — see README"}"#.into(),
                        false,
                    )
                }
            };
            let csrf: String = (0..16)
                .map(|_| format!("{:02x}", rand::random::<u8>()))
                .collect();
            state.set_pending_oauth_state(csrf.clone(), client_id.clone(), client_secret);
            let auth_url = format!(
                "https://accounts.google.com/o/oauth2/v2/auth\
                 ?client_id={client_id}\
                 &redirect_uri=http%3A%2F%2F127.0.0.1%3A10000%2Foauth%2Fgoogle%2Fcallback\
                 &response_type=code\
                 &scope=https%3A%2F%2Fwww.googleapis.com%2Fauth%2Fcalendar.readonly\
                 &access_type=offline\
                 &prompt=consent\
                 &state={csrf}"
            );
            (
                serde_json::json!({ "auth_url": auth_url }).to_string(),
                false,
            )
        }
        Command::RevokeGoogleCalendar => {
            state.revoke_google_calendar();
            ok(true)
        }
        Command::SyncCalendar => {
            let import_rules = state.list_import_rules();
            // CalDAV sync
            if let Some(cfg) = state.caldav_config() {
                let s = state.clone();
                let rules = import_rules.clone();
                tokio::spawn(async move {
                    match crate::calendar::fetch_ics(&cfg).await {
                        Ok(ics) => {
                            let default_id = s.effective_default_rule_set_id();
                            let schedules =
                                crate::calendar::parse_schedules(&ics, &rules, default_id);
                            tracing::info!(
                                "calendar sync (manual): imported {} schedules",
                                schedules.len()
                            );
                            s.apply_calendar_schedules(schedules);
                        }
                        Err(e) => tracing::warn!("CalDAV manual sync failed: {e}"),
                    }
                });
            }
            // Google Calendar sync
            if let Some(cfg) = state.google_calendar_config() {
                let s = state.clone();
                let rules = import_rules.clone();
                tokio::spawn(async move {
                    let default_id = s.effective_default_rule_set_id();
                    match crate::calendar::fetch_google_calendar_schedules(&cfg, &rules, default_id)
                        .await
                    {
                        Ok(schedules) => {
                            tracing::info!(
                                "Google Calendar sync (manual): imported {} schedules",
                                schedules.len()
                            );
                            s.apply_calendar_schedules(schedules);
                        }
                        Err(e) => tracing::warn!("Google Calendar manual sync failed: {e}"),
                    }
                });
            }
            ok(false)
        }
        Command::AddImportRule {
            keyword,
            schedule_type,
        } => {
            state.add_import_rule(keyword, schedule_type);
            ok(true)
        }
        Command::RemoveImportRule {
            keyword,
            schedule_type,
        } => {
            state.remove_import_rule(&keyword, &schedule_type);
            ok(true)
        }
        Command::Shutdown => {
            tokio::spawn(async {
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                std::process::exit(0);
            });
            ok(false)
        }
        Command::ListImportRules => {
            let rules: Vec<shared::ipc::ImportRuleSummary> = state
                .list_import_rules()
                .into_iter()
                .map(|r| shared::ipc::ImportRuleSummary {
                    keyword: r.keyword,
                    schedule_type: r.schedule_type,
                })
                .collect();
            (
                serde_json::to_string(&rules).unwrap_or_else(|e| format!("{{\"error\": \"{e}\"}}")),
                false,
            )
        }
    }
}

fn ok(mutated: bool) -> (String, bool) {
    (r#"{"ok": true}"#.into(), mutated)
}

#[cfg(test)]
#[path = "ipc_tests.rs"]
mod tests;
