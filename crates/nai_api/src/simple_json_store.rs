use std::marker::PhantomData;

use anyhow::Context;
use rusqlite::{Connection, OptionalExtension, params};

use crate::{db::Database, last_generation::now_ms};

#[derive(Debug, Clone)]
pub(crate) struct NameJsonStore<T> {
    db: Database,
    table: &'static str,
    _phantom: PhantomData<T>,
}

impl<T> NameJsonStore<T>
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + Send + 'static,
{
    pub fn new(db: Database, table: &'static str) -> anyhow::Result<Self> {
        db.with_conn(|conn| init_name_table(conn, table))?;
        Ok(Self {
            db,
            table,
            _phantom: PhantomData,
        })
    }

    pub async fn list_names(&self) -> anyhow::Result<Vec<String>> {
        let table = self.table;
        self.db
            .with_conn_blocking("list_names name-json", move |conn| {
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
    }

    pub async fn get(&self, name: &str) -> anyhow::Result<Option<T>> {
        let table = self.table;
        let name = name.to_string();
        self.db
            .with_conn_blocking("get name-json", move |conn| {
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
    }

    pub async fn upsert(&self, name: &str, preset: &T) -> anyhow::Result<()> {
        let table = self.table;
        let name = name.to_string();
        let preset_json = serde_json::to_string(preset).context("serialize preset")?;
        self.db
            .with_conn_blocking("upsert name-json", move |conn| {
                let sql = format!(
                    "INSERT INTO {table} (name, updated_at_ms, preset_json) VALUES (?1, ?2, ?3)\
                     ON CONFLICT(name) DO UPDATE SET updated_at_ms=excluded.updated_at_ms, preset_json=excluded.preset_json"
                );
                conn.execute(&sql, params![name, now_ms(), preset_json])?;
                Ok(())
            })
            .await
    }

    pub async fn delete(&self, name: &str) -> anyhow::Result<bool> {
        let table = self.table;
        let name = name.to_string();
        self.db
            .with_conn_blocking("delete name-json", move |conn| {
                let sql = format!("DELETE FROM {table} WHERE name = ?1");
                let rows = conn.execute(&sql, params![name])?;
                Ok(rows > 0)
            })
            .await
    }

    pub async fn rename(&self, from: &str, to: &str) -> anyhow::Result<()> {
        let table = self.table;
        let from = from.to_string();
        let to = to.to_string();
        self.db
            .with_conn_blocking("rename name-json", move |conn| {
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
