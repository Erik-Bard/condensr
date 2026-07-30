use anyhow::Context;
use sqlx::{PgPool, migrate::Migrator, postgres::PgPoolOptions};

pub static MIGRATOR: Migrator = sqlx::migrate!();

pub async fn connect(url: &str) -> Result<PgPool, anyhow::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(url)
        .await
        .context("failed to connect to existing database")?;
    MIGRATOR
        .run(&pool)
        .await
        .context("failed to run database migrations")?;
    Ok(pool)
}
