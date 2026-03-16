use anyhow::Result;
use axum::{routing::get, Router};
use tracing::info;

const BLOCK_PAGE_HTML: &str = include_str!("../../../block-page/index.html");

async fn block_page() -> axum::response::Html<&'static str> {
    axum::response::Html(BLOCK_PAGE_HTML)
}

pub async fn serve() -> Result<()> {
    let app = Router::new().route("/", get(block_page));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:10000").await?;
    info!("block page server listening on http://127.0.0.1:10000");
    axum::serve(listener, app).await?;
    Ok(())
}
