use anyhow::Result;
use chrono::{Datelike, NaiveDateTime, NaiveTime};
use ical::parser::ical::component::IcalEvent;
use shared::models::{
    CalDavConfig, CalendarImportRule, GoogleCalendarConfig, Schedule, ScheduleType,
};
use uuid::Uuid;

/// Fetch raw ICS text from a URL (with optional basic auth).
pub async fn fetch_ics(cfg: &CalDavConfig) -> Result<String> {
    let client = reqwest::Client::new();
    let mut req = client.get(&cfg.url);
    if let (Some(user), Some(pass)) = (&cfg.username, &cfg.password) {
        req = req.basic_auth(user, Some(pass));
    }
    let text = req.send().await?.error_for_status()?.text().await?;
    Ok(text)
}

/// Parse ICS text and convert matching events into `Schedule` entries.
///
/// Only future / ongoing events are included; past events are skipped.
/// Global `import_rules` determine whether each event is Focus or Break and
/// which allowed list to apply.
pub fn parse_schedules(
    ics: &str,
    import_rules: &[CalendarImportRule],
    default_rule_set_id: Uuid,
) -> Vec<Schedule> {
    let (window_start, window_end) = schedule_window_bounds();
    let reader = ical::IcalParser::new(ics.as_bytes());
    let mut schedules = Vec::new();

    for calendar in reader.flatten() {
        for event in calendar.events {
            if let Some(schedule) = event_to_schedule(
                &event,
                import_rules,
                window_start,
                window_end,
                default_rule_set_id,
            ) {
                schedules.push(schedule);
            }
        }
    }
    schedules
}

fn event_to_schedule(
    event: &IcalEvent,
    import_rules: &[CalendarImportRule],
    window_start: chrono::NaiveDateTime,
    window_end: chrono::NaiveDateTime,
    default_rule_set_id: Uuid,
) -> Option<Schedule> {
    let summary = prop_value(event, "SUMMARY")?;
    let dtstart = prop_value(event, "DTSTART")?;
    let dtend = prop_value(event, "DTEND")?;

    let start_dt = parse_dt(&dtstart)?;
    let end_dt = parse_dt(&dtend)?;

    // Keep only events from previous/current/next week in local time.
    if start_dt < window_start || start_dt >= window_end {
        return None;
    }

    let (schedule_type, rule_set_id) = resolve_rule(&summary, import_rules, default_rule_set_id);

    // Map the event's weekday to a Schedule.
    // For recurring events this is a simplification — full RRULE expansion
    // is out of scope for Phase 4.
    let weekday = start_dt.weekday();
    let start_time = start_dt.time();
    let end_time = end_dt.time();
    let imported_repeating = event.properties.iter().any(|p| p.name == "RRULE");

    Some(Schedule {
        id: Uuid::new_v4(),
        name: summary,
        days: vec![weekday],
        start: start_time,
        end: end_time,
        rule_set_id,
        enabled: true,
        imported: true,
        imported_repeating,
        specific_date: Some(start_dt.date()),
        schedule_type,
    })
}

/// Match a summary against import rules. Returns (ScheduleType, rule_set_id).
/// Focus events use the rule's rule_set_id override or fall back to default.
/// Break events use nil uuid (no allowed list needed).
fn resolve_rule(
    summary: &str,
    import_rules: &[CalendarImportRule],
    default_rule_set_id: Uuid,
) -> (ScheduleType, Uuid) {
    let lower = summary.to_lowercase();
    if let Some(rule) = import_rules
        .iter()
        .find(|r| lower.contains(&r.keyword))
    {
        let rule_set_id = match rule.schedule_type {
            ScheduleType::Focus => rule.rule_set_id.unwrap_or(default_rule_set_id),
            ScheduleType::Break => Uuid::nil(),
        };
        (rule.schedule_type.clone(), rule_set_id)
    } else {
        // Default: Focus with default allowed list
        (ScheduleType::Focus, default_rule_set_id)
    }
}

fn prop_value(event: &IcalEvent, name: &str) -> Option<String> {
    event
        .properties
        .iter()
        .find(|p| p.name == name)
        .and_then(|p| p.value.clone())
}

/// Parse both compact (`20260316T090000Z`) and date-only (`20260316`) formats.
fn parse_dt(s: &str) -> Option<NaiveDateTime> {
    // Strip trailing Z (UTC marker) — we treat all times as local for simplicity
    let s = s.trim_end_matches('Z');
    if s.len() == 15 {
        NaiveDateTime::parse_from_str(s, "%Y%m%dT%H%M%S").ok()
    } else if s.len() == 8 {
        let date = chrono::NaiveDate::parse_from_str(s, "%Y%m%d").ok()?;
        Some(date.and_time(NaiveTime::from_hms_opt(0, 0, 0)?))
    } else {
        None
    }
}

// ── Google Calendar API ───────────────────────────────────────────────────────

/// Refresh the Google OAuth2 access token. Returns (new_access_token, expiry_unix_secs).
pub async fn refresh_google_token(cfg: &GoogleCalendarConfig) -> Result<(String, i64)> {
    let refresh_token = cfg
        .refresh_token
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("no refresh token stored"))?;
    let body = format!(
        "client_id={}&client_secret={}&refresh_token={}&grant_type=refresh_token",
        cfg.client_id, cfg.client_secret, refresh_token
    );
    let resp: serde_json::Value = reqwest::Client::new()
        .post("https://oauth2.googleapis.com/token")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let access_token = resp["access_token"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("no access_token in refresh response"))?
        .to_string();
    let expires_in = resp["expires_in"].as_i64().unwrap_or(3600);
    Ok((access_token, chrono::Utc::now().timestamp() + expires_in))
}

/// Fetch events from the primary Google Calendar and convert matching ones to Schedules.
pub async fn fetch_google_calendar_schedules(
    cfg: &GoogleCalendarConfig,
    import_rules: &[CalendarImportRule],
    default_rule_set_id: Uuid,
) -> Result<Vec<Schedule>> {
    let access_token = cfg
        .access_token
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("no access token"))?;

    let (window_start_local, window_end_local) = schedule_window_bounds();
    // Use Z (UTC) suffix so no encoding is needed for the `+` in ±offset timestamps
    let fmt = "%Y-%m-%dT%H:%M:%SZ";
    let time_min = window_start_local.and_utc().format(fmt).to_string();
    let time_max = window_end_local.and_utc().format(fmt).to_string();
    let url = format!(
        "https://www.googleapis.com/calendar/v3/calendars/primary/events\
         ?singleEvents=true&orderBy=startTime&timeMin={time_min}&timeMax={time_max}"
    );

    let resp: serde_json::Value = reqwest::Client::new()
        .get(&url)
        .bearer_auth(access_token)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let items = resp["items"].as_array().cloned().unwrap_or_default();
    // Every instance is stored with its specific_date. The calendar view shows
    // whichever events fall in the displayed week. Recurring events naturally
    // appear on every week within the fetched window; one-time events appear once.
    Ok(items
        .iter()
        .filter_map(|item| {
            google_event_to_schedule(
                item,
                import_rules,
                window_start_local,
                window_end_local,
                default_rule_set_id,
            )
        })
        .collect())
}

fn google_event_to_schedule(
    event: &serde_json::Value,
    import_rules: &[CalendarImportRule],
    window_start: chrono::NaiveDateTime,
    window_end: chrono::NaiveDateTime,
    default_rule_set_id: Uuid,
) -> Option<Schedule> {
    let summary = event["summary"].as_str()?;

    let (schedule_type, rule_set_id) = resolve_rule(summary, import_rules, default_rule_set_id);

    let start_str = event["start"]["dateTime"]
        .as_str()
        .or_else(|| event["start"]["date"].as_str())?;
    let end_str = event["end"]["dateTime"]
        .as_str()
        .or_else(|| event["end"]["date"].as_str())?;

    let start_dt = chrono::DateTime::parse_from_rfc3339(start_str)
        .map(|dt| dt.naive_local())
        .or_else(|_| {
            chrono::NaiveDate::parse_from_str(start_str, "%Y-%m-%d")
                .map(|d| chrono::NaiveDateTime::new(d, chrono::NaiveTime::MIN))
        })
        .ok()?;
    let end_dt = chrono::DateTime::parse_from_rfc3339(end_str)
        .map(|dt| dt.naive_local())
        .or_else(|_| {
            chrono::NaiveDate::parse_from_str(end_str, "%Y-%m-%d")
                .map(|d| chrono::NaiveDateTime::new(d, chrono::NaiveTime::MIN))
        })
        .ok()?;

    if start_dt < window_start || start_dt >= window_end {
        return None;
    }

    Some(Schedule {
        id: Uuid::new_v4(),
        name: summary.to_string(),
        days: vec![start_dt.weekday()],
        start: start_dt.time(),
        end: end_dt.time(),
        rule_set_id,
        enabled: true,
        imported: true,
        imported_repeating: event.get("recurringEventId").is_some()
            || event.get("recurrence").and_then(|v| v.as_array()).is_some(),
        specific_date: Some(start_dt.date()),
        schedule_type,
    })
}

/// Inclusive lower bound and exclusive upper bound for the schedule view window:
/// previous week Monday 00:00 local → week after next Monday 00:00 local.
fn schedule_window_bounds() -> (chrono::NaiveDateTime, chrono::NaiveDateTime) {
    let today = chrono::Local::now().date_naive();
    let days_from_mon = today.weekday().num_days_from_monday() as i64;
    let this_monday = today - chrono::Duration::days(days_from_mon);
    let prev_monday = this_monday - chrono::Duration::weeks(1);
    let week_after_next_monday = this_monday + chrono::Duration::weeks(2);

    (
        chrono::NaiveDateTime::new(prev_monday, chrono::NaiveTime::MIN),
        chrono::NaiveDateTime::new(week_after_next_monday, chrono::NaiveTime::MIN),
    )
}

#[cfg(test)]
#[path = "calendar_tests.rs"]
mod tests;
