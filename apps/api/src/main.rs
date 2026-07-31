use std::net::SocketAddr;

use condensr_api::{AppState, config::Config};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app_config = Config::load()?;

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_new(
            &app_config.log_filter,
        )?)
        .init();

    let state = AppState::new(&app_config).await?;

    let app = condensr_api::build_router(state, &app_config.http);

    let addr = SocketAddr::from(([0, 0, 0, 0], app_config.app_port));
    tracing::info!("condensr API listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}
