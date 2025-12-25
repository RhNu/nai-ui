use std::{path::PathBuf, sync::Arc};

use anyhow::Context;
use rusqlite::Connection;

#[derive(Debug, Clone)]
pub struct Database {
    inner: Arc<SqliteBackend>,
}

impl Database {
    pub fn sqlite(db_path: PathBuf) -> anyhow::Result<Self> {
        Ok(Self {
            inner: Arc::new(SqliteBackend::new(db_path)?),
        })
    }

    pub fn health_check(&self) -> anyhow::Result<()> {
        self.with_conn(|conn| {
            conn.query_row("PRAGMA quick_check", [], |_row| Ok(()))?;
            Ok(())
        })
        .context("sqlite health_check")
    }

    pub fn with_conn<R>(
        &self,
        f: impl FnOnce(&mut Connection) -> anyhow::Result<R>,
    ) -> anyhow::Result<R> {
        let mut conn = self.inner.open()?;
        f(&mut conn)
    }

    pub async fn with_conn_blocking<R: Send + 'static>(
        &self,
        label: &'static str,
        f: impl FnOnce(&mut Connection) -> anyhow::Result<R> + Send + 'static,
    ) -> anyhow::Result<R> {
        let db = self.clone();
        tokio::task::spawn_blocking(move || db.with_conn(f))
            .await
            .with_context(|| format!("join {label}"))?
    }
}

#[derive(Debug, Clone)]
struct SqliteBackend {
    db_path: Arc<PathBuf>,
}

impl SqliteBackend {
    fn new(db_path: PathBuf) -> anyhow::Result<Self> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create sqlite parent dir: {}", parent.display()))?;
        }

        Ok(Self {
            db_path: Arc::new(db_path),
        })
    }

    fn open(&self) -> anyhow::Result<Connection> {
        let conn = Connection::open(&*self.db_path)
            .with_context(|| format!("open sqlite db: {}", self.db_path.display()))?;
        conn.busy_timeout(std::time::Duration::from_secs(3))?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        Ok(conn)
    }
}
