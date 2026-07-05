use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    routing::get,
};
use condensr_core::decode;
use serde::Serialize;

use crate::{AppState, models::error::AppError};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/links", get(list_links))
        .route("/{code}", get(redirect))
}

#[derive(Debug, Serialize)]
pub struct ListLinksResponse {
    pub id: u64,
    pub long_url: String,
    pub created_at: String,
    pub code: String,
}

async fn list_links(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let rows = sqlx::query!(
        "SELECT id, long_url, created_at FROM links ORDER BY created_at ASC LIMIT 100"
    )
    .fetch_all(&state.db)
    .await?;

    let links = rows
        .into_iter()
        .map(|row| {
            let id: u64 = row.id.try_into()?;
            Ok(ListLinksResponse {
                id,
                code: condensr_core::encode(id),
                long_url: row.long_url,
                created_at: row.created_at.to_rfc3339(),
            })
        })
        .collect::<Result<Vec<_>, AppError>>()?;

    Ok(Json(links))
}

async fn redirect(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<Response, AppError> {
    let Some(id) = decode(&code).ok().and_then(|id| i64::try_from(id).ok())
    else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    let row = sqlx::query!("SELECT long_url FROM links WHERE id = $1", id)
        .fetch_optional(&state.db)
        .await?;

    Ok(match row {
        Some(link) => Redirect::temporary(&link.long_url).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    })
}
