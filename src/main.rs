use std::sync::Arc;

use axum::Router;
use nai_api::{
    AppState, CharacterPresetStore, LastGenerationStore, PresetStore, PromptPresetStore,
};
use nai_core::{config::AppConfig, job::JobStore, outputs::OutputStore};
use nai_nai::NaiClient;
use tokio::net::TcpListener;
use tokio::sync::Semaphore;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(fmt::layer())
        .init();

    info!("starting nai-ui backend");

    let config = AppConfig::load()?;
    let nai_cli = NaiClient::new(config.token.clone(), config.proxy.clone())?;
    let outputs = OutputStore::new(&config)?;

    let last_generation_db = config.output_dir.join("nai-ui.sqlite");
    let last_generation = LastGenerationStore::new(last_generation_db)?;

    let presets_db = config.output_dir.join("nai-ui.sqlite");
    let presets = PresetStore::new(presets_db)?;
    presets
        .ensure_defaults(&[
            "nai-diffusion-4-5-full",
            "nai-diffusion-4-5-curated",
            "nai-diffusion-4-full",
            "nai-diffusion-4-curated-preview",
            "nai-diffusion-3",
            "nai-diffusion-furry-3",
        ])
        .await?;

    let prompt_presets_db = config.output_dir.join("nai-ui.sqlite");
    let prompt_presets = PromptPresetStore::new(prompt_presets_db)?;
    prompt_presets.ensure_default().await?;

    let character_presets_db = config.output_dir.join("nai-ui.sqlite");
    let character_presets = CharacterPresetStore::new(character_presets_db)?;

    let jobs = JobStore::new();
    let job_sem = Arc::new(Semaphore::new(1));

    info!(bind = %config.bind, output_dir = %config.output_dir.display(), "config loaded");
    info!(
        format_input = config.format_input,
        cool_time = config.cool_time,
        cool_jitter = config.cool_jitter,
        "job pacing"
    );
    info!(max_concurrent_jobs = 1, "job queue");

    let state = Arc::new(AppState {
        config,
        nai: nai_cli,
        outputs,
        jobs,
        job_sem,
        last_generation,
        presets,
        prompt_presets,
        character_presets,
    });

    let app: Router;

    #[cfg(debug_assertions)]
    {
        app = nai_api::router(state.clone())
            .layer(TraceLayer::new_for_http())
            .layer(CorsLayer::permissive());
    }
    #[cfg(not(debug_assertions))]
    {
        app = nai_api::router(state.clone()).layer(TraceLayer::new_for_http());
    }

    let listener = TcpListener::bind(&state.config.bind).await?;
    tracing::info!("listening on http://{}", state.config.bind);
    axum::serve(listener, app).await?;

    Ok(())
}
