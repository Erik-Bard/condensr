use std::sync::atomic::{AtomicU32, Ordering};

use axum::{
    Router,
    body::Body,
    http::{Request, Response, StatusCode},
};
use condensr_api::AppState;
use http_body_util::BodyExt;
use sqlx::{Connection, PgConnection, PgPool};
use tokio::sync::Mutex;
use tower::ServiceExt;
use url::Url;

static DB_SETUP: Mutex<()> = Mutex::const_new(());
static DB_COUNTER: AtomicU32 = AtomicU32::new(0);

pub struct TestApp {
    pub router: Router,
    pub pool: PgPool,
    pub base_url: String,
    db_name: String,
    admin_url: String,
}

fn server_url() -> Url {
    let _ = dotenvy::dotenv();
    let raw = std::env::var("TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .unwrap_or_else(|_| {
            "postgres://condensr:condensr@localhost:5432/postgres".into()
        });
    Url::parse(&raw).expect("database url must be a valid URL")
}

pub async fn spawn_app() -> TestApp {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    let db_name = format!(
        "condensr_test_{}_{}_{}",
        std::process::id(),
        DB_COUNTER.fetch_add(1, Ordering::Relaxed),
        nanos
    );

    let mut db_url = server_url();
    db_url.set_path(&db_name);
    let mut admin_url = server_url();
    admin_url.set_path("postgres");

    let pool = {
        let _guard = DB_SETUP.lock().await;
        condensr_api::database::pg_database::connect(db_url.as_str())
            .await
            .expect(
                "failed to connect to Postgres; run `docker compose up -d db` first",
            )
    };

    let base_url = "http://testhost".to_string();
    let state = AppState {
        db: pool.clone(),
        base_url: base_url.clone(),
    };

    TestApp {
        router: condensr_api::build_router(state),
        pool,
        base_url,
        db_name,
        admin_url: admin_url.into(),
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        let db_name = self.db_name.clone();
        let admin_url = self.admin_url.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async move {
                if let Ok(mut conn) = PgConnection::connect(&admin_url).await {
                    let stmt = format!(
                        "DROP DATABASE IF EXISTS \"{db_name}\" WITH (FORCE)"
                    );
                    let _ = sqlx::query(&stmt).execute(&mut conn).await;
                    let _ = conn.close().await;
                }
            });
        })
        .join()
        .ok();
    }
}

pub async fn send(app: &TestApp, req: Request<Body>) -> Response<Body> {
    app.router.clone().oneshot(req).await.unwrap()
}

pub async fn get(app: &TestApp, path: &str) -> Response<Body> {
    let req = Request::builder()
        .method("GET")
        .uri(path)
        .body(Body::empty())
        .unwrap();
    send(app, req).await
}

pub async fn post_json(
    app: &TestApp,
    path: &str,
    body: serde_json::Value,
) -> Response<Body> {
    let req = Request::builder()
        .method("POST")
        .uri(path)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();
    send(app, req).await
}

pub async fn body_json(res: Response<Body>) -> serde_json::Value {
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

pub async fn body_bytes(res: Response<Body>) -> Vec<u8> {
    res.into_body().collect().await.unwrap().to_bytes().to_vec()
}

pub async fn shorten(
    app: &TestApp,
    url: &str,
) -> (StatusCode, serde_json::Value) {
    let res =
        post_json(app, "/api/shorten", serde_json::json!({ "url": url })).await;
    let status = res.status();
    (status, body_json(res).await)
}
