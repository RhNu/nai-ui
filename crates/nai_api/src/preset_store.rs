use anyhow::Context;
use rusqlite::{Connection, OptionalExtension, params};

use crate::{db::Database, last_generation::now_ms};

pub const DEFAULT_PRESET_NAME: &str = "默认";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GeneratePreset {
    pub quantity: u32,
    pub width: u32,
    pub height: u32,
    pub steps: u32,
    pub scale: f32,
    pub sampler: String,
    pub noise_schedule: Option<String>,
    pub cfg_rescale: Option<f32>,
    pub seed: i64,
    pub add_quality_tags: bool,
    pub undesired_content_preset: String,
    pub sm: bool,
    pub sm_dyn: bool,
    pub use_coords: bool,
    pub legacy_uc: bool,
}

#[derive(Debug, Clone)]
pub struct PresetStore {
    db: Database,
}

impl PresetStore {
    pub fn new(db: Database) -> anyhow::Result<Self> {
        db.with_conn(Self::init_schema)?;
        Ok(Self { db })
    }

    pub async fn ensure_defaults(&self, models: &[&str]) -> anyhow::Result<()> {
        let models = models.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        self.db
            .with_conn_blocking("ensure_defaults", move |conn| {
                for m in models {
                    let exists: Option<i64> = conn
                        .query_row(
                            "SELECT 1 FROM presets WHERE model = ?1 AND name = ?2",
                            params![m, DEFAULT_PRESET_NAME],
                            |r| r.get(0),
                        )
                        .optional()?;

                    if exists.is_some() {
                        continue;
                    }

                    let preset = default_preset_for_model(&m);
                    let preset_json = serde_json::to_string(&preset).context("serialize preset")?;
                    conn.execute(
                        "INSERT INTO presets (model, name, updated_at_ms, preset_json) VALUES (?1, ?2, ?3, ?4)",
                        params![m, DEFAULT_PRESET_NAME, now_ms(), preset_json],
                    )?;
                }
                Ok(())
            })
            .await
    }

    pub async fn list_names(&self, model: &str) -> anyhow::Result<Vec<String>> {
        let model = model.to_string();
        self.db
            .with_conn_blocking("list preset names", move |conn| {
                let mut stmt = conn.prepare(
                    "SELECT name FROM presets WHERE model = ?1 ORDER BY updated_at_ms DESC",
                )?;
                let mut rows = stmt.query(params![model])?;
                let mut out = vec![];
                while let Some(r) = rows.next()? {
                    out.push(r.get::<_, String>(0)?);
                }
                Ok(out)
            })
            .await
    }

    pub async fn get(&self, model: &str, name: &str) -> anyhow::Result<Option<GeneratePreset>> {
        let model = model.to_string();
        let name = name.to_string();
        self.db
            .with_conn_blocking("get preset", move |conn| {
                let row: Option<String> = conn
                    .query_row(
                        "SELECT preset_json FROM presets WHERE model = ?1 AND name = ?2",
                        params![model, name],
                        |r| r.get(0),
                    )
                    .optional()?;
                let Some(preset_json) = row else {
                    return Ok(None);
                };
                let preset = serde_json::from_str(&preset_json).context("parse preset json")?;
                Ok(Some(preset))
            })
            .await
    }

    pub async fn upsert(
        &self,
        model: &str,
        name: &str,
        preset: &GeneratePreset,
    ) -> anyhow::Result<()> {
        let model = model.to_string();
        let name = name.to_string();
        let preset_json = serde_json::to_string(preset).context("serialize preset")?;

        self.db
            .with_conn_blocking("upsert preset", move |conn| {
                conn.execute(
                    "INSERT INTO presets (model, name, updated_at_ms, preset_json) VALUES (?1, ?2, ?3, ?4)\
                     ON CONFLICT(model, name) DO UPDATE SET updated_at_ms=excluded.updated_at_ms, preset_json=excluded.preset_json",
                    params![model, name, now_ms(), preset_json],
                )?;
                Ok(())
            })
            .await
    }

    pub async fn delete(&self, model: &str, name: &str) -> anyhow::Result<bool> {
        let model = model.to_string();
        let name = name.to_string();
        self.db
            .with_conn_blocking("delete preset", move |conn| {
                let rows = conn.execute(
                    "DELETE FROM presets WHERE model = ?1 AND name = ?2",
                    params![model, name],
                )?;
                Ok(rows > 0)
            })
            .await
    }

    pub async fn rename(&self, model: &str, from: &str, to: &str) -> anyhow::Result<()> {
        let model = model.to_string();
        let from = from.to_string();
        let to = to.to_string();

        self.db
            .with_conn_blocking("rename preset", move |conn| {
                let tx = conn.transaction()?;

                let exists: Option<i64> = tx
                    .query_row(
                        "SELECT 1 FROM presets WHERE model = ?1 AND name = ?2",
                        params![model, from],
                        |r| r.get(0),
                    )
                    .optional()?;
                if exists.is_none() {
                    return Ok(());
                }

                let conflict: Option<i64> = tx
                    .query_row(
                        "SELECT 1 FROM presets WHERE model = ?1 AND name = ?2",
                        params![model, to],
                        |r| r.get(0),
                    )
                    .optional()?;
                if conflict.is_some() {
                    anyhow::bail!("preset already exists: {to}");
                }

                tx.execute(
                    "UPDATE presets SET name = ?3, updated_at_ms = ?4 WHERE model = ?1 AND name = ?2",
                    params![model, from, to, now_ms()],
                )?;
                tx.commit()?;
                Ok(())
            })
            .await
    }

    fn init_schema(conn: &mut Connection) -> anyhow::Result<()> {
        conn.execute_batch(
            "\
            CREATE TABLE IF NOT EXISTS presets (
                model TEXT NOT NULL,
                name TEXT NOT NULL,
                updated_at_ms INTEGER NOT NULL,
                preset_json TEXT NOT NULL,
                PRIMARY KEY(model, name)
            );
            ",
        )
        .context("init presets schema")?;
        Ok(())
    }
}

fn default_preset_for_model(model: &str) -> GeneratePreset {
    let is_v3 = model == "nai-diffusion-3" || model == "nai-diffusion-furry-3";
    GeneratePreset {
        quantity: 1,
        width: 832,
        height: 1216,
        steps: 27,
        scale: 5.0,
        sampler: "k_euler_ancestral".to_string(),
        noise_schedule: "karras".to_string().into(),
        cfg_rescale: None,
        seed: -1,
        add_quality_tags: true,
        undesired_content_preset: "None".to_string(),
        sm: is_v3 && false,
        sm_dyn: is_v3 && false,
        use_coords: !is_v3 && true,
        legacy_uc: !is_v3 && false,
    }
}
