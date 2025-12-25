use std::sync::Arc;

use axum::{extract::State, routing::{get, post}, Json, Router};
use tracing::debug;

use nai_core::dto::{OutputsDeleteRequest, OutputsDeleteResponse, OutputsListResponse};

use super::{ApiError, ApiResult, AppState};

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/outputs", get(list_outputs))
        .route("/api/outputs/delete", post(outputs_delete))
}

async fn list_outputs(State(state): State<Arc<AppState>>) -> ApiResult<OutputsListResponse> {
    debug!("outputs_list");
    let items = state
        .outputs
        .list_items(200)
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(OutputsListResponse { items }))
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
