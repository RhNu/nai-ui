use std::sync::Arc;

use axum::{extract::{Query, State}, routing::{get, post}, Json, Router};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::CharacterSlotPreset;

use super::{ApiError, ApiResult, AppState};

#[derive(Serialize)]
struct PresetNamesResponse {
    names: Vec<String>,
}

#[derive(Serialize)]
struct CharacterPresetGetResponse {
    preset: Option<CharacterSlotPreset>,
}

#[derive(Deserialize)]
struct CharacterPresetPutRequest {
    name: String,
    preset: CharacterSlotPreset,
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
        .route("/api/character_presets", get(character_presets_list))
        .route(
            "/api/character_preset",
            get(character_preset_get)
                .put(character_preset_put)
                .delete(character_preset_delete),
        )
        .route("/api/character_preset/rename", post(character_preset_rename))
}

async fn character_presets_list(State(state): State<Arc<AppState>>) -> ApiResult<PresetNamesResponse> {
    debug!("character_presets_list");
    let names = state
        .character_presets
        .list_names()
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(PresetNamesResponse { names }))
}

async fn character_preset_get(
    State(state): State<Arc<AppState>>,
    Query(q): Query<NameQuery>,
) -> ApiResult<CharacterPresetGetResponse> {
    debug!(name = %q.name, "character_preset_get");
    let preset = state
        .character_presets
        .get(&q.name)
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(CharacterPresetGetResponse { preset }))
}

async fn character_preset_put(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CharacterPresetPutRequest>,
) -> ApiResult<serde_json::Value> {
    debug!(name = %req.name, "character_preset_put");
    state
        .character_presets
        .upsert(&req.name, &req.preset)
        .await
        .map_err(ApiError::bad_request)?;
    Ok(Json(super::error::ok_true()))
}

async fn character_preset_delete(
    State(state): State<Arc<AppState>>,
    Query(q): Query<NameQuery>,
) -> ApiResult<serde_json::Value> {
    debug!(name = %q.name, "character_preset_delete");
    state
        .character_presets
        .delete(&q.name)
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(super::error::ok_true()))
}

async fn character_preset_rename(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SimpleRenameRequest>,
) -> ApiResult<serde_json::Value> {
    debug!(from = %req.from, to = %req.to, "character_preset_rename");
    state
        .character_presets
        .rename(&req.from, &req.to)
        .await
        .map_err(ApiError::bad_request)?;
    Ok(Json(super::error::ok_true()))
}
