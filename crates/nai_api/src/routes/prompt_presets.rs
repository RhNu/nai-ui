use std::sync::Arc;

use axum::{extract::{Query, State}, routing::{get, post}, Json, Router};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::PromptPreset;

use super::{ApiError, ApiResult, AppState};

#[derive(Serialize)]
struct PresetNamesResponse {
    names: Vec<String>,
}

#[derive(Serialize)]
struct PromptPresetGetResponse {
    preset: Option<PromptPreset>,
}

#[derive(Deserialize)]
struct PromptPresetPutRequest {
    name: String,
    preset: PromptPreset,
}

#[derive(Deserialize)]
struct SimpleRenameRequest {
    from: String,
    to: String,
}

#[derive(Deserialize)]
struct NameQuery {
    name: String,
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/prompt_presets", get(prompt_presets_list))
        .route(
            "/api/prompt_preset",
            get(prompt_preset_get)
                .put(prompt_preset_put)
                .delete(prompt_preset_delete),
        )
        .route("/api/prompt_preset/rename", post(prompt_preset_rename))
}

async fn prompt_presets_list(State(state): State<Arc<AppState>>) -> ApiResult<PresetNamesResponse> {
    debug!("prompt_presets_list");
    let names = state
        .prompt_presets
        .list_names()
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(PresetNamesResponse { names }))
}

async fn prompt_preset_get(
    State(state): State<Arc<AppState>>,
    Query(q): Query<NameQuery>,
) -> ApiResult<PromptPresetGetResponse> {
    debug!(name = %q.name, "prompt_preset_get");
    let preset = state
        .prompt_presets
        .get(&q.name)
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(PromptPresetGetResponse { preset }))
}

async fn prompt_preset_put(
    State(state): State<Arc<AppState>>,
    Json(req): Json<PromptPresetPutRequest>,
) -> ApiResult<serde_json::Value> {
    debug!(name = %req.name, "prompt_preset_put");
    state
        .prompt_presets
        .upsert(&req.name, &req.preset)
        .await
        .map_err(ApiError::bad_request)?;
    Ok(Json(super::error::ok_true()))
}

async fn prompt_preset_delete(
    State(state): State<Arc<AppState>>,
    Query(q): Query<NameQuery>,
) -> ApiResult<serde_json::Value> {
    debug!(name = %q.name, "prompt_preset_delete");
    state
        .prompt_presets
        .delete(&q.name)
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(super::error::ok_true()))
}

async fn prompt_preset_rename(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SimpleRenameRequest>,
) -> ApiResult<serde_json::Value> {
    debug!(from = %req.from, to = %req.to, "prompt_preset_rename");
    state
        .prompt_presets
        .rename(&req.from, &req.to)
        .await
        .map_err(ApiError::bad_request)?;
    Ok(Json(super::error::ok_true()))
}
