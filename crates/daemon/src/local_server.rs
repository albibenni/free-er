use crate::app_state::AppState;
use anyhow::Result;
use axum::{
    extract::State,
    response::Json,
    routing::get,
    Router,
};
use serde::Serialize;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

const BLOCK_PAGE_HTML: &str = include_str!("../../../block-page/index.html");

async fn block_page() -> axum::response::Html<&'static str> {
    axum::response::Html(BLOCK_PAGE_HTML)
}

/// Response polled by the browser extension.
#[derive(Serialize)]
struct ApiStatus {
    focus_active: bool,
    /// Allowed URL patterns. Empty = block everything except nothing (i.e. all blocked).
    allowed_urls: Vec<String>,
}

async fn api_status(State(state): State<AppState>) -> Json<ApiStatus> {
    let snap = state.snapshot();
    let allowed_urls = if snap.focus_active {
        state
            .active_rule_set()
            .map(|rs| rs.allowed_urls)
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    Json(ApiStatus {
        focus_active: snap.focus_active,
        allowed_urls,
    })
}

pub async fn serve(state: AppState) -> Result<()> {
    // Allow the browser extension (any origin) to call /api/status
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(block_page))
        .route("/api/status", get(api_status))
        .with_state(state)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:10000").await?;
    info!("block page + API server listening on http://127.0.0.1:10000");
    axum::serve(listener, app).await?;
    Ok(())
}
