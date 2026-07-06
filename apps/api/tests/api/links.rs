use axum::http::StatusCode;

use crate::common::{get, shorten, spawn_app};

#[tokio::test]
async fn empty_database_returns_empty_list() {
    let app = spawn_app().await;

    let res = get(&app, "/api/links").await;
    assert_eq!(res.status(), StatusCode::OK);
    let body = crate::common::body_json(res).await;
    assert_eq!(body, serde_json::json!([]));
}

#[tokio::test]
async fn list_entries_have_expected_shape() {
    let app = spawn_app().await;

    shorten(&app, "https://example.com/a").await;
    shorten(&app, "https://example.com/b").await;
    shorten(&app, "https://example.com/c").await;

    let res = get(&app, "/api/links").await;
    assert_eq!(res.status(), StatusCode::OK);
    let body = crate::common::body_json(res).await;
    let items = body.as_array().unwrap();
    assert_eq!(items.len(), 3);

    for item in items {
        let id = item["id"].as_u64().unwrap();
        let code = item["code"].as_str().unwrap();
        let long_url = item["long_url"].as_str().unwrap();
        let created_at = item["created_at"].as_str().unwrap();
        let short_url = item["short_url"].as_str().unwrap();

        assert!(chrono::DateTime::parse_from_rfc3339(created_at).is_ok());
        assert_eq!(short_url, format!("{}/{}", app.base_url, code));
        assert_eq!(condensr_core::decode(code).unwrap(), id);
        assert!(long_url.starts_with("https://example.com/"));
    }
}

#[tokio::test]
async fn list_is_ordered_by_insertion() {
    let app = spawn_app().await;

    shorten(&app, "https://example.com/first").await;
    shorten(&app, "https://example.com/second").await;
    shorten(&app, "https://example.com/third").await;

    let res = get(&app, "/api/links").await;
    let body = crate::common::body_json(res).await;
    let items = body.as_array().unwrap();

    let long_urls: Vec<&str> = items
        .iter()
        .map(|item| item["long_url"].as_str().unwrap())
        .collect();
    assert_eq!(
        long_urls,
        vec![
            "https://example.com/first",
            "https://example.com/second",
            "https://example.com/third",
        ]
    );

    let timestamps: Vec<&str> = items
        .iter()
        .map(|item| item["created_at"].as_str().unwrap())
        .collect();
    let mut sorted = timestamps.clone();
    sorted.sort();
    assert_eq!(timestamps, sorted);
}

#[tokio::test]
async fn list_is_capped_at_100_oldest() {
    let app = spawn_app().await;

    sqlx::query(
        "INSERT INTO links (long_url, created_at)
         SELECT 'https://seed.example/' || g,
                now() - make_interval(secs => (200 - g)::double precision)
         FROM generate_series(1, 150) g",
    )
    .execute(&app.pool)
    .await
    .unwrap();

    let res = get(&app, "/api/links").await;
    let body = crate::common::body_json(res).await;
    let items = body.as_array().unwrap();
    assert_eq!(items.len(), 100);

    let long_urls: Vec<&str> = items
        .iter()
        .map(|item| item["long_url"].as_str().unwrap())
        .collect();
    let expected: Vec<String> = (1..=100)
        .map(|n| format!("https://seed.example/{n}"))
        .collect();
    assert_eq!(long_urls, expected);
}

#[tokio::test]
async fn deduped_url_appears_once_in_list() {
    let app = spawn_app().await;

    shorten(&app, "https://example.com/dup").await;
    shorten(&app, "https://example.com/dup").await;
    shorten(&app, "HTTPS://Example.com/dup").await;

    let res = get(&app, "/api/links").await;
    let body = crate::common::body_json(res).await;
    let items = body.as_array().unwrap();
    assert_eq!(items.len(), 1);
}

#[tokio::test]
async fn post_method_not_allowed_on_links() {
    let app = spawn_app().await;

    let res =
        crate::common::post_json(&app, "/api/links", serde_json::json!({}))
            .await;
    assert_eq!(res.status(), StatusCode::METHOD_NOT_ALLOWED);
}
