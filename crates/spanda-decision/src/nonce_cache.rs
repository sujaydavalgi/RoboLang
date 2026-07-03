//! Persisted nonce registry for distributed decision replay protection.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

/// On-disk nonce registry (survives reboot).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PersistedNonceRegistry {
    pub version: u32,
    pub seen: HashSet<String>,
    pub updated_at_ms: f64,
}

impl PersistedNonceRegistry {
    /// Empty nonce registry.
    pub fn new() -> Self {
        Self {
            version: 1,
            ..Default::default()
        }
    }

    /// Register a nonce; returns Err on replay.
    pub fn register(&mut self, nonce: &str) -> Result<(), String> {
        if nonce.is_empty() {
            return Err("empty nonce rejected".into());
        }
        if !self.seen.insert(nonce.to_string()) {
            return Err(format!("replayed decision nonce '{nonce}' rejected"));
        }
        self.updated_at_ms = current_time_ms();
        Ok(())
    }

    /// Clear all nonces (tests only).
    pub fn clear(&mut self) {
        self.seen.clear();
    }
}

static REGISTRY_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn registry_lock() -> &'static Mutex<()> {
    REGISTRY_LOCK.get_or_init(|| Mutex::new(()))
}

/// Default path for the nonce registry file.
pub fn default_nonce_registry_path() -> PathBuf {
    std::env::var("SPANDA_DECISION_NONCE_CACHE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(".spanda/decision-nonce-registry.json"))
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

/// Load the persisted nonce registry from disk.
pub fn load_nonce_registry(path: Option<&Path>) -> PersistedNonceRegistry {
    let path = path
        .map(Path::to_path_buf)
        .unwrap_or_else(default_nonce_registry_path);
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return PersistedNonceRegistry::new();
    };
    serde_json::from_str(&raw).unwrap_or_else(|_| PersistedNonceRegistry::new())
}

/// Save the persisted nonce registry to disk.
pub fn save_nonce_registry(
    registry: &mut PersistedNonceRegistry,
    path: Option<&Path>,
) -> Result<(), String> {
    let path = path
        .map(Path::to_path_buf)
        .unwrap_or_else(default_nonce_registry_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create nonce registry dir: {e}"))?;
    }
    registry.updated_at_ms = current_time_ms();
    let body = serde_json::to_string_pretty(registry)
        .map_err(|e| format!("failed to serialize nonce registry: {e}"))?;
    std::fs::write(&path, body).map_err(|e| format!("failed to write nonce registry: {e}"))
}

/// Register a decision nonce with disk persistence; rejects replays.
pub fn register_persisted_nonce(nonce: &str) -> Result<(), String> {
    let _guard = registry_lock()
        .lock()
        .map_err(|_| "nonce registry lock poisoned".to_string())?;
    let mut registry = load_nonce_registry(None);
    registry.register(nonce)?;
    save_nonce_registry(&mut registry, None)
}

/// Clear persisted nonce registry (tests only).
pub fn clear_persisted_nonce_registry() {
    let _guard = registry_lock().lock().ok();
    let mut registry = PersistedNonceRegistry::new();
    let _ = save_nonce_registry(&mut registry, None);
}
