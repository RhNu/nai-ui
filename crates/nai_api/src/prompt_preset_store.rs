use anyhow::Context;
use nai_core::dto::CharacterPrompt;

use crate::{db::Database, simple_json_store::NameJsonStore};

pub const DEFAULT_PROMPT_PRESET_NAME: &str = "默认";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PromptPreset {
    pub positive: String,
    pub negative: String,
    pub add_quality_tags: Option<bool>,
    pub undesired_content_preset: Option<String>,
    pub character_prompts: Vec<CharacterPrompt>,
}

#[derive(Debug, Clone)]
pub struct PromptPresetStore {
    inner: NameJsonStore<PromptPreset>,
}

impl PromptPresetStore {
    pub fn new(db: Database) -> anyhow::Result<Self> {
        Ok(Self {
            inner: NameJsonStore::new(db, "prompt_presets")?,
        })
    }

    pub async fn ensure_default(&self) -> anyhow::Result<()> {
        if self.get(DEFAULT_PROMPT_PRESET_NAME).await?.is_some() {
            return Ok(());
        }

        let preset = PromptPreset {
            positive: String::new(),
            negative: String::new(),
            add_quality_tags: None,
            undesired_content_preset: None,
            character_prompts: vec![],
        };
        self.upsert(DEFAULT_PROMPT_PRESET_NAME, &preset).await
    }

    pub async fn list_names(&self) -> anyhow::Result<Vec<String>> {
        self.inner.list_names().await
    }

    pub async fn get(&self, name: &str) -> anyhow::Result<Option<PromptPreset>> {
        self.inner.get(name).await
    }

    pub async fn upsert(&self, name: &str, preset: &PromptPreset) -> anyhow::Result<()> {
        self.inner.upsert(name, preset).await
    }

    pub async fn delete(&self, name: &str) -> anyhow::Result<bool> {
        self.inner.delete(name).await
    }

    pub async fn rename(&self, from: &str, to: &str) -> anyhow::Result<()> {
        self.inner
            .rename(from, to)
            .await
            .with_context(|| format!("rename prompt preset: {from} -> {to}"))
    }
}
