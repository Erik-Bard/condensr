use sqlx::{Postgres, migrate::MigrateDatabase};

use crate::common::{spawn_app, unique_database_url};

#[tokio::test]
async fn startup_applies_embedded_migrations() {
    let app = spawn_app().await;
    let migration_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM _sqlx_migrations")
            .fetch_one(&app.pool)
            .await
            .unwrap();

    assert_eq!(migration_count, 2);
}

#[tokio::test]
async fn startup_does_not_create_a_missing_database() {
    let database_url = unique_database_url("missing");
    assert!(
        !Postgres::database_exists(database_url.as_str())
            .await
            .unwrap()
    );

    let result =
        condensr_api::database::pg_database::connect(database_url.as_str())
            .await;

    assert!(result.is_err());
    assert!(
        !Postgres::database_exists(database_url.as_str())
            .await
            .unwrap()
    );
}
