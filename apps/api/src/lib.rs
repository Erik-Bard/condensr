pub mod config;
pub mod database;
pub mod models;
pub mod routes;

use sqlx::PgPool;

use crate::{config::Config, database::pg_database};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub base_url: String,
}

impl AppState {
    pub async fn new(config: &Config) -> Result<Self, anyhow::Error> {
        Ok(AppState {
            db: pg_database::connect(&config.database_url).await?,
            base_url: config.base_url.to_string(),
        })
    }
}
