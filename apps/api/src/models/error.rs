use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ApiError<'a> {
    pub error: &'a str,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

pub struct AppError {
    status: StatusCode,
    error: &'static str,
    description: String,
}

impl AppError {
    pub fn bad_request(
        error: &'static str,
        description: impl Into<String>,
    ) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            error,
            description: description.into(),
        }
    }

    pub fn too_many_requests() -> Self {
        Self {
            status: StatusCode::TOO_MANY_REQUESTS,
            error: "rate_limit_exceeded",
            description: "too many shortening requests".to_string(),
        }
    }

    pub fn request_timeout() -> Self {
        Self {
            status: StatusCode::REQUEST_TIMEOUT,
            error: "request_timeout",
            description: "request processing exceeded the configured timeout"
                .to_string(),
        }
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        let error = err.into();
        tracing::error!(error = ?error, "request failed with an internal error");
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: "internal_server_error",
            description: "an internal server error occurred".to_string(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = ApiError {
            error: self.error,
            description: self.description,
            details: None,
        };
        (self.status, Json(body)).into_response()
    }
}
