use std::net::SocketAddr;

use axum::{Json, Router, http::StatusCode, response::IntoResponse, routing::get};
use condensr_api::{AppState, config::Config, routes};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(debug_assertions)]
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "condensr_api=debug,tower_http=debug,info".into()),
        )
        .init();

    let app_config = Config::new().unwrap();

    let state = AppState::new(&app_config).await?;

    let app = Router::new()
        .route("/health", get(health))
        .merge(routes::shorten::router())
        .merge(routes::links::router())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], app_config.app_port));
    tracing::info!("condensr API listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({ "status": "ok" })))
}
