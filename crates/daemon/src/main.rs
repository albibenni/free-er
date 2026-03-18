mod app_state;
mod blocking;
mod calendar;
mod ipc;
mod local_server;
mod persistence;
mod pomodoro;
mod rule_matcher;

use anyhow::Result;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("free_er=debug".parse()?),
        )
        .init();

    info!("free-er daemon starting");

    let config = persistence::load().await?;
    let state = app_state::AppState::new(config);

    // Background task: advance pomodoro phases automatically.
    let tick_state = state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
        loop {
            interval.tick().await;
            tick_state.tick();
        }
    });

    // Background task: sync CalDAV calendar every 6 hours.
    let cal_state = state.clone();
    tokio::spawn(async move {
        let mut interval =
            tokio::time::interval(tokio::time::Duration::from_secs(6 * 60 * 60));
        loop {
            interval.tick().await;
            if let Some(cfg) = cal_state.caldav_config() {
                match calendar::fetch_ics(&cfg).await {
                    Ok(ics) => {
                        let default_id = cal_state.list_rule_sets().first()
                            .map(|r| r.id).unwrap_or_else(uuid::Uuid::nil);
                        let schedules = calendar::parse_schedules(&ics, &cfg, default_id);
                        info!("calendar sync: imported {} schedules", schedules.len());
                        cal_state.apply_calendar_schedules(schedules);
                    }
                    Err(e) => warn!("calendar sync failed: {e}"),
                }
            }
        }
    });

    // Background task: sync Google Calendar every 6 hours.
    let gcal_state = state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(6 * 60 * 60));
        loop {
            interval.tick().await;
            let cfg = match gcal_state.google_calendar_config() {
                Some(c) if c.access_token.is_some() => c,
                _ => continue,
            };
            // Refresh token if within 5 minutes of expiry
            let cfg = if cfg.token_expiry_secs.map(|e| e - chrono::Utc::now().timestamp() < 300).unwrap_or(true) {
                match calendar::refresh_google_token(&cfg).await {
                    Ok((token, expiry)) => {
                        gcal_state.update_google_tokens(token, expiry);
                        let s = gcal_state.config();
                        tokio::spawn(async move { let _ = persistence::save(&s).await; });
                        match gcal_state.google_calendar_config() {
                            Some(c) => c,
                            None => continue,
                        }
                    }
                    Err(e) => { warn!("Google token refresh failed: {e}"); continue; }
                }
            } else { cfg };

            let import_rules = cfg.import_rules.clone();
            let default_id = gcal_state.list_rule_sets().first()
                .map(|r| r.id).unwrap_or_else(uuid::Uuid::nil);
            match calendar::fetch_google_calendar_schedules(&cfg, &import_rules, default_id).await {
                Ok(schedules) => {
                    info!("Google Calendar sync: imported {} schedules", schedules.len());
                    gcal_state.apply_calendar_schedules(schedules);
                }
                Err(e) => warn!("Google Calendar sync failed: {e}"),
            }
        }
    });

    // Background task: activate focus / break based on user-defined schedules.
    let sched_state = state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            sched_state.apply_schedule();
        }
    });

    tokio::try_join!(
        ipc::serve(state.clone()),
        local_server::serve(state.clone()),
    )?;

    Ok(())
}
