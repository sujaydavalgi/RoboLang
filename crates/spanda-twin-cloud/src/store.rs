//! In-memory Twin Cloud snapshot store (Control Center SaaS backend).

use crate::snapshot::{TwinCloudSnapshot, TwinCloudSummary, TWIN_CLOUD_API_VERSION};
use std::collections::HashMap;

/// Thread-safe in-memory twin snapshot registry.
#[derive(Debug, Default)]
pub struct TwinCloudStore {
    latest: HashMap<String, TwinCloudSnapshot>,
}

impl TwinCloudStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Rebuild store from persisted snapshot records.
    pub fn from_records(snapshots: Vec<TwinCloudSnapshot>) -> Self {
        let mut store = Self::new();
        for snapshot in snapshots {
            store.upsert(snapshot);
        }
        store
    }

    pub fn upsert(&mut self, snapshot: TwinCloudSnapshot) -> TwinCloudSnapshot {
        let twin_id = snapshot.twin_id.clone();
        self.latest.insert(twin_id, snapshot.clone());
        snapshot
    }

    pub fn get(&self, twin_id: &str) -> Option<&TwinCloudSnapshot> {
        self.latest.get(twin_id)
    }

    pub fn list(&self, tenant_id: Option<&str>) -> Vec<TwinCloudSummary> {
        self.latest
            .values()
            .filter(|snapshot| tenant_id.is_none_or(|tenant| snapshot.tenant_id.as_str() == tenant))
            .map(TwinCloudSnapshot::summary)
            .collect()
    }

    pub fn list_response(&self, tenant_id: Option<&str>) -> crate::snapshot::TwinCloudListResponse {
        crate::snapshot::TwinCloudListResponse {
            version: TWIN_CLOUD_API_VERSION.into(),
            twins: self.list(tenant_id),
        }
    }

    /// Clone all stored snapshots for persistence.
    pub fn list_owned(&self) -> Vec<TwinCloudSnapshot> {
        self.latest.values().cloned().collect()
    }
}
