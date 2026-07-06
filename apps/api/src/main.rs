use std::net::SocketAddr;

use condensr_api::{AppState, config::Config};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(debug_assertions)]
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| {
                    "condensr_api=debug,tower_http=debug,info".into()
                }),
        )
        .init();

    let app_config = Config::new().unwrap();

    let state = AppState::new(&app_config).await?;

    let app = condensr_api::build_router(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], app_config.app_port));
    tracing::info!("condensr API listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
