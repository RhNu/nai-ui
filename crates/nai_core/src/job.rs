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
    status: JobStatus,
    cancel: CancellationToken,
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
        let created_at_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let mut map = self.inner.lock().await;
        map.insert(
            id,
            JobRecord {
                kind: kind.into(),
                created_at_ms,
                status: JobStatus::Queued,
                cancel: cancel.clone(),
            },
        );
        (id, cancel)
    }

    pub async fn set_status(&self, id: Uuid, status: JobStatus) {
        let mut map = self.inner.lock().await;
        if let Some(r) = map.get_mut(&id) {
            r.status = status;
        }
    }

    pub async fn get_status(&self, id: Uuid) -> Option<JobStatus> {
        let map = self.inner.lock().await;
        map.get(&id).map(|r| r.status.clone())
    }

    pub async fn list(&self, limit: usize) -> Vec<JobSummary> {
        let map = self.inner.lock().await;
        let mut items: Vec<JobSummary> = map
            .iter()
            .map(|(id, r)| JobSummary {
                id: *id,
                kind: r.kind.clone(),
                created_at_ms: r.created_at_ms,
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
                r.status = JobStatus::Cancelled;
                true
            }
            None => false,
        }
    }
}
