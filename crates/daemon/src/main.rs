mod app_state;
mod blocking;
mod ipc;
mod local_server;
mod persistence;
mod pomodoro;
mod rule_matcher;

use anyhow::Result;
use tracing::info;

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

    tokio::try_join!(
        ipc::serve(state.clone()),
        local_server::serve(),
    )?;

    Ok(())
}
