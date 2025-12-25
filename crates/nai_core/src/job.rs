use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::dto::GenerateResponse;

#[derive(Debug, Clone, Serialize)]
pub struct JobSummary {
    pub id: Uuid,
    pub kind: String,
    pub created_at_ms: u64,
    pub started_at_ms: Option<u64>,
    pub finished_at_ms: Option<u64>,
    pub updated_at_ms: u64,
    pub status: JobStatus,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum JobStatus {
    Queued,
    Running,
    Succeeded { outputs: Vec<GenerateResponse> },
    Failed { error: String },
    Cancelled,
}

#[derive(Clone)]
pub struct JobStore {
    inner: Arc<Mutex<HashMap<Uuid, JobRecord>>>,
}

#[derive(Clone)]
struct JobRecord {
    kind: String,
    created_at_ms: u64,
    started_at_ms: Option<u64>,
    finished_at_ms: Option<u64>,
    updated_at_ms: u64,
    status: JobStatus,
    cancel: CancellationToken,
}

const TERMINAL_WINDOW: usize = 3;

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn is_terminal(status: &JobStatus) -> bool {
    matches!(
        status,
        JobStatus::Succeeded { .. } | JobStatus::Failed { .. } | JobStatus::Cancelled
    )
}

fn apply_status(record: &mut JobRecord, status: JobStatus) {
    let ts = now_ms();
    record.status = status;
    record.updated_at_ms = ts;

    if matches!(record.status, JobStatus::Running) {
        if record.started_at_ms.is_none() {
            record.started_at_ms = Some(ts);
        }
    }

    if is_terminal(&record.status) {
        if record.started_at_ms.is_none() {
            record.started_at_ms = Some(ts);
        }
        record.finished_at_ms = Some(ts);
    }
}

fn prune_terminal_jobs(map: &mut HashMap<Uuid, JobRecord>) {
    let mut terminal: Vec<(Uuid, u64)> = map
        .iter()
        .filter(|(_, r)| is_terminal(&r.status))
        .map(|(id, r)| (*id, r.created_at_ms))
        .collect();

    if terminal.len() <= TERMINAL_WINDOW {
        return;
    }

    terminal.sort_by_key(|(_, created)| *created);
    let excess = terminal.len().saturating_sub(TERMINAL_WINDOW);
    for (id, _) in terminal.into_iter().take(excess) {
        map.remove(&id);
    }
}

impl JobStore {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn create(&self, kind: impl Into<String>) -> (Uuid, CancellationToken) {
        let id = Uuid::new_v4();
        let cancel = CancellationToken::new();
        let created_at_ms = now_ms();
        let mut map = self.inner.lock().await;
        map.insert(
            id,
            JobRecord {
                kind: kind.into(),
                created_at_ms,
                started_at_ms: None,
                finished_at_ms: None,
                updated_at_ms: created_at_ms,
                status: JobStatus::Queued,
                cancel: cancel.clone(),
            },
        );
        (id, cancel)
    }

    pub async fn set_status(&self, id: Uuid, status: JobStatus) {
        let mut map = self.inner.lock().await;
        if let Some(r) = map.get_mut(&id) {
            apply_status(r, status);
        }
        prune_terminal_jobs(&mut map);
    }

    pub async fn get_status(&self, id: Uuid) -> Option<JobStatus> {
        let map = self.inner.lock().await;
        map.get(&id).map(|r| r.status.clone())
    }

    pub async fn list(&self, limit: usize) -> Vec<JobSummary> {
        let mut map = self.inner.lock().await;
        prune_terminal_jobs(&mut map);
        let mut items: Vec<JobSummary> = map
            .iter()
            .map(|(id, r)| JobSummary {
                id: *id,
                kind: r.kind.clone(),
                created_at_ms: r.created_at_ms,
                started_at_ms: r.started_at_ms,
                finished_at_ms: r.finished_at_ms,
                updated_at_ms: r.updated_at_ms,
                status: r.status.clone(),
            })
            .collect();
        items.sort_by_key(|x| x.created_at_ms);
        if items.len() > limit {
            items = items.split_off(items.len() - limit);
        }
        items
    }

    pub async fn cancel(&self, id: Uuid) -> bool {
        let mut map = self.inner.lock().await;
        match map.get_mut(&id) {
            Some(r) => {
                r.cancel.cancel();
                apply_status(r, JobStatus::Cancelled);
                prune_terminal_jobs(&mut map);
                true
            }
            None => false,
        }
    }
}
