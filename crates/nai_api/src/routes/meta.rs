use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::get};
use serde_json::json;
use tracing::{debug, error};

use nai_core::nai::NaiApi;

use super::{ApiError, ApiResult, AppState};

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/meta", get(meta))
        .route("/api/anlas", get(anlas))
}

async fn health(State(state): State<Arc<AppState>>) -> ApiResult<serde_json::Value> {
    state.db.health_check().map_err(ApiError::internal)?;
    Ok(Json(json!({ "ok": true })))
}

async fn meta() -> ApiResult<serde_json::Value> {
    Ok(Json(json!({
        "models": [
            "nai-diffusion-4-5-full",
            "nai-diffusion-4-5-curated",
            "nai-diffusion-4-full",
            "nai-diffusion-4-curated-preview",
            "nai-diffusion-3",
            "nai-diffusion-furry-3"
        ],
        "samplers": [
            "k_euler",
            "k_euler_ancestral",
            "k_dpmpp_2s_ancestral",
            "k_dpmpp_2m",
            "k_dpmpp_sde",
            "k_dpmpp_2m_sde",
            "ddim_v3"
        ],
        "noise_schedules": ["native", "karras", "exponential", "polyexponential"],
        "uc_presets": ["Heavy", "Light", "Furry Focus", "Human Focus", "None"]
    })))
}

async fn anlas(State(state): State<Arc<AppState>>) -> ApiResult<serde_json::Value> {
    debug!("anlas");
    match state.nai.inquire_anlas().await {
        Ok(left) => Ok(Json(json!({"anlas": left}))),
        Err(e) => {
            error!(error = %e, "failed to inquire anlas");
            Err(ApiError::internal(e))
        }
    }
}
