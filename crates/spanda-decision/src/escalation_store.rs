//! Persistent escalation approval store for distributed decisions.

use crate::types::DecisionEscalation;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Granted escalation record persisted on disk.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EscalationGrant {
    pub escalation_id: String,
    pub entity_id: String,
    pub approver: String,
    pub approved_at_ms: f64,
    pub reason: String,
}

/// Pending escalation awaiting human approval.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PendingEscalation {
    pub escalation_id: String,
    pub entity_id: String,
    pub action: String,
    pub reason: String,
    pub created_at_ms: f64,
}

/// On-disk escalation store (survives reboot and Control Center restart).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PersistedEscalationStore {
    pub version: u32,
    pub pending: HashMap<String, PendingEscalation>,
    pub granted: HashMap<String, EscalationGrant>,
    pub updated_at_ms: f64,
}

impl PersistedEscalationStore {
    /// Empty escalation store.
    pub fn new() -> Self {
        Self {
            version: 1,
            ..Default::default()
        }
    }

    /// Record a pending escalation.
    pub fn register_pending(&mut self, pending: PendingEscalation) {
        self.pending.insert(pending.escalation_id.clone(), pending);
        self.updated_at_ms = current_time_ms();
    }

    /// Grant an escalation and remove from pending.
    pub fn grant(&mut self, escalation_id: &str, approver: &str) -> Result<EscalationGrant, String> {
        let pending = self
            .pending
            .remove(escalation_id)
            .ok_or_else(|| format!("escalation '{escalation_id}' not found in pending store"))?;
        let grant = EscalationGrant {
            escalation_id: escalation_id.to_string(),
            entity_id: pending.entity_id,
            approver: approver.to_string(),
            approved_at_ms: current_time_ms(),
            reason: pending.reason,
        };
        self.granted.insert(escalation_id.to_string(), grant.clone());
        self.updated_at_ms = current_time_ms();
        Ok(grant)
    }

    /// Grant by ID even when not pending (API approval path).
    pub fn grant_direct(
        &mut self,
        escalation_id: &str,
        entity_id: &str,
        approver: &str,
        reason: &str,
    ) -> EscalationGrant {
        self.pending.remove(escalation_id);
        let grant = EscalationGrant {
            escalation_id: escalation_id.to_string(),
            entity_id: entity_id.to_string(),
            approver: approver.to_string(),
            approved_at_ms: current_time_ms(),
            reason: reason.to_string(),
        };
        self.granted.insert(escalation_id.to_string(), grant.clone());
        self.updated_at_ms = current_time_ms();
        grant
    }

    /// Check whether an escalation has been granted.
    pub fn is_granted(&self, escalation_id: &str) -> bool {
        self.granted.contains_key(escalation_id)
    }
}

/// Default path for the escalation store file.
pub fn default_escalation_store_path() -> PathBuf {
    std::env::var("SPANDA_DECISION_ESCALATION_STORE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(".spanda/decision-escalations.json"))
}

fn current_time_ms() -> f64 {
    std::env::var("SPANDA_SIM_TIME_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as f64)
                .unwrap_or(0.0)
        })
}

/// Load the persisted escalation store from disk.
pub fn load_escalation_store(path: Option<&Path>) -> PersistedEscalationStore {
    let path = path
        .map(Path::to_path_buf)
        .unwrap_or_else(default_escalation_store_path);
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return PersistedEscalationStore::new();
    };
    serde_json::from_str(&raw).unwrap_or_else(|_| PersistedEscalationStore::new())
}

/// Save the persisted escalation store to disk.
pub fn save_escalation_store(
    store: &mut PersistedEscalationStore,
    path: Option<&Path>,
) -> Result<(), String> {
    let path = path
        .map(Path::to_path_buf)
        .unwrap_or_else(default_escalation_store_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create escalation store dir: {e}"))?;
    }
    store.updated_at_ms = current_time_ms();
    let body = serde_json::to_string_pretty(store)
        .map_err(|e| format!("failed to serialize escalation store: {e}"))?;
    std::fs::write(&path, body).map_err(|e| format!("failed to write escalation store: {e}"))
}

/// Check whether an escalation is approved (persisted store + env fallback).
pub fn escalation_is_approved(escalation_id: &str) -> bool {
    if load_escalation_store(None).is_granted(escalation_id) {
        return true;
    }
    std::env::var("SPANDA_DECISION_ESCALATION_APPROVED")
        .map(|v| matches!(v.as_str(), "1" | "true" | "yes") || v == escalation_id)
        .unwrap_or(false)
}

/// Register a pending escalation in the persistent store.
pub fn register_pending_escalation(
    escalation_id: &str,
    entity_id: &str,
    action: &str,
    reason: &str,
) -> Result<(), String> {
    let mut store = load_escalation_store(None);
    store.register_pending(PendingEscalation {
        escalation_id: escalation_id.to_string(),
        entity_id: entity_id.to_string(),
        action: action.to_string(),
        reason: reason.to_string(),
        created_at_ms: current_time_ms(),
    });
    save_escalation_store(&mut store, None)
}

/// Approve an escalation and persist the grant.
pub fn approve_escalation_persisted(
    escalation_id: &str,
    approver: &str,
    entity_id: Option<&str>,
) -> Result<EscalationGrant, String> {
    let mut store = load_escalation_store(None);
    let grant = if store.pending.contains_key(escalation_id) {
        store.grant(escalation_id, approver)?
    } else {
        store.grant_direct(
            escalation_id,
            entity_id.unwrap_or("unknown"),
            approver,
            "approved via API",
        )
    };
    save_escalation_store(&mut store, None)?;
    Ok(grant)
}

/// Approve a pending escalation struct (legacy API compat).
pub fn approve_escalation_with_store(
    escalation: &mut DecisionEscalation,
    approver: &str,
) -> Result<(), String> {
    if !escalation.pending_approval {
        return Err("escalation does not require approval".into());
    }
    approve_escalation_persisted(&escalation.escalation_id, approver, Some(&escalation.entity_id))?;
    escalation.pending_approval = false;
    escalation.reason = format!("{} (approved by {approver})", escalation.reason);
    Ok(())
}
