use std::collections::HashMap;

use anyhow::Context;
use rusqlite::{Connection, OptionalExtension, params};

use crate::{db::Database, last_generation::now_ms};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct PromptSnippet {
    pub body: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PromptSnippetSummary {
    pub name: String,
    pub tags: Vec<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PromptSnippetStore {
    db: Database,
}

impl PromptSnippetStore {
    pub fn new(db: Database) -> anyhow::Result<Self> {
        db.with_conn(Self::init_schema)?;
        Ok(Self { db })
    }

    pub async fn list(
        &self,
        query: Option<&str>,
        tags: &[String],
    ) -> anyhow::Result<Vec<PromptSnippetSummary>> {
        let q = query.map(|s| s.to_string());
        let tags = normalize_tags(tags.iter().map(|s| s.as_str()));

        self.db
            .with_conn_blocking("snippet list", move |conn| {
                let mut stmt = conn.prepare(
                    "SELECT name, preset_json FROM prompt_snippets ORDER BY updated_at_ms DESC",
                )?;
                let mut rows = stmt.query([])?;
                let mut out = Vec::new();
                while let Some(r) = rows.next()? {
                    let name: String = r.get(0)?;
                    let json: String = r.get(1)?;
                    let snippet: PromptSnippet =
                        serde_json::from_str(&json).context("parse snippet")?;

                    if let Some(q) = &q {
                        let needle = q.to_lowercase();
                        if !name.to_lowercase().contains(&needle)
                            && !snippet
                                .tags
                                .iter()
                                .any(|t| t.to_lowercase().contains(&needle))
                            && snippet
                                .description
                                .as_ref()
                                .map(|d| d.to_lowercase().contains(&needle))
                                .unwrap_or(false)
                        {
                            continue;
                        }
                    }

                    if !tags.is_empty() {
                        let matched = snippet
                            .tags
                            .iter()
                            .any(|t| tags.iter().any(|want| want == &t.to_lowercase()));
                        if !matched {
                            continue;
                        }
                    }

                    out.push(PromptSnippetSummary {
                        name,
                        tags: normalize_tags(snippet.tags.iter().map(|s| s.as_str())),
                        description: snippet.description.clone(),
                    });
                }
                Ok(out)
            })
            .await
    }

    pub async fn get(&self, name: &str) -> anyhow::Result<Option<PromptSnippet>> {
        let name = name.to_string();
        self.db
            .with_conn_blocking("snippet get", move |conn| {
                let row: Option<String> = conn
                    .query_row(
                        "SELECT preset_json FROM prompt_snippets WHERE name = ?1",
                        params![name],
                        |r| r.get(0),
                    )
                    .optional()?;
                let Some(json) = row else {
                    return Ok(None);
                };
                let mut snippet: PromptSnippet =
                    serde_json::from_str(&json).context("parse snippet")?;
                snippet.tags = normalize_tags(snippet.tags.iter().map(|s| s.as_str()));
                Ok(Some(snippet))
            })
            .await
    }

    pub async fn upsert(&self, name: &str, mut snippet: PromptSnippet) -> anyhow::Result<()> {
        snippet.tags = normalize_tags(snippet.tags.iter().map(|s| s.as_str()));
        let name = name.to_string();
        let json = serde_json::to_string(&snippet).context("serialize snippet")?;
        self.db
            .with_conn_blocking("snippet upsert", move |conn| {
                conn.execute(
                    "INSERT INTO prompt_snippets (name, updated_at_ms, preset_json) VALUES (?1, ?2, ?3)\
                     ON CONFLICT(name) DO UPDATE SET updated_at_ms=excluded.updated_at_ms, preset_json=excluded.preset_json",
                    params![name, now_ms(), json],
                )?;
                Ok(())
            })
            .await
    }

    pub async fn delete(&self, name: &str) -> anyhow::Result<bool> {
        let name = name.to_string();
        self.db
            .with_conn_blocking("snippet delete", move |conn| {
                let rows =
                    conn.execute("DELETE FROM prompt_snippets WHERE name = ?1", params![name])?;
                Ok(rows > 0)
            })
            .await
    }

    pub async fn rename(&self, from: &str, to: &str) -> anyhow::Result<()> {
        let from = from.to_string();
        let to = to.to_string();
        self.db
            .with_conn_blocking("snippet rename", move |conn| {
                let tx = conn.transaction()?;

                let exists: Option<i64> = tx
                    .query_row(
                        "SELECT 1 FROM prompt_snippets WHERE name = ?1",
                        params![from],
                        |r| r.get(0),
                    )
                    .optional()?;
                if exists.is_none() {
                    return Ok(());
                }

                let conflict: Option<i64> = tx
                    .query_row(
                        "SELECT 1 FROM prompt_snippets WHERE name = ?1",
                        params![to],
                        |r| r.get(0),
                    )
                    .optional()?;
                if conflict.is_some() {
                    anyhow::bail!("snippet already exists: {to}");
                }

                tx.execute(
                    "UPDATE prompt_snippets SET name = ?2, updated_at_ms = ?3 WHERE name = ?1",
                    params![from, to, now_ms()],
                )?;
                tx.commit()?;
                Ok(())
            })
            .await
    }

    fn init_schema(conn: &mut Connection) -> anyhow::Result<()> {
        conn.execute_batch(
            "\
            CREATE TABLE IF NOT EXISTS prompt_snippets (\
                name TEXT NOT NULL PRIMARY KEY,\
                updated_at_ms INTEGER NOT NULL,\
                preset_json TEXT NOT NULL\
            );\
            ",
        )
        .with_context(|| "init prompt_snippets schema")?;
        Ok(())
    }
}

fn normalize_tags<'a>(tags: impl IntoIterator<Item = &'a str>) -> Vec<String> {
    let mut map = HashMap::<String, ()>::new();
    for t in tags {
        let cleaned = t.trim();
        if cleaned.is_empty() {
            continue;
        }
        map.entry(cleaned.to_lowercase()).or_insert(());
    }
    let mut out: Vec<String> = map.keys().cloned().collect();
    out.sort();
    out
}
