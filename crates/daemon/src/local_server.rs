use crate::app_state::AppState;
use anyhow::Result;
use axum::{
    extract::{Query, State},
    response::Json,
    routing::get,
    Router,
};
use serde::Serialize;
use std::collections::HashMap;
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

async fn oauth_google_callback(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> axum::response::Html<String> {
    let code = match params.get("code") {
        Some(c) => c.clone(),
        None => return axum::response::Html("<h1>Error: missing code parameter</h1>".into()),
    };
    let received_state = match params.get("state") {
        Some(s) => s.clone(),
        None => return axum::response::Html("<h1>Error: missing state parameter</h1>".into()),
    };

    let (client_id, client_secret) = match state.take_pending_oauth(&received_state) {
        Some(creds) => creds,
        None => {
            return axum::response::Html("<h1>Error: invalid or expired OAuth state</h1>".into())
        }
    };

    let body = format!(
        "code={}&client_id={}&client_secret={}&redirect_uri={}&grant_type=authorization_code",
        code, client_id, client_secret,
        "http%3A%2F%2F127.0.0.1%3A10000%2Foauth%2Fgoogle%2Fcallback"
    );
    let client = reqwest::Client::new();
    let token_resp = client
        .post("https://oauth2.googleapis.com/token")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await;

    let json: serde_json::Value = match token_resp {
        Err(e) => return axum::response::Html(format!("<h1>Token exchange failed: {e}</h1>")),
        Ok(r) => match r.json().await {
            Ok(j) => j,
            Err(e) => {
                return axum::response::Html(format!(
                    "<h1>Failed to parse token response: {e}</h1>"
                ))
            }
        },
    };

    let access_token = json["access_token"].as_str().unwrap_or("").to_string();
    let refresh_token = json["refresh_token"].as_str().unwrap_or("").to_string();
    let expires_in = json["expires_in"].as_i64().unwrap_or(3600);

    if access_token.is_empty() {
        return axum::response::Html(format!("<h1>Error: no access_token — {json}</h1>"));
    }

    let expiry_secs = chrono::Utc::now().timestamp() + expires_in;
    state.set_google_calendar_tokens(
        client_id,
        client_secret,
        access_token,
        refresh_token,
        expiry_secs,
    );

    let config = state.config();
    tokio::spawn(async move {
        if let Err(e) = crate::persistence::save(&config).await {
            tracing::warn!("Failed to persist Google tokens: {e}");
        }
    });

    // Trigger an immediate calendar sync instead of waiting up to 15 minutes.
    let sync_state = state.clone();
    tokio::spawn(async move {
        if let Some(cfg) = sync_state.google_calendar_config() {
            let import_rules = cfg.import_rules.clone();
            let default_id = sync_state.list_rule_sets().first()
                .map(|r| r.id).unwrap_or_else(uuid::Uuid::nil);
            match crate::calendar::fetch_google_calendar_schedules(&cfg, &import_rules, default_id).await {
                Ok(schedules) => {
                    info!("Google Calendar initial sync: imported {} schedules", schedules.len());
                    sync_state.apply_calendar_schedules(schedules);
                }
                Err(e) => tracing::warn!("Google Calendar initial sync failed: {e}"),
            }
        }
    });

    axum::response::Html(
        "<html><body style='font-family:sans-serif;text-align:center;padding:4rem'>\
         <h1>✓ Google Calendar connected!</h1>\
         <p>You can close this tab and return to free-er.</p>\
         </body></html>"
            .into(),
    )
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
        .route("/oauth/google/callback", get(oauth_google_callback))
        .with_state(state)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:10000").await?;
    info!("block page + API server listening on http://127.0.0.1:10000");
    axum::serve(listener, app).await?;
    Ok(())
}
