//! In-memory Twin Cloud snapshot store (Control Center SaaS backend).

use crate::snapshot::{TwinCloudSnapshot, TwinCloudSummary, TWIN_CLOUD_API_VERSION};
use std::collections::HashMap;

const DEFAULT_MAX_HISTORY: usize = 50;

/// Thread-safe in-memory twin snapshot registry with per-twin history.
#[derive(Debug)]
pub struct TwinCloudStore {
    latest: HashMap<String, TwinCloudSnapshot>,
    history: HashMap<String, Vec<TwinCloudSnapshot>>,
    max_history_per_twin: usize,
}

impl Default for TwinCloudStore {
    fn default() -> Self {
        Self::new()
    }
}

impl TwinCloudStore {
    pub fn new() -> Self {
        Self {
            latest: HashMap::new(),
            history: HashMap::new(),
            max_history_per_twin: DEFAULT_MAX_HISTORY,
        }
    }

    /// Rebuild store from persisted latest rows and optional history map.
    pub fn from_records(
        latest: Vec<TwinCloudSnapshot>,
        history: HashMap<String, Vec<TwinCloudSnapshot>>,
    ) -> Self {
        let mut store = Self::new();
        for snapshot in latest {
            store.latest.insert(snapshot.twin_id.clone(), snapshot);
        }
        store.history = history;
        store
    }

    /// Backward-compatible hydrate when only latest snapshots were persisted.
    pub fn from_latest_records(snapshots: Vec<TwinCloudSnapshot>) -> Self {
        let mut store = Self::new();
        for snapshot in snapshots {
            store.upsert(snapshot);
        }
        store
    }

    pub fn upsert(&mut self, snapshot: TwinCloudSnapshot) -> TwinCloudSnapshot {
        let twin_id = snapshot.twin_id.clone();
        let history = self.history.entry(twin_id.clone()).or_default();
        history.push(snapshot.clone());
        if history.len() > self.max_history_per_twin {
            history.remove(0);
        }
        self.latest.insert(twin_id, snapshot.clone());
        snapshot
    }

    pub fn get(&self, twin_id: &str) -> Option<&TwinCloudSnapshot> {
        self.latest.get(twin_id)
    }

    pub fn history(&self, twin_id: &str) -> Vec<&TwinCloudSnapshot> {
        self.history
            .get(twin_id)
            .map(|entries| entries.iter().collect())
            .unwrap_or_default()
    }

    pub fn list(&self, tenant_id: Option<&str>) -> Vec<TwinCloudSummary> {
        self.latest
            .values()
            .filter(|snapshot| tenant_id.is_none_or(|tenant| snapshot.tenant_id.as_str() == tenant))
            .map(|snapshot| {
                let mut summary = snapshot.summary();
                summary.history_count = self.history.get(&summary.twin_id).map(|v| v.len()).unwrap_or(0);
                summary
            })
            .collect()
    }

    pub fn list_response(&self, tenant_id: Option<&str>) -> crate::snapshot::TwinCloudListResponse {
        crate::snapshot::TwinCloudListResponse {
            version: TWIN_CLOUD_API_VERSION.into(),
            twins: self.list(tenant_id),
        }
    }

    /// Clone latest snapshots for persistence export.
    pub fn list_owned(&self) -> Vec<TwinCloudSnapshot> {
        self.latest.values().cloned().collect()
    }

    /// Clone full per-twin history for persistence export.
    pub fn history_owned(&self) -> HashMap<String, Vec<TwinCloudSnapshot>> {
        self.history.clone()
    }
}
