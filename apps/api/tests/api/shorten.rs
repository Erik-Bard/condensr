use axum::body::Body;
use axum::http::{Request, StatusCode};

use crate::common::{body_bytes, post_json, send, shorten, spawn_app};

#[tokio::test]
async fn new_url_returns_created_with_expected_shape() {
    let app = spawn_app().await;

    let (status, body) = shorten(&app, "https://example.com/path").await;

    assert_eq!(status, StatusCode::CREATED);
    let obj = body.as_object().unwrap();
    assert_eq!(
        obj.keys()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>(),
        ["code", "short_url", "long_url"]
            .into_iter()
            .map(String::from)
            .collect()
    );
    let code = body["code"].as_str().unwrap();
    assert!(!code.is_empty());
    assert_eq!(body["short_url"], format!("{}/{}", app.base_url, code));
    assert_eq!(body["long_url"], "https://example.com/path");
}

#[tokio::test]
async fn repeated_url_is_idempotent() {
    let app = spawn_app().await;

    let (first_status, first_body) =
        shorten(&app, "https://example.com/repeat").await;
    assert_eq!(first_status, StatusCode::CREATED);

    let (second_status, second_body) =
        shorten(&app, "https://example.com/repeat").await;
    assert_eq!(second_status, StatusCode::OK);
    assert_eq!(first_body, second_body);
}

#[tokio::test]
async fn normalization_equivalence_classes_dedupe() {
    let app = spawn_app().await;

    let pairs = [
        ("HTTPS://Example.COM/x", "https://example.com/x"),
        ("http://example.com:80/a", "http://example.com/a"),
        ("https://example.com", "https://example.com/"),
    ];

    for (variant, canonical) in pairs {
        let (_, first) = shorten(&app, variant).await;
        let (second_status, second) = shorten(&app, canonical).await;
        assert_eq!(second_status, StatusCode::OK);
        assert_eq!(first["code"], second["code"]);
    }
}

#[tokio::test]
async fn normalization_preserves_query_and_fragment() {
    let app = spawn_app().await;

    let (_, body) = shorten(&app, "https://example.com").await;
    assert_eq!(body["long_url"], "https://example.com/");

    let (_, body) = shorten(&app, "https://example.com/p?q=1#frag").await;
    assert_eq!(body["long_url"], "https://example.com/p?q=1#frag");
}

#[tokio::test]
async fn invalid_urls_return_bad_request() {
    let app = spawn_app().await;

    for invalid in ["not a url", "", "example.com"] {
        let (status, body) = shorten(&app, invalid).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "input: {invalid:?}");
        assert_eq!(body["error"], "invalid_url");
        assert_eq!(body["description"], "not a valid URL");
        assert!(body.as_object().unwrap().get("details").is_none());
    }
}

#[tokio::test]
async fn non_web_schemes_are_rejected() {
    let app = spawn_app().await;

    for scheme_url in [
        "ftp://example.com",
        "javascript:alert(1)",
        "file:///etc/passwd",
    ] {
        let (status, body) = shorten(&app, scheme_url).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "input: {scheme_url:?}");
        assert_eq!(body["error"], "invalid_url");
        assert_eq!(body["description"], "URL scheme must be http or https");
    }
}

#[tokio::test]
async fn missing_content_type_is_unsupported_media_type() {
    let app = spawn_app().await;

    let req = Request::builder()
        .method("POST")
        .uri("/api/shorten")
        .body(Body::from(r#"{"url":"https://example.com"}"#))
        .unwrap();
    let res = send(&app, req).await;
    assert_eq!(res.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
}

#[tokio::test]
async fn malformed_json_body_is_bad_request() {
    let app = spawn_app().await;

    let req = Request::builder()
        .method("POST")
        .uri("/api/shorten")
        .header("content-type", "application/json")
        .body(Body::from("{ not json"))
        .unwrap();
    let res = send(&app, req).await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn empty_json_body_is_bad_request() {
    let app = spawn_app().await;

    let req = Request::builder()
        .method("POST")
        .uri("/api/shorten")
        .header("content-type", "application/json")
        .body(Body::empty())
        .unwrap();
    let res = send(&app, req).await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn missing_url_field_is_unprocessable() {
    let app = spawn_app().await;

    let res = post_json(&app, "/api/shorten", serde_json::json!({})).await;
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn wrong_url_field_type_is_unprocessable() {
    let app = spawn_app().await;

    let res =
        post_json(&app, "/api/shorten", serde_json::json!({ "url": 123 }))
            .await;
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn get_method_not_allowed_on_shorten() {
    let app = spawn_app().await;

    let res = crate::common::get(&app, "/api/shorten").await;
    assert_eq!(res.status(), StatusCode::METHOD_NOT_ALLOWED);
    let _ = body_bytes(res).await;
}
