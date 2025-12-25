use std::sync::Arc;

use axum::{extract::State, routing::get, Json, Router};
use serde::Serialize;

use nai_core::dto::BaseGenerateRequest;

use crate::LastGenerationRecord;

use super::{ApiError, ApiResult, AppState};

#[derive(Serialize)]
struct LastGenerationGetResponse {
    record: Option<LastGenerationRecord>,
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route(
        "/api/last_generation",
        get(last_generation_get)
            .put(last_generation_put)
            .delete(last_generation_delete),
    )
}

async fn last_generation_get(State(state): State<Arc<AppState>>) -> ApiResult<LastGenerationGetResponse> {
    let record = state
        .last_generation
        .get()
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(LastGenerationGetResponse { record }))
}

async fn last_generation_put(
    State(state): State<Arc<AppState>>,
    Json(req): Json<BaseGenerateRequest>,
) -> ApiResult<serde_json::Value> {
    let record = LastGenerationRecord {
        updated_at_ms: crate::last_generation::now_ms(),
        base: req,
    };

    state
        .last_generation
        .set(record)
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(super::error::ok_true()))
}

async fn last_generation_delete(State(state): State<Arc<AppState>>) -> ApiResult<serde_json::Value> {
    state
        .last_generation
        .clear()
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(super::error::ok_true()))
}
