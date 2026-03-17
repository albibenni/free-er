use anyhow::Result;
use chrono::{Datelike, NaiveDateTime, NaiveTime};
use ical::parser::ical::component::IcalEvent;
use shared::models::{CalDavConfig, CalendarImportRule, GoogleCalendarConfig, Schedule};
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
/// Title-based matching: if any `import_rule.keyword` is a case-insensitive
/// substring of the event SUMMARY, that rule's `rule_set_id` is used.
pub fn parse_schedules(ics: &str, cfg: &CalDavConfig) -> Vec<Schedule> {
    let now = chrono::Local::now().naive_local();
    let reader = ical::IcalParser::new(ics.as_bytes());
    let mut schedules = Vec::new();

    for calendar in reader.flatten() {
        for event in calendar.events {
            if let Some(schedule) = event_to_schedule(&event, cfg, now) {
                schedules.push(schedule);
            }
        }
    }
    schedules
}

fn event_to_schedule(
    event: &IcalEvent,
    cfg: &CalDavConfig,
    now: chrono::NaiveDateTime,
) -> Option<Schedule> {
    let summary = prop_value(event, "SUMMARY")?;
    let dtstart = prop_value(event, "DTSTART")?;
    let dtend = prop_value(event, "DTEND")?;

    let start_dt = parse_dt(&dtstart)?;
    let end_dt = parse_dt(&dtend)?;

    // Skip events that have already ended
    if end_dt <= now {
        return None;
    }

    // Find a matching import rule
    let rule_set_id = cfg.import_rules.iter().find_map(|rule| {
        if summary
            .to_lowercase()
            .contains(&rule.keyword.to_lowercase())
        {
            Some(rule.rule_set_id)
        } else {
            None
        }
    })?;

    // Map the event's weekday to a Schedule.
    // For recurring events this is a simplification — full RRULE expansion
    // is out of scope for Phase 4.
    let weekday = start_dt.weekday();
    let start_time = start_dt.time();
    let end_time = end_dt.time();

    Some(Schedule {
        id: Uuid::new_v4(),
        name: summary,
        days: vec![weekday],
        start: start_time,
        end: end_time,
        rule_set_id,
        enabled: true,
    })
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
) -> Result<Vec<Schedule>> {
    let access_token = cfg
        .access_token
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("no access token"))?;

    let now = chrono::Utc::now();
    // Use Z (UTC) suffix so no encoding is needed for the `+` in ±offset timestamps
    let fmt = "%Y-%m-%dT%H:%M:%SZ";
    let time_min = now.format(fmt).to_string();
    let time_max = (now + chrono::Duration::days(30)).format(fmt).to_string();
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
    let local_now = chrono::Local::now().naive_local();
    Ok(items
        .iter()
        .filter_map(|item| google_event_to_schedule(item, import_rules, local_now))
        .collect())
}

fn google_event_to_schedule(
    event: &serde_json::Value,
    import_rules: &[CalendarImportRule],
    now: chrono::NaiveDateTime,
) -> Option<Schedule> {
    let summary = event["summary"].as_str()?;

    // Use the first matching rule's rule_set_id, or nil if none match.
    // All events are imported regardless of whether a rule matches.
    let rule_set_id = import_rules
        .iter()
        .find_map(|rule| {
            if summary.to_lowercase().contains(&rule.keyword.to_lowercase()) {
                Some(rule.rule_set_id)
            } else {
                None
            }
        })
        .unwrap_or_else(Uuid::nil);

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
                .map(|d| d.and_hms_opt(0, 0, 0).unwrap())
        })
        .ok()?;
    let end_dt = chrono::DateTime::parse_from_rfc3339(end_str)
        .map(|dt| dt.naive_local())
        .or_else(|_| {
            chrono::NaiveDate::parse_from_str(end_str, "%Y-%m-%d")
                .map(|d| d.and_hms_opt(0, 0, 0).unwrap())
        })
        .ok()?;

    if end_dt <= now {
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
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::models::CalendarImportRule;

    const SAMPLE_ICS: &str = "BEGIN:VCALENDAR\r\n\
BEGIN:VEVENT\r\n\
SUMMARY:Deep Work Session\r\n\
DTSTART:29991231T090000Z\r\n\
DTEND:29991231T110000Z\r\n\
END:VEVENT\r\n\
BEGIN:VEVENT\r\n\
SUMMARY:Lunch break\r\n\
DTSTART:29991231T120000Z\r\n\
DTEND:29991231T130000Z\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

    #[test]
    fn parses_matching_events() {
        let rule_set_id = Uuid::new_v4();
        let cfg = CalDavConfig {
            url: String::new(),
            username: None,
            password: None,
            import_rules: vec![CalendarImportRule {
                keyword: "work".into(),
                rule_set_id,
            }],
        };
        let schedules = parse_schedules(SAMPLE_ICS, &cfg);
        assert_eq!(schedules.len(), 1);
        assert_eq!(schedules[0].name, "Deep Work Session");
        assert_eq!(schedules[0].rule_set_id, rule_set_id);
        assert_eq!(
            schedules[0].start,
            NaiveTime::from_hms_opt(9, 0, 0).unwrap()
        );
        assert_eq!(schedules[0].end, NaiveTime::from_hms_opt(11, 0, 0).unwrap());
    }

    #[test]
    fn skips_non_matching_events() {
        let cfg = CalDavConfig {
            url: String::new(),
            username: None,
            password: None,
            import_rules: vec![CalendarImportRule {
                keyword: "work".into(),
                rule_set_id: Uuid::new_v4(),
            }],
        };
        let schedules = parse_schedules(SAMPLE_ICS, &cfg);
        // "Lunch break" should not match
        assert!(schedules.iter().all(|s| s.name != "Lunch break"));
    }
}
