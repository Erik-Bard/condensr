use axum::http::StatusCode;

use crate::common::{body_bytes, get, shorten, spawn_app};

#[tokio::test]
async fn redirects_to_long_url() {
    let app = spawn_app().await;

    let (_, body) = shorten(&app, "https://example.com/target").await;
    let code = body["code"].as_str().unwrap();
    let long_url = body["long_url"].as_str().unwrap().to_string();

    let res = get(&app, &format!("/{code}")).await;
    assert_eq!(res.status(), StatusCode::TEMPORARY_REDIRECT);
    let location = res
        .headers()
        .get("location")
        .and_then(|v| v.to_str().ok())
        .unwrap()
        .to_string();
    assert_eq!(location, long_url);
    assert!(body_bytes(res).await.is_empty());
}

#[tokio::test]
async fn unknown_decodable_code_is_not_found() {
    let app = spawn_app().await;

    let code = condensr_core::encode(999_999_999);
    let res = get(&app, &format!("/{code}")).await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
    assert!(body_bytes(res).await.is_empty());
}

#[tokio::test]
async fn out_of_range_code_is_not_found() {
    let app = spawn_app().await;

    let code = condensr_core::encode(u64::MAX);
    let res = get(&app, &format!("/{code}")).await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn undecodable_code_is_not_found() {
    let app = spawn_app().await;

    for code in ["abc_def", "-"] {
        let res = get(&app, &format!("/{code}")).await;
        assert_eq!(res.status(), StatusCode::NOT_FOUND, "code: {code:?}");
    }
}

#[tokio::test]
async fn static_routes_take_precedence_over_code_route() {
    let app = spawn_app().await;

    let res = get(&app, "/health").await;
    assert_eq!(res.status(), StatusCode::OK);

    let res = get(&app, "/api/links").await;
    assert_eq!(res.status(), StatusCode::OK);
}
