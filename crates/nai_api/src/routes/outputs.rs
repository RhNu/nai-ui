use std::sync::Arc;

use axum::{extract::State, routing::{get, post}, Json, Router};
use serde::Deserialize;
use tracing::debug;

use nai_core::dto::{OutputsDeleteRequest, OutputsDeleteResponse, OutputsListResponse};

use super::{ApiError, ApiResult, AppState};

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/outputs", get(list_outputs))
        .route("/api/outputs/delete", post(outputs_delete))
}

#[derive(Deserialize)]
struct OutputsListQuery {
    limit: Option<usize>,
    offset: Option<usize>,
}

async fn list_outputs(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(query): axum::extract::Query<OutputsListQuery>,
) -> ApiResult<OutputsListResponse> {
    debug!("outputs_list");
    let limit = query.limit.unwrap_or(60).clamp(1, 200);
    let offset = query.offset.unwrap_or(0);

    let (items, has_more, next_offset) = state
        .outputs
        .list_items_paginated(limit, offset)
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(OutputsListResponse {
        items,
        next_offset,
        has_more,
    }))
}

async fn outputs_delete(
    State(state): State<Arc<AppState>>,
    Json(req): Json<OutputsDeleteRequest>,
) -> ApiResult<OutputsDeleteResponse> {
    debug!(count = req.items.len(), "outputs_delete");
    let deleted = state
        .outputs
        .delete_rel_files(&req.items)
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(OutputsDeleteResponse { deleted }))
}
