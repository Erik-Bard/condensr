use anyhow::Context;
use sqlx::{
    Connection, PgConnection, PgPool, Postgres,
    migrate::{MigrateDatabase, Migrator},
    postgres::PgPoolOptions,
};
use tracing::info;

pub static MIGRATOR: Migrator = sqlx::migrate!();

pub async fn connect(url: &str) -> Result<PgPool, anyhow::Error> {
    check_for_migrations(url).await?;

    PgPoolOptions::new()
        .max_connections(10)
        .connect(url)
        .await
        .context("failed to connect to database")
}

async fn check_for_migrations(uri: &str) -> Result<(), anyhow::Error> {
    if !Postgres::database_exists(uri)
        .await
        .context("failed to check if database exists")?
    {
        info!("Creating database...");
        Postgres::create_database(uri)
            .await
            .context("failed to create database")?;
    }

    info!("Applying migrations...");

    let mut conn = PgConnection::connect(uri)
        .await
        .context("failed to connect to database for migrations")?;

    MIGRATOR
        .run(&mut conn)
        .await
        .context("failed to run database migrations")?;

    conn.close()
        .await
        .context("failed to close migration connection")?;

    Ok(())
}
