use std::{net::SocketAddr, num::NonZeroU32, time::Duration};

use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{Request, StatusCode},
};
use condensr_api::config::{CorsConfig, HttpConfig, RateLimitConfig};

use crate::common::{
    body_json, get, send, shorten, spawn_app, spawn_app_with_http_config,
};

fn rate_limited_http(trusted_proxy_cidrs: &[&str]) -> HttpConfig {
    HttpConfig {
        shorten_rate_limit: Some(RateLimitConfig {
            requests: NonZeroU32::new(1).unwrap(),
            window: Duration::from_secs(60),
            burst: NonZeroU32::new(1).unwrap(),
            trusted_proxy_cidrs: trusted_proxy_cidrs
                .iter()
                .map(|cidr| cidr.parse().unwrap())
                .collect(),
        }),
        ..HttpConfig::default()
    }
}

async fn post_from(
    app: &crate::common::TestApp,
    peer: &str,
    forwarded_for: Option<&str>,
    url: &str,
) -> axum::response::Response {
    let body = serde_json::to_vec(&serde_json::json!({ "url": url })).unwrap();
    let mut builder = Request::builder()
        .method("POST")
        .uri("/api/shorten")
        .header("content-type", "application/json");
    if let Some(forwarded_for) = forwarded_for {
        builder = builder.header("x-forwarded-for", forwarded_for);
    }
    let mut request = builder.body(Body::from(body)).unwrap();
    request
        .extensions_mut()
        .insert(ConnectInfo(peer.parse::<SocketAddr>().unwrap()));
    send(app, request).await
}

#[tokio::test]
async fn cors_is_absent_by_default() {
    let app = spawn_app().await;
    let request = Request::builder()
        .method("OPTIONS")
        .uri("/api/shorten")
        .header("origin", "https://app.example")
        .header("access-control-request-method", "POST")
        .body(Body::empty())
        .unwrap();
    let response = send(&app, request).await;
    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    assert!(
        response
            .headers()
            .get("access-control-allow-origin")
            .is_none()
    );
}

#[tokio::test]
async fn exact_origin_cors_handles_preflight_only_on_api_routes() {
    let app = spawn_app_with_http_config(HttpConfig {
        cors: Some(CorsConfig::Origins(vec![
            "https://app.example".to_string(),
        ])),
        ..HttpConfig::default()
    })
    .await;

    let request = Request::builder()
        .method("OPTIONS")
        .uri("/api/shorten")
        .header("origin", "https://app.example")
        .header("access-control-request-method", "POST")
        .header("access-control-request-headers", "content-type")
        .body(Body::empty())
        .unwrap();
    let response = send(&app, request).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get("access-control-allow-origin")
            .unwrap(),
        "https://app.example"
    );
    assert!(
        response
            .headers()
            .get("access-control-allow-credentials")
            .is_none()
    );

    let request = Request::builder()
        .method("OPTIONS")
        .uri("/api/shorten")
        .header("origin", "https://other.example")
        .header("access-control-request-method", "POST")
        .body(Body::empty())
        .unwrap();
    let response = send(&app, request).await;
    assert!(
        response
            .headers()
            .get("access-control-allow-origin")
            .is_none()
    );

    let request = Request::builder()
        .method("GET")
        .uri("/health")
        .header("origin", "https://app.example")
        .body(Body::empty())
        .unwrap();
    let response = send(&app, request).await;
    assert!(
        response
            .headers()
            .get("access-control-allow-origin")
            .is_none()
    );
}

#[tokio::test]
async fn wildcard_cors_is_explicit_and_has_no_credentials() {
    let app = spawn_app_with_http_config(HttpConfig {
        cors: Some(CorsConfig::Any),
        ..HttpConfig::default()
    })
    .await;
    let request = Request::builder()
        .method("GET")
        .uri("/api/links")
        .header("origin", "https://any.example")
        .body(Body::empty())
        .unwrap();
    let response = send(&app, request).await;
    assert_eq!(
        response
            .headers()
            .get("access-control-allow-origin")
            .unwrap(),
        "*"
    );
    assert!(
        response
            .headers()
            .get("access-control-allow-credentials")
            .is_none()
    );
}

#[tokio::test]
async fn shortening_limit_is_per_client_and_returns_retry_metadata() {
    let app = spawn_app_with_http_config(rate_limited_http(&[])).await;

    let first =
        post_from(&app, "192.0.2.1:1000", None, "https://example.com/first")
            .await;
    assert_eq!(first.status(), StatusCode::CREATED);

    let limited =
        post_from(&app, "192.0.2.1:2000", None, "https://example.com/second")
            .await;
    assert_eq!(limited.status(), StatusCode::TOO_MANY_REQUESTS);
    assert!(limited.headers().get("retry-after").is_some());
    let body = body_json(limited).await;
    assert_eq!(body["error"], "rate_limit_exceeded");

    let other_client =
        post_from(&app, "192.0.2.2:1000", None, "https://example.com/second")
            .await;
    assert_eq!(other_client.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn shortening_limit_honors_the_burst_allowance() {
    let app = spawn_app_with_http_config(HttpConfig {
        shorten_rate_limit: Some(RateLimitConfig {
            requests: NonZeroU32::new(60).unwrap(),
            window: Duration::from_secs(60),
            burst: NonZeroU32::new(2).unwrap(),
            trusted_proxy_cidrs: Vec::new(),
        }),
        ..HttpConfig::default()
    })
    .await;

    for suffix in ["first", "second"] {
        assert_eq!(
            post_from(
                &app,
                "192.0.2.1:1000",
                None,
                &format!("https://example.com/{suffix}"),
            )
            .await
            .status(),
            StatusCode::CREATED
        );
    }
    assert_eq!(
        post_from(&app, "192.0.2.1:1000", None, "https://example.com/third",)
            .await
            .status(),
        StatusCode::TOO_MANY_REQUESTS
    );
}

#[tokio::test]
async fn forwarded_client_ip_requires_a_trusted_peer() {
    let untrusted =
        spawn_app_with_http_config(rate_limited_http(&["10.0.0.0/8"])).await;
    let first = post_from(
        &untrusted,
        "203.0.113.10:1000",
        Some("192.0.2.1"),
        "https://example.com/a",
    )
    .await;
    assert_eq!(first.status(), StatusCode::CREATED);
    let second = post_from(
        &untrusted,
        "203.0.113.10:1000",
        Some("192.0.2.2"),
        "https://example.com/b",
    )
    .await;
    assert_eq!(second.status(), StatusCode::TOO_MANY_REQUESTS);

    let trusted =
        spawn_app_with_http_config(rate_limited_http(&["10.0.0.0/8"])).await;
    let first = post_from(
        &trusted,
        "10.0.0.10:1000",
        Some("192.0.2.1"),
        "https://example.com/a",
    )
    .await;
    assert_eq!(first.status(), StatusCode::CREATED);
    let second = post_from(
        &trusted,
        "10.0.0.10:1000",
        Some("192.0.2.2"),
        "https://example.com/b",
    )
    .await;
    assert_eq!(second.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn shortening_limit_does_not_cover_other_routes() {
    let app = spawn_app_with_http_config(rate_limited_http(&[])).await;
    let first =
        post_from(&app, "192.0.2.1:1000", None, "https://example.com/target")
            .await;
    let body = body_json(first).await;
    let code = body["code"].as_str().unwrap();
    let limited =
        post_from(&app, "192.0.2.1:1000", None, "https://example.com/other")
            .await;
    assert_eq!(limited.status(), StatusCode::TOO_MANY_REQUESTS);

    assert_eq!(get(&app, "/health").await.status(), StatusCode::OK);
    assert_eq!(get(&app, "/api/links").await.status(), StatusCode::OK);
    assert_eq!(
        get(&app, &format!("/{code}")).await.status(),
        StatusCode::TEMPORARY_REDIRECT
    );
}

#[tokio::test]
async fn shortening_body_has_a_small_default_limit() {
    let app = spawn_app().await;
    let request = Request::builder()
        .method("POST")
        .uri("/api/shorten")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_vec(&serde_json::json!({
                "url": format!("https://example.com/{}", "a".repeat(17 * 1024))
            }))
            .unwrap(),
        ))
        .unwrap();
    let response = send(&app, request).await;
    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
}

#[tokio::test]
async fn throttling_is_disabled_by_default() {
    let app = spawn_app().await;
    for index in 0..12 {
        let (status, _) =
            shorten(&app, &format!("https://example.com/default-{index}"))
                .await;
        assert_eq!(status, StatusCode::CREATED);
    }
}
