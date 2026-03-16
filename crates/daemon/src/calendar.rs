use anyhow::{Context, Result};
use chrono::{Datelike, NaiveDateTime, NaiveTime, Timelike, Weekday};
use ical::parser::ical::component::IcalEvent;
use shared::models::{CalDavConfig, Schedule};
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
        assert_eq!(schedules[0].start, NaiveTime::from_hms_opt(9, 0, 0).unwrap());
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
