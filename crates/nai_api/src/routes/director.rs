use std::sync::Arc;

use axum::{extract::State, routing::post, Json, Router};
use serde_json::json;
use tracing::info;

use nai_core::dto::{DirectorPromptRequest, DirectorRequest, DirectorResponse};
use nai_core::services;

use super::{ApiError, ApiResult, AppState};

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/director/remove_bg", post(director_remove_bg))
        .route("/api/director/line_art", post(director_line_art))
        .route("/api/director/sketch", post(director_sketch))
        .route("/api/director/colorize", post(director_colorize))
        .route("/api/director/emotion", post(director_emotion))
        .route("/api/director/declutter", post(director_declutter))
}

async fn director_remove_bg(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DirectorRequest>,
) -> ApiResult<DirectorResponse> {
    info!(width = req.width, height = req.height, "director remove_bg");
    let payload = json!({
        "req_type": "bg-removal",
        "use_new_shared_trial": true,
        "width": req.width,
        "height": req.height,
        "image": req.image_base64,
    });
    let resp = services::director_call(&state.outputs, &state.nai, payload, true)
        .await
        .map_err(ApiError::bad_request)?;
    Ok(Json(resp))
}

async fn director_line_art(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DirectorRequest>,
) -> ApiResult<DirectorResponse> {
    director_simple(state, req, "lineart").await
}

async fn director_sketch(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DirectorRequest>,
) -> ApiResult<DirectorResponse> {
    director_simple(state, req, "sketch").await
}

async fn director_declutter(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DirectorRequest>,
) -> ApiResult<DirectorResponse> {
    director_simple(state, req, "declutter").await
}

async fn director_colorize(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DirectorPromptRequest>,
) -> ApiResult<DirectorResponse> {
    let payload = json!({
        "req_type": "colorize",
        "use_new_shared_trial": true,
        "prompt": req.prompt,
        "defry": req.defry,
        "width": req.base.width,
        "height": req.base.height,
        "image": req.base.image_base64,
    });
    let resp = services::director_call(&state.outputs, &state.nai, payload, false)
        .await
        .map_err(ApiError::bad_request)?;
    Ok(Json(resp))
}

async fn director_emotion(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DirectorPromptRequest>,
) -> ApiResult<DirectorResponse> {
    let payload = json!({
        "req_type": "emotion",
        "use_new_shared_trial": true,
        "prompt": req.prompt,
        "defry": req.defry,
        "width": req.base.width,
        "height": req.base.height,
        "image": req.base.image_base64,
    });
    let resp = services::director_call(&state.outputs, &state.nai, payload, false)
        .await
        .map_err(ApiError::bad_request)?;
    Ok(Json(resp))
}

async fn director_simple(
    state: Arc<AppState>,
    req: DirectorRequest,
    kind: &'static str,
) -> ApiResult<DirectorResponse> {
    let payload = json!({
        "req_type": kind,
        "use_new_shared_trial": true,
        "width": req.width,
        "height": req.height,
        "image": req.image_base64,
    });
    let resp = services::director_call(&state.outputs, &state.nai, payload, false)
        .await
        .map_err(ApiError::bad_request)?;
    Ok(Json(resp))
}
