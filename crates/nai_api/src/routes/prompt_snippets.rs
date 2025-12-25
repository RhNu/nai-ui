use std::sync::Arc;

use axum::{
    Json, Router,
    extract::Query,
    extract::State,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::{
    expand_prompts_pair,
    prompt_snippet_store::{PromptSnippet, PromptSnippetSummary},
};

use super::{ApiError, ApiResult, AppState};

#[derive(Serialize)]
struct SnippetsListResponse {
    items: Vec<PromptSnippetSummary>,
}

#[derive(Serialize)]
struct SnippetGetResponse {
    snippet: Option<PromptSnippet>,
}

#[derive(Deserialize)]
struct SnippetPutRequest {
    name: String,
    snippet: PromptSnippet,
}

#[derive(Deserialize)]
struct SnippetRenameRequest {
    from: String,
    to: String,
}

#[derive(Debug, Deserialize)]
struct SnippetListQuery {
    q: Option<String>,
    tags: Option<String>,
}

#[derive(Deserialize)]
struct NameQuery {
    name: String,
}

#[derive(Deserialize)]
struct SnippetPreviewRequest {
    positive: String,
    negative: String,
}

#[derive(Serialize)]
struct SnippetPreviewResponse {
    positive: String,
    negative: String,
    warnings: Vec<String>,
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/prompt_snippets", get(prompt_snippets_list))
        .route(
            "/api/prompt_snippet",
            get(prompt_snippet_get)
                .put(prompt_snippet_put)
                .delete(prompt_snippet_delete),
        )
        .route("/api/prompt_snippet/rename", post(prompt_snippet_rename))
        .route("/api/prompt_snippet/preview", post(prompt_snippet_preview))
}

fn parse_tags(tags: Option<String>) -> Vec<String> {
    tags.map(|s| {
        s.split(',')
            .map(|t| t.trim())
            .filter(|t| !t.is_empty())
            .map(|t| t.to_string())
            .collect::<Vec<_>>()
    })
    .unwrap_or_default()
}

async fn prompt_snippets_list(
    State(state): State<Arc<AppState>>,
    Query(q): Query<SnippetListQuery>,
) -> ApiResult<SnippetsListResponse> {
    debug!(?q, "prompt_snippets_list");
    let tags = parse_tags(q.tags);
    let items = state
        .prompt_snippets
        .list(q.q.as_deref(), &tags)
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(SnippetsListResponse { items }))
}

async fn prompt_snippet_get(
    State(state): State<Arc<AppState>>,
    Query(q): Query<NameQuery>,
) -> ApiResult<SnippetGetResponse> {
    debug!(name = %q.name, "prompt_snippet_get");
    let snippet = state
        .prompt_snippets
        .get(&q.name)
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(SnippetGetResponse { snippet }))
}

async fn prompt_snippet_put(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SnippetPutRequest>,
) -> ApiResult<serde_json::Value> {
    debug!(name = %req.name, "prompt_snippet_put");
    state
        .prompt_snippets
        .upsert(&req.name, req.snippet)
        .await
        .map_err(ApiError::bad_request)?;
    Ok(Json(super::error::ok_true()))
}

async fn prompt_snippet_delete(
    State(state): State<Arc<AppState>>,
    Query(q): Query<NameQuery>,
) -> ApiResult<serde_json::Value> {
    debug!(name = %q.name, "prompt_snippet_delete");
    state
        .prompt_snippets
        .delete(&q.name)
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(super::error::ok_true()))
}

async fn prompt_snippet_rename(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SnippetRenameRequest>,
) -> ApiResult<serde_json::Value> {
    debug!(from = %req.from, to = %req.to, "prompt_snippet_rename");
    if req.to.trim().is_empty() {
        return Err(ApiError::bad_request(anyhow::anyhow!("empty snippet name")));
    }
    state
        .prompt_snippets
        .rename(&req.from, &req.to)
        .await
        .map_err(ApiError::bad_request)?;
    Ok(Json(super::error::ok_true()))
}

async fn prompt_snippet_preview(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SnippetPreviewRequest>,
) -> ApiResult<SnippetPreviewResponse> {
    debug!("prompt_snippet_preview");
    let expanded = expand_prompts_pair(
        &state.config,
        &state.prompt_snippets,
        &req.positive,
        &req.negative,
    )
    .await
    .map_err(ApiError::bad_request)?;

    Ok(Json(SnippetPreviewResponse {
        positive: expanded.positive,
        negative: expanded.negative,
        warnings: expanded.warnings,
    }))
}
