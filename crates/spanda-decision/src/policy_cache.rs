//! Local policy cache for offline and edge operation.

use crate::offline::OfflinePolicySpec;
use crate::types::DecisionPolicy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Cached policy bundle maintained on an edge entity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct LocalPolicyCache {
    pub entity_id: String,
    pub safety_rules: Vec<String>,
    pub recovery_playbooks: Vec<String>,
    pub mission_constraints: Vec<String>,
    pub trust_policy_version: String,
    pub capability_requirements: Vec<String>,
    pub approval_rules: Vec<String>,
    pub policies: HashMap<String, DecisionPolicy>,
    pub last_sync_ms: f64,
    pub signature: Option<String>,
}

impl LocalPolicyCache {
    /// Build an empty cache for an entity.
    pub fn new(entity_id: impl Into<String>) -> Self {
        Self {
            entity_id: entity_id.into(),
            ..Default::default()
        }
    }

    /// Look up a cached policy by name.
    pub fn get_policy(&self, name: &str) -> Option<&DecisionPolicy> {
        self.policies.get(name)
    }

    /// Insert or update a cached policy.
    pub fn upsert_policy(&mut self, policy: DecisionPolicy) {
        self.policies.insert(policy.name.clone(), policy);
    }
}

/// On-disk signed offline policy cache (survives reboot).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PersistedPolicyCache {
    pub version: u32,
    pub policies: HashMap<String, OfflinePolicySpec>,
    pub updated_at_ms: f64,
}

impl PersistedPolicyCache {
    /// Empty persisted cache.
    pub fn new() -> Self {
        Self {
            version: 1,
            ..Default::default()
        }
    }

    /// Upsert a signed offline policy entry.
    pub fn upsert_offline_policy(&mut self, spec: OfflinePolicySpec) {
        self.policies.insert(spec.name.clone(), spec);
        self.updated_at_ms = current_time_ms();
    }

    /// Look up a cached offline policy by name.
    pub fn get_offline_policy(&self, name: &str) -> Option<&OfflinePolicySpec> {
        self.policies.get(name)
    }
}

/// Default path for the signed offline policy cache file.
pub fn default_policy_cache_path() -> PathBuf {
    std::env::var("SPANDA_DECISION_POLICY_CACHE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(".spanda/decision-policy-cache.json"))
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

/// Load the persisted offline policy cache from disk.
pub fn load_persisted_policy_cache(path: Option<&Path>) -> PersistedPolicyCache {
    let path = path
        .map(Path::to_path_buf)
        .unwrap_or_else(default_policy_cache_path);
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return PersistedPolicyCache::new();
    };
    serde_json::from_str(&raw).unwrap_or_else(|_| PersistedPolicyCache::new())
}

/// Save the persisted offline policy cache to disk.
pub fn save_persisted_policy_cache(
    cache: &mut PersistedPolicyCache,
    path: Option<&Path>,
) -> Result<(), String> {
    let path = path
        .map(Path::to_path_buf)
        .unwrap_or_else(default_policy_cache_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create cache dir {}: {e}", parent.display()))?;
    }
    cache.updated_at_ms = current_time_ms();
    let body = serde_json::to_string_pretty(cache)
        .map_err(|e| format!("failed to serialize policy cache: {e}"))?;
    std::fs::write(&path, body).map_err(|e| format!("failed to write {}: {e}", path.display()))
}

/// Merge program offline policies with last-known-valid signed entries from disk.
pub fn merge_offline_policies_with_cache(
    mut policies: Vec<OfflinePolicySpec>,
    cache: &PersistedPolicyCache,
) -> Vec<OfflinePolicySpec> {
    for policy in &mut policies {
        let Some(cached) = cache.get_offline_policy(&policy.name) else {
            continue;
        };
        if policy
            .signature
            .as_ref()
            .map(|s| s.is_empty())
            .unwrap_or(true)
        {
            policy.signature = cached.signature.clone();
        }
        if policy.expires_at_ms.is_none() {
            policy.expires_at_ms = cached.expires_at_ms;
        }
        if policy.policy_version == "1.0.0" && cached.policy_version != "1.0.0" {
            policy.policy_version = cached.policy_version.clone();
        }
    }
    policies
}

/// Populate cache from program-extracted policies.
pub fn build_policy_cache(
    entity_id: &str,
    policies: Vec<DecisionPolicy>,
    safety_rules: Vec<String>,
) -> LocalPolicyCache {
    let mut cache = LocalPolicyCache::new(entity_id);
    cache.safety_rules = safety_rules;
    for p in policies {
        cache.upsert_policy(p);
    }
    cache
}
