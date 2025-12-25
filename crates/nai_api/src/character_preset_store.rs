use std::path::PathBuf;

use anyhow::Context;
use nai_core::dto::Center;

use crate::simple_json_store::NameJsonStore;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CharacterSlotPreset {
    pub prompt: String,
    pub uc: String,
    pub center: Center,
}

#[derive(Debug, Clone)]
pub struct CharacterPresetStore {
    inner: NameJsonStore<CharacterSlotPreset>,
}

impl CharacterPresetStore {
    pub fn new(db_path: PathBuf) -> anyhow::Result<Self> {
        Ok(Self {
            inner: NameJsonStore::new(db_path, "character_presets")?,
        })
    }

    pub async fn list_names(&self) -> anyhow::Result<Vec<String>> {
        self.inner.list_names().await
    }

    pub async fn get(&self, name: &str) -> anyhow::Result<Option<CharacterSlotPreset>> {
        self.inner.get(name).await
    }

    pub async fn upsert(&self, name: &str, preset: &CharacterSlotPreset) -> anyhow::Result<()> {
        self.inner.upsert(name, preset).await
    }

    pub async fn delete(&self, name: &str) -> anyhow::Result<bool> {
        self.inner.delete(name).await
    }

    pub async fn rename(&self, from: &str, to: &str) -> anyhow::Result<()> {
        self.inner
            .rename(from, to)
            .await
            .with_context(|| format!("rename character preset: {from} -> {to}"))
    }
}
