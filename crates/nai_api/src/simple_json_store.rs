use std::{path::PathBuf, sync::Arc};

use anyhow::Context;
use rusqlite::{Connection, OptionalExtension, params};

use crate::last_generation::now_ms;

pub(crate) fn open_sqlite(db_path: &PathBuf) -> anyhow::Result<Connection> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create sqlite parent dir: {}", parent.display()))?;
    }

    let conn = Connection::open(db_path)
        .with_context(|| format!("open sqlite db: {}", db_path.display()))?;
    conn.busy_timeout(std::time::Duration::from_secs(3))?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    Ok(conn)
}

#[derive(Debug, Clone)]
pub(crate) struct NameJsonStore<T> {
    db_path: Arc<PathBuf>,
    table: &'static str,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> NameJsonStore<T>
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + Send + 'static,
{
    pub fn new(db_path: PathBuf, table: &'static str) -> anyhow::Result<Self> {
        let conn = open_sqlite(&db_path)?;
        init_name_table(&conn, table)?;
        Ok(Self {
            db_path: Arc::new(db_path),
            table,
            _phantom: std::marker::PhantomData,
        })
    }

    pub async fn list_names(&self) -> anyhow::Result<Vec<String>> {
        let db_path = self.db_path.clone();
        let table = self.table;
        tokio::task::spawn_blocking(move || {
            let conn = open_sqlite(&db_path)?;
            let sql = format!("SELECT name FROM {table} ORDER BY updated_at_ms DESC");
            let mut stmt = conn.prepare(&sql)?;
            let mut rows = stmt.query([])?;
            let mut out = vec![];
            while let Some(r) = rows.next()? {
                out.push(r.get::<_, String>(0)?);
            }
            Ok(out)
        })
        .await
        .context("join list_names name-json")?
    }

    pub async fn get(&self, name: &str) -> anyhow::Result<Option<T>> {
        let db_path = self.db_path.clone();
        let table = self.table;
        let name = name.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = open_sqlite(&db_path)?;
            let sql = format!("SELECT preset_json FROM {table} WHERE name = ?1");
            let row: Option<String> = conn
                .query_row(&sql, params![name], |r| r.get(0))
                .optional()?;
            let Some(preset_json) = row else {
                return Ok(None);
            };
            let preset = serde_json::from_str(&preset_json).context("parse preset json")?;
            Ok(Some(preset))
        })
        .await
        .context("join get name-json")?
    }

    pub async fn upsert(&self, name: &str, preset: &T) -> anyhow::Result<()> {
        let db_path = self.db_path.clone();
        let table = self.table;
        let name = name.to_string();
        let preset_json = serde_json::to_string(preset).context("serialize preset")?;
        tokio::task::spawn_blocking(move || {
            let conn = open_sqlite(&db_path)?;
            let sql = format!(
                "INSERT INTO {table} (name, updated_at_ms, preset_json) VALUES (?1, ?2, ?3)\
                 ON CONFLICT(name) DO UPDATE SET updated_at_ms=excluded.updated_at_ms, preset_json=excluded.preset_json"
            );
            conn.execute(&sql, params![name, now_ms(), preset_json])?;
            Ok(())
        })
        .await
        .context("join upsert name-json")?
    }

    pub async fn delete(&self, name: &str) -> anyhow::Result<bool> {
        let db_path = self.db_path.clone();
        let table = self.table;
        let name = name.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = open_sqlite(&db_path)?;
            let sql = format!("DELETE FROM {table} WHERE name = ?1");
            let rows = conn.execute(&sql, params![name])?;
            Ok(rows > 0)
        })
        .await
        .context("join delete name-json")?
    }

    pub async fn rename(&self, from: &str, to: &str) -> anyhow::Result<()> {
        let db_path = self.db_path.clone();
        let table = self.table;
        let from = from.to_string();
        let to = to.to_string();
        tokio::task::spawn_blocking(move || {
            let mut conn = open_sqlite(&db_path)?;
            let tx = conn.transaction()?;

            let exists_sql = format!("SELECT 1 FROM {table} WHERE name = ?1");
            let exists: Option<i64> = tx
                .query_row(&exists_sql, params![from], |r| r.get(0))
                .optional()?;
            if exists.is_none() {
                return Ok(());
            }

            let conflict_sql = format!("SELECT 1 FROM {table} WHERE name = ?1");
            let conflict: Option<i64> = tx
                .query_row(&conflict_sql, params![to], |r| r.get(0))
                .optional()?;
            if conflict.is_some() {
                anyhow::bail!("preset already exists: {to}");
            }

            let update_sql =
                format!("UPDATE {table} SET name = ?2, updated_at_ms = ?3 WHERE name = ?1");
            tx.execute(&update_sql, params![from, to, now_ms()])?;
            tx.commit()?;
            Ok(())
        })
        .await
        .context("join rename name-json")?
    }
}

fn init_name_table(conn: &Connection, table: &'static str) -> anyhow::Result<()> {
    let sql = format!(
        "\
        CREATE TABLE IF NOT EXISTS {table} (\
            name TEXT NOT NULL PRIMARY KEY,\
            updated_at_ms INTEGER NOT NULL,\
            preset_json TEXT NOT NULL\
        );\
        "
    );
    conn.execute_batch(&sql)
        .with_context(|| format!("init {table} schema"))?;
    Ok(())
}
