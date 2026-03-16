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

    // Background task: sync CalDAV calendar every 15 minutes.
    let cal_state = state.clone();
    tokio::spawn(async move {
        let mut interval =
            tokio::time::interval(tokio::time::Duration::from_secs(15 * 60));
        loop {
            interval.tick().await;
            if let Some(cfg) = cal_state.caldav_config() {
                match calendar::fetch_ics(&cfg).await {
                    Ok(ics) => {
                        let schedules = calendar::parse_schedules(&ics, &cfg);
                        info!("calendar sync: imported {} schedules", schedules.len());
                        cal_state.apply_calendar_schedules(schedules);
                    }
                    Err(e) => warn!("calendar sync failed: {e}"),
                }
            }
        }
    });

    tokio::try_join!(
        ipc::serve(state.clone()),
        local_server::serve(state.clone()),
    )?;

    Ok(())
}
