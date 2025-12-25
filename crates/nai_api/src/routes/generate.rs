use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::post};
use tracing::{info, warn};

use nai_core::dto::{
    BaseGenerateRequest, CharacterRequest, GenerateResponse, Img2ImgRequest, InpaintRequest,
};
use nai_core::services;

use super::{ApiError, ApiResult, AppState};

async fn apply_snippets_to_base(
    state: &AppState,
    base: &mut BaseGenerateRequest,
) -> Result<(), ApiError> {
    let mut warnings = Vec::new();

    let expanded = crate::expand_prompts_pair(
        &state.config,
        &state.prompt_snippets,
        &base.positive,
        &base.negative,
    )
    .await
    .map_err(ApiError::bad_request)?;
    warnings.extend(expanded.warnings.iter().cloned());
    base.positive = expanded.positive;
    base.negative = expanded.negative;

    if let Some(chars) = base.character_prompts.as_mut() {
        for cp in chars.iter_mut() {
            if cp.prompt.trim().is_empty() && cp.uc.trim().is_empty() {
                continue;
            }
            let expanded_char = crate::expand_prompts_pair(
                &state.config,
                &state.prompt_snippets,
                &cp.prompt,
                &cp.uc,
            )
            .await
            .map_err(ApiError::bad_request)?;
            warnings.extend(expanded_char.warnings.iter().cloned());
            cp.prompt = expanded_char.positive;
            cp.uc = expanded_char.negative;
        }
    }

    for w in warnings {
        warn!(warning = %w, "prompt snippet warning");
    }
    Ok(())
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/generate/t2i", post(t2i))
        .route("/api/generate/i2i", post(i2i))
        .route("/api/generate/inpaint", post(inpaint))
        .route("/api/generate/character", post(character))
}

async fn t2i(
    State(state): State<Arc<AppState>>,
    Json(req): Json<BaseGenerateRequest>,
) -> ApiResult<GenerateResponse> {
    let mut req = req;
    apply_snippets_to_base(&state, &mut req).await?;
    info!(
        model = %req.model,
        width = req.width,
        height = req.height,
        steps = req.steps,
        sampler = %req.sampler,
        "generate t2i"
    );

    if let Err(e) = state.last_generation.set_from_base(&req).await {
        warn!(error = %e, "failed to cache last_generation");
    }

    let resp = services::generate_t2i(&state.config, &state.outputs, &state.nai, req)
        .await
        .map_err(ApiError::bad_request)?;
    Ok(Json(resp))
}

async fn i2i(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Img2ImgRequest>,
) -> ApiResult<GenerateResponse> {
    let mut req = req;
    apply_snippets_to_base(&state, &mut req.base).await?;
    info!(
        model = %req.base.model,
        width = req.base.width,
        height = req.base.height,
        steps = req.base.steps,
        sampler = %req.base.sampler,
        "generate i2i"
    );

    if let Err(e) = state.last_generation.set_from_base(&req.base).await {
        warn!(error = %e, "failed to cache last_generation");
    }

    let resp = services::generate_i2i(&state.config, &state.outputs, &state.nai, req)
        .await
        .map_err(ApiError::bad_request)?;
    Ok(Json(resp))
}

async fn inpaint(
    State(state): State<Arc<AppState>>,
    Json(req): Json<InpaintRequest>,
) -> ApiResult<GenerateResponse> {
    let mut req = req;
    apply_snippets_to_base(&state, &mut req.base).await?;
    info!(
        model = %req.base.model,
        width = req.base.width,
        height = req.base.height,
        steps = req.base.steps,
        sampler = %req.base.sampler,
        "generate inpaint"
    );

    if let Err(e) = state.last_generation.set_from_base(&req.base).await {
        warn!(error = %e, "failed to cache last_generation");
    }

    let resp = services::generate_inpaint(&state.config, &state.outputs, &state.nai, req)
        .await
        .map_err(ApiError::bad_request)?;
    Ok(Json(resp))
}

async fn character(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CharacterRequest>,
) -> ApiResult<GenerateResponse> {
    let mut req = req;
    apply_snippets_to_base(&state, &mut req.base).await?;
    info!(
        model = %req.base.model,
        width = req.base.width,
        height = req.base.height,
        steps = req.base.steps,
        sampler = %req.base.sampler,
        style_aware = req.style_aware,
        "generate character"
    );

    if let Err(e) = state.last_generation.set_from_base(&req.base).await {
        warn!(error = %e, "failed to cache last_generation");
    }

    let resp = services::generate_character(&state.config, &state.outputs, &state.nai, req)
        .await
        .map_err(ApiError::bad_request)?;
    Ok(Json(resp))
}
