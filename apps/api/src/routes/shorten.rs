use axum::{
    Json, Router, extract::State, http::StatusCode, response::IntoResponse,
    routing::post,
};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{AppState, models::error::AppError};

pub fn router() -> Router<AppState> {
    Router::new().route("/api/shorten", post(shorten))
}

#[derive(Debug, Deserialize)]
pub struct ShortenRequest {
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct ShortenResponse {
    pub code: String,
    pub short_url: String,
    pub long_url: String,
}

fn normalize_url(raw: &str) -> Result<String, &'static str> {
    let parsed = Url::parse(raw).map_err(|_| "not a valid URL")?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err("URL scheme must be http or https");
    }
    Ok(parsed.into())
}

async fn shorten(
    State(state): State<AppState>,
    Json(req): Json<ShortenRequest>,
) -> Result<impl IntoResponse, AppError> {
    let long_url = normalize_url(&req.url)
        .map_err(|reason| AppError::bad_request("invalid_url", reason))?;

    let row = sqlx::query!(
        "INSERT INTO links (long_url) VALUES ($1)
         ON CONFLICT (long_url) DO UPDATE SET long_url = EXCLUDED.long_url
         RETURNING id, (xmax = 0) AS inserted",
        long_url
    )
    .fetch_one(&state.db)
    .await?;

    let code = condensr_core::encode(row.id.try_into()?);
    let status = if row.inserted.unwrap_or(false) {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    };

    let res = ShortenResponse {
        short_url: format!("{}/{}", state.base_url, code),
        long_url,
        code,
    };

    Ok((status, Json(res)).into_response())
}

#[cfg(test)]
mod tests {
    use super::normalize_url;

    #[test]
    fn accepts_http_and_https() {
        assert!(normalize_url("http://example.com").is_ok());
        assert!(
            normalize_url("https://example.com/very/long/path?q=1#frag")
                .is_ok()
        );
    }

    #[test]
    fn rejects_non_urls() {
        assert!(normalize_url("not a url").is_err());
        assert!(normalize_url("").is_err());
        assert!(normalize_url("example.com").is_err());
        assert!(normalize_url("https://").is_err());
    }

    #[test]
    fn rejects_non_web_schemes() {
        assert!(normalize_url("javascript:alert(1)").is_err());
        assert!(normalize_url("file:///etc/passwd").is_err());
        assert!(normalize_url("ftp://example.com").is_err());
    }

    #[test]
    fn canonicalizes_for_dedupe() {
        assert_eq!(
            normalize_url("https://Example.com").unwrap(),
            "https://example.com/"
        );
        assert_eq!(
            normalize_url("https://example.com").unwrap(),
            normalize_url("https://example.com/").unwrap()
        );
        assert_eq!(
            normalize_url("http://example.com:80/a").unwrap(),
            "http://example.com/a"
        );
    }
}
