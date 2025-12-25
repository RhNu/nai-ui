use std::sync::Arc;

use anyhow::anyhow;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

use crate::{DEFAULT_PRESET_NAME, GeneratePreset};

use super::{ApiError, ApiResult, AppState};

#[derive(Serialize)]
struct PresetsListResponse {
    names: Vec<String>,
}

#[derive(Deserialize)]
struct PresetGetQuery {
    model: String,
    name: String,
}

#[derive(Serialize)]
struct PresetGetResponse {
    preset: Option<GeneratePreset>,
}

#[derive(Deserialize)]
struct PresetPutRequest {
    model: String,
    name: String,
    preset: GeneratePreset,
}

#[derive(Deserialize)]
struct PresetRenameRequest {
    model: String,
    from: String,
    to: String,
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/presets/{model}", get(presets_list))
        .route(
            "/api/preset",
            get(preset_get).put(preset_put).delete(preset_delete),
        )
        .route("/api/preset/rename", post(preset_rename))
}

async fn presets_list(
    State(state): State<Arc<AppState>>,
    Path(model): Path<String>,
) -> ApiResult<PresetsListResponse> {
    let names = state
        .presets
        .list_names(&model)
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(PresetsListResponse { names }))
}

async fn preset_get(
    State(state): State<Arc<AppState>>,
    Query(q): Query<PresetGetQuery>,
) -> ApiResult<PresetGetResponse> {
    let preset = state
        .presets
        .get(&q.model, &q.name)
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(PresetGetResponse { preset }))
}

async fn preset_put(
    State(state): State<Arc<AppState>>,
    Json(req): Json<PresetPutRequest>,
) -> ApiResult<serde_json::Value> {
    state
        .presets
        .upsert(&req.model, &req.name, &req.preset)
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(super::error::ok_true()))
}

async fn preset_delete(
    State(state): State<Arc<AppState>>,
    Query(q): Query<PresetGetQuery>,
) -> ApiResult<serde_json::Value> {
    if q.name.trim() == DEFAULT_PRESET_NAME {
        return Err(ApiError::bad_request(anyhow!(
            "cannot delete default preset"
        )));
    }

    state
        .presets
        .delete(&q.model, &q.name)
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(super::error::ok_true()))
}

async fn preset_rename(
    State(state): State<Arc<AppState>>,
    Json(req): Json<PresetRenameRequest>,
) -> ApiResult<serde_json::Value> {
    if req.from.trim() == DEFAULT_PRESET_NAME {
        return Err(ApiError::bad_request(anyhow!(
            "cannot rename default preset"
        )));
    }

    let to = req.to.trim();
    if to.is_empty() {
        return Err(ApiError::bad_request(anyhow!("empty preset name")));
    }

    state
        .presets
        .rename(&req.model, &req.from, to)
        .await
        .map_err(ApiError::bad_request)?;
    Ok(Json(super::error::ok_true()))
}
