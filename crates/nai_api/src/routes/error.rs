use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use serde_json::json;

pub type ApiResult<T> = Result<Json<T>, ApiError>;

#[derive(Debug)]
pub enum ApiError {
    BadRequest(anyhow::Error),
    NotFound(String),
    Internal(anyhow::Error),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl ApiError {
    pub fn bad_request<E: Into<anyhow::Error>>(err: E) -> Self {
        Self::BadRequest(err.into())
    }

    pub fn internal<E: Into<anyhow::Error>>(err: E) -> Self {
        Self::Internal(err.into())
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, msg) = match self {
            ApiError::BadRequest(err) => (StatusCode::BAD_REQUEST, err.to_string()),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::Internal(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
        };

        (status, Json(ErrorBody { error: msg })).into_response()
    }
}

pub fn ok_true() -> serde_json::Value {
    json!({ "ok": true })
}
