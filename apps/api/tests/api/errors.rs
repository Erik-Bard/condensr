use axum::http::StatusCode;
use condensr_api::models::error::ApiError;

use crate::common::{get, spawn_app};

#[tokio::test]
async fn internal_error_has_expected_shape() {
    let app = spawn_app().await;
    app.pool.close().await;

    let res = get(&app, "/api/links").await;
    assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let bytes = crate::common::body_bytes(res).await;
    let parsed: ApiError = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(parsed.error, "internal_server_error");
    assert!(parsed.details.is_none());

    let raw: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(raw.as_object().unwrap().get("details").is_none());
}
