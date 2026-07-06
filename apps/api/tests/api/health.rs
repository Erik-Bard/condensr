use crate::common::{body_json, get, spawn_app};

#[tokio::test]
async fn health_returns_ok_status() {
    let app = spawn_app().await;

    let res = get(&app, "/health").await;
    assert_eq!(res.status(), 200);

    let content_type = res
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    assert!(content_type.starts_with("application/json"));

    let body = body_json(res).await;
    assert_eq!(body, serde_json::json!({ "status": "ok" }));
}
