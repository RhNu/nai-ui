use std::sync::Arc;

use axum::Router;
use tokio::sync::Semaphore;
use tower_http::services::{ServeDir, ServeFile};

use nai_core::{config::AppConfig, job::JobStore, outputs::OutputStore};
use nai_nai::NaiClient;

use crate::{CharacterPresetStore, LastGenerationStore, PresetStore, PromptPresetStore};

mod character_presets;
mod director;
mod error;
mod generate;
mod jobs;
mod last_generation;
mod meta;
mod outputs;
mod presets;
mod prompt_presets;

pub use error::{ApiError, ApiResult};

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub nai: NaiClient,
    pub outputs: OutputStore,
    pub jobs: JobStore,
    pub job_sem: Arc<Semaphore>,
    pub last_generation: LastGenerationStore,
    pub presets: PresetStore,
    pub prompt_presets: PromptPresetStore,
    pub character_presets: CharacterPresetStore,
}

pub fn router(state: Arc<AppState>) -> Router {
    let outputs_root = state.outputs.root().to_path_buf();

    let mut router = Router::<Arc<AppState>>::new()
        .nest_service("/outputs", ServeDir::new(outputs_root))
        .merge(meta::routes())
        .merge(outputs::routes())
        .merge(last_generation::routes())
        .merge(presets::routes())
        .merge(prompt_presets::routes())
        .merge(character_presets::routes())
        .merge(generate::routes())
        .merge(jobs::routes())
        .merge(director::routes())
        .with_state(state.clone());

    if let Some(static_dir) = state.config.static_dir.clone() {
        let index_file = static_dir.join("index.html");
        let service = ServeDir::new(static_dir).not_found_service(ServeFile::new(index_file));
        router = router.fallback_service(service);
    }

    router
}
