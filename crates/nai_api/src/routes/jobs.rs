use std::{sync::Arc, time::Duration};

use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use rand::Rng;
use serde::Serialize;
use serde_json::Value;
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use nai_core::{
    config::AppConfig,
    dto::{
        BaseGenerateRequest, CharacterRequest, Img2ImgRequest, InpaintRequest, JobSubmitResponse,
    },
    job::{JobStatus, JobSummary},
    services,
};
use nai_nai::NaiError;

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

#[derive(Serialize)]
struct JobsListResponse {
    items: Vec<JobSummary>,
}

#[derive(Clone, Copy)]
enum JobKind {
    T2i,
    I2i,
    Inpaint,
    Character,
}

impl JobKind {
    fn as_str(self) -> &'static str {
        match self {
            JobKind::T2i => "t2i",
            JobKind::I2i => "i2i",
            JobKind::Inpaint => "inpaint",
            JobKind::Character => "character",
        }
    }
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/jobs", get(jobs_list))
        .route("/api/jobs/{id}", get(job_status))
        .route("/api/jobs/{id}/cancel", post(job_cancel))
        .route("/api/jobs/t2i", post(job_t2i))
        .route("/api/jobs/i2i", post(job_i2i))
        .route("/api/jobs/inpaint", post(job_inpaint))
        .route("/api/jobs/character", post(job_character))
}

async fn jobs_list(State(state): State<Arc<AppState>>) -> ApiResult<JobsListResponse> {
    debug!("jobs_list");
    let items = state.jobs.list(200).await;
    Ok(Json(JobsListResponse { items }))
}

async fn job_status(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> ApiResult<JobStatus> {
    debug!(job_id = %id, "job_status");
    match state.jobs.get_status(id).await {
        Some(s) => Ok(Json(s)),
        None => Err(ApiError::not_found("job not found")),
    }
}

async fn job_cancel(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> ApiResult<serde_json::Value> {
    info!(job_id = %id, "job_cancel request");
    if state.jobs.cancel(id).await {
        Ok(Json(super::error::ok_true()))
    } else {
        Err(ApiError::not_found("job not found"))
    }
}

async fn job_t2i(
    State(state): State<Arc<AppState>>,
    Json(req): Json<BaseGenerateRequest>,
) -> ApiResult<JobSubmitResponse> {
    let mut req = req;
    apply_snippets_to_base(&state, &mut req).await?;
    if let Err(e) = state.last_generation.set_from_base(&req).await {
        warn!(error = %e, "failed to cache last_generation");
    }
    submit_job(
        state,
        JobKind::T2i,
        serde_json::to_value(req).map_err(ApiError::bad_request)?,
    )
    .await
}

async fn job_i2i(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Img2ImgRequest>,
) -> ApiResult<JobSubmitResponse> {
    let mut req = req;
    apply_snippets_to_base(&state, &mut req.base).await?;
    if let Err(e) = state.last_generation.set_from_base(&req.base).await {
        warn!(error = %e, "failed to cache last_generation");
    }
    submit_job(
        state,
        JobKind::I2i,
        serde_json::to_value(req).map_err(ApiError::bad_request)?,
    )
    .await
}

async fn job_inpaint(
    State(state): State<Arc<AppState>>,
    Json(req): Json<InpaintRequest>,
) -> ApiResult<JobSubmitResponse> {
    let mut req = req;
    apply_snippets_to_base(&state, &mut req.base).await?;
    if let Err(e) = state.last_generation.set_from_base(&req.base).await {
        warn!(error = %e, "failed to cache last_generation");
    }
    submit_job(
        state,
        JobKind::Inpaint,
        serde_json::to_value(req).map_err(ApiError::bad_request)?,
    )
    .await
}

async fn job_character(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CharacterRequest>,
) -> ApiResult<JobSubmitResponse> {
    let mut req = req;
    apply_snippets_to_base(&state, &mut req.base).await?;
    if let Err(e) = state.last_generation.set_from_base(&req.base).await {
        warn!(error = %e, "failed to cache last_generation");
    }
    submit_job(
        state,
        JobKind::Character,
        serde_json::to_value(req).map_err(ApiError::bad_request)?,
    )
    .await
}

async fn submit_job(
    state: Arc<AppState>,
    kind: JobKind,
    payload: Value,
) -> ApiResult<JobSubmitResponse> {
    let (id, cancel) = state.jobs.create(kind.as_str()).await;

    let qty = payload
        .get("quantity")
        .and_then(|v| v.as_u64())
        .unwrap_or(1);
    info!(job_id = %id, kind = kind.as_str(), quantity = qty, "job submitted");

    let state2 = state.clone();
    tokio::spawn(async move {
        let queued_at = Instant::now();

        let permit = tokio::select! {
            _ = cancel.cancelled() => {
                info!(job_id = %id, kind = kind.as_str(), "job cancelled while queued");
                state2.jobs.set_status(id, JobStatus::Cancelled).await;
                return;
            }
            p = state2.job_sem.clone().acquire_owned() => {
                match p {
                    Ok(p) => p,
                    Err(e) => {
                        error!(job_id = %id, kind = kind.as_str(), error = %e, "failed to acquire job semaphore");
                        state2.jobs.set_status(id, JobStatus::Failed { error: e.to_string() }).await;
                        return;
                    }
                }
            }
        };

        info!(job_id = %id, kind = kind.as_str(), queued_ms = queued_at.elapsed().as_millis() as u64, "job dequeued");
        state2.jobs.set_status(id, JobStatus::Running).await;

        let result: anyhow::Result<Vec<nai_core::dto::GenerateResponse>> = (async {
            match kind {
                JobKind::T2i => {
                    let req: BaseGenerateRequest = serde_json::from_value(payload.clone())?;
                    let qty = req.quantity.unwrap_or(1).max(1) as usize;
                    let mut outs = Vec::with_capacity(qty);
                    for idx in 0..qty {
                        if cancel.is_cancelled() {
                            info!(job_id = %id, kind = kind.as_str(), done = idx, total = qty, "job cancelled during run");
                            break;
                        }
                        info!(job_id = %id, kind = kind.as_str(), index = idx + 1, total = qty, "generate t2i");
                        let out = with_429_retry(&cancel, id, || {
                            let req2 = req.clone();
                            let st = state2.clone();
                            async move {
                                services::generate_t2i(&st.config, &st.outputs, &st.nai, req2).await
                            }
                        })
                        .await?;
                        outs.push(out);
                        cooldown_sleep(&state2.config, &cancel, id).await;
                    }
                    Ok(outs)
                }
                JobKind::I2i => {
                    let req: Img2ImgRequest = serde_json::from_value(payload.clone())?;
                    let qty = req.base.quantity.unwrap_or(1).max(1) as usize;
                    let mut outs = Vec::with_capacity(qty);
                    for idx in 0..qty {
                        if cancel.is_cancelled() {
                            info!(job_id = %id, kind = kind.as_str(), done = idx, total = qty, "job cancelled during run");
                            break;
                        }
                        info!(job_id = %id, kind = kind.as_str(), index = idx + 1, total = qty, "generate i2i");
                        let out = with_429_retry(&cancel, id, || {
                            let req2 = req.clone();
                            let st = state2.clone();
                            async move {
                                services::generate_i2i(&st.config, &st.outputs, &st.nai, req2).await
                            }
                        })
                        .await?;
                        outs.push(out);
                        cooldown_sleep(&state2.config, &cancel, id).await;
                    }
                    Ok(outs)
                }
                JobKind::Inpaint => {
                    let req: InpaintRequest = serde_json::from_value(payload.clone())?;
                    let qty = req.base.quantity.unwrap_or(1).max(1) as usize;
                    let mut outs = Vec::with_capacity(qty);
                    for idx in 0..qty {
                        if cancel.is_cancelled() {
                            info!(job_id = %id, kind = kind.as_str(), done = idx, total = qty, "job cancelled during run");
                            break;
                        }
                        info!(job_id = %id, kind = kind.as_str(), index = idx + 1, total = qty, "generate inpaint");
                        let out = with_429_retry(&cancel, id, || {
                            let req2 = req.clone();
                            let st = state2.clone();
                            async move {
                                services::generate_inpaint(&st.config, &st.outputs, &st.nai, req2).await
                            }
                        })
                        .await?;
                        outs.push(out);
                        cooldown_sleep(&state2.config, &cancel, id).await;
                    }
                    Ok(outs)
                }
                JobKind::Character => {
                    let req: CharacterRequest = serde_json::from_value(payload.clone())?;
                    let qty = req.base.quantity.unwrap_or(1).max(1) as usize;
                    let mut outs = Vec::with_capacity(qty);
                    for idx in 0..qty {
                        if cancel.is_cancelled() {
                            info!(job_id = %id, kind = kind.as_str(), done = idx, total = qty, "job cancelled during run");
                            break;
                        }
                        info!(job_id = %id, kind = kind.as_str(), index = idx + 1, total = qty, "generate character");
                        let out = with_429_retry(&cancel, id, || {
                            let req2 = req.clone();
                            let st = state2.clone();
                            async move {
                                services::generate_character(&st.config, &st.outputs, &st.nai, req2).await
                            }
                        })
                        .await?;
                        outs.push(out);
                        cooldown_sleep(&state2.config, &cancel, id).await;
                    }
                    Ok(outs)
                }
            }
        })
        .await;

        drop(permit);

        if cancel.is_cancelled() {
            info!(job_id = %id, kind = kind.as_str(), "job cancelled after run");
            state2.jobs.set_status(id, JobStatus::Cancelled).await;
            return;
        }

        match result {
            Ok(outputs) => {
                info!(job_id = %id, kind = kind.as_str(), outputs = outputs.len(), "job succeeded");
                state2
                    .jobs
                    .set_status(id, JobStatus::Succeeded { outputs })
                    .await;
            }
            Err(e) => {
                warn!(job_id = %id, kind = kind.as_str(), error = %e, "job failed");
                state2
                    .jobs
                    .set_status(
                        id,
                        JobStatus::Failed {
                            error: e.to_string(),
                        },
                    )
                    .await;
            }
        }
    });

    Ok(Json(JobSubmitResponse { job_id: id }))
}

async fn cooldown_sleep(cfg: &AppConfig, cancel: &CancellationToken, job_id: Uuid) {
    if cfg.cool_time == 0 {
        return;
    }

    let base = cfg.cool_time as f64;
    let jitter = cfg.cool_jitter.max(0.0);
    let min_s = (base - jitter).abs();
    let max_s = base + jitter;
    let sleep_s = if max_s <= min_s {
        min_s
    } else {
        let mut rng = rand::rng();
        rng.random_range(min_s..=max_s)
    };

    debug!(job_id = %job_id, sleep_s, "cooldown sleep");
    tokio::select! {
        _ = cancel.cancelled() => {},
        _ = tokio::time::sleep(tokio::time::Duration::from_secs_f64(sleep_s)) => {},
    }
}

fn is_http_429(err: &anyhow::Error) -> bool {
    for cause in err.chain() {
        if let Some(ne) = cause.downcast_ref::<NaiError>() {
            if let NaiError::BadStatus { status, .. } = ne {
                if *status == 429 {
                    return true;
                }
            }
        }
    }
    false
}

async fn rate_limit_extra_sleep(cancel: &CancellationToken, job_id: Uuid) {
    let sleep_dur = Duration::from_secs(20);
    warn!(job_id = %job_id, sleep_s = sleep_dur.as_secs(), "rate limited (429), extra sleep before retry");
    tokio::select! {
        _ = cancel.cancelled() => {},
        _ = tokio::time::sleep(sleep_dur) => {},
    }
}

async fn with_429_retry<T, F, Fut>(
    cancel: &CancellationToken,
    job_id: Uuid,
    mut op: F,
) -> anyhow::Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = anyhow::Result<T>>,
{
    match op().await {
        Ok(v) => Ok(v),
        Err(e) => {
            if !is_http_429(&e) {
                return Err(e);
            }
            if cancel.is_cancelled() {
                return Err(e);
            }

            rate_limit_extra_sleep(cancel, job_id).await;
            if cancel.is_cancelled() {
                return Err(e);
            }

            op().await
        }
    }
}
