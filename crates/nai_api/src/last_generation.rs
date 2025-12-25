use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Context;
use nai_core::dto::{BaseGenerateRequest, CharacterPrompt};
use rusqlite::{Connection, OptionalExtension, params};

use crate::db::Database;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LastGenerationRecord {
    pub updated_at_ms: i64,
    pub base: BaseGenerateRequest,
}

#[derive(Debug, Clone)]
pub struct LastGenerationStore {
    db: Database,
}

impl LastGenerationStore {
    pub fn new(db: Database) -> anyhow::Result<Self> {
        db.with_conn(Self::init_schema)?;
        Ok(Self { db })
    }

    pub async fn get(&self) -> anyhow::Result<Option<LastGenerationRecord>> {
        self.db
            .with_conn_blocking("last_generation get", move |conn| {
                let row: Option<(i64, String, String, String, String, String)> = conn
                    .query_row(
                        "SELECT updated_at_ms, base_json, model, positive, negative, character_prompts_json FROM last_generation WHERE id = 1",
                        [],
                        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?)),
                    )
                    .optional()?;

                let Some((updated_at_ms, base_json, model, positive, negative, character_prompts_json)) = row else {
                    return Ok(None);
                };

                let base: Option<BaseGenerateRequest> = serde_json::from_str(&base_json).ok();
                let base = match base {
                    Some(b) => b,
                    None => {
                        let character_prompts: Vec<CharacterPrompt> =
                            serde_json::from_str(&character_prompts_json).unwrap_or_default();
                        default_base_for_model(&model, &positive, &negative, character_prompts)
                    }
                };

                Ok(Some(LastGenerationRecord { updated_at_ms, base }))
            })
            .await
    }

    pub async fn set_from_base(&self, base: &BaseGenerateRequest) -> anyhow::Result<()> {
        self.set(LastGenerationRecord {
            updated_at_ms: now_ms(),
            base: base.clone(),
        })
        .await
    }

    pub async fn set(&self, record: LastGenerationRecord) -> anyhow::Result<()> {
        self.db
            .with_conn_blocking("last_generation set", move |conn| {
                let base_json = serde_json::to_string(&record.base).context("serialize base")?;
                let character_prompts_json = serde_json::to_string(
                    &record.base.character_prompts.clone().unwrap_or_default(),
                )
                .context("serialize character_prompts")?;

                conn.execute(
                    "INSERT OR REPLACE INTO last_generation (id, updated_at_ms, base_json, model, positive, negative, character_prompts_json) VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        record.updated_at_ms,
                        base_json,
                        record.base.model,
                        record.base.positive,
                        record.base.negative,
                        character_prompts_json,
                    ],
                )?;

                Ok(())
            })
            .await
    }

    pub async fn clear(&self) -> anyhow::Result<()> {
        self.db
            .with_conn_blocking("last_generation clear", move |conn| {
                conn.execute("DELETE FROM last_generation WHERE id = 1", [])?;
                Ok(())
            })
            .await
    }

    fn init_schema(conn: &mut Connection) -> anyhow::Result<()> {
        conn.execute_batch(
            "\
            CREATE TABLE IF NOT EXISTS last_generation (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                updated_at_ms INTEGER NOT NULL,
                base_json TEXT NOT NULL,
                model TEXT NOT NULL,
                positive TEXT NOT NULL,
                negative TEXT NOT NULL,
                character_prompts_json TEXT NOT NULL
            );
            ",
        )
        .context("init last_generation schema")?;

        // Migration: old schema had no base_json.
        let _ = conn.execute(
            "ALTER TABLE last_generation ADD COLUMN base_json TEXT NOT NULL DEFAULT '{}'",
            [],
        );
        Ok(())
    }
}

fn default_base_for_model(
    model: &str,
    positive: &str,
    negative: &str,
    character_prompts: Vec<CharacterPrompt>,
) -> BaseGenerateRequest {
    let is_v3 = model == "nai-diffusion-3" || model == "nai-diffusion-furry-3";
    BaseGenerateRequest {
        model: model.to_string(),
        positive: positive.to_string(),
        negative: negative.to_string(),
        quantity: Some(1),
        width: 832,
        height: 1216,
        steps: 27,
        scale: 5.0,
        sampler: "k_euler_ancestral".to_string(),
        noise_schedule: None,
        cfg_rescale: None,
        seed: -1,
        add_quality_tags: Some(true),
        undesired_content_preset: Some("None".to_string()),
        sm: if is_v3 { Some(false) } else { None },
        sm_dyn: if is_v3 { Some(false) } else { None },
        use_coords: if !is_v3 { Some(true) } else { None },
        legacy_uc: if !is_v3 { Some(false) } else { None },
        character_prompts: if !is_v3 {
            Some(character_prompts)
        } else {
            None
        },
        reference_image_multiple: None,
        reference_information_extracted_multiple: None,
        reference_strength_multiple: None,
    }
}

pub(crate) fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}
