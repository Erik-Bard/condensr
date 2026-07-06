use axum::{
    Json, Router, http::StatusCode, response::IntoResponse, routing::get,
};

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/health", get(health))
}

async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({ "status": "ok" })))
}
