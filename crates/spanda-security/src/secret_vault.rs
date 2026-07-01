//! Managed secret vault contract with rotation and expiration metadata.
//!
use crate::error::{SecurityError, SecurityResult};
use crate::secrets::{SecretHandle, SecretStore};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metadata for a managed secret (values never appear in audit exports).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecretMetadata {
    pub name: String,
    pub created_at_ms: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at_ms: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation_due_at_ms: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_rotated_at_ms: Option<f64>,
    #[serde(default)]
    pub rotation_count: u32,
}

impl SecretMetadata {
    pub fn new(name: impl Into<String>, now_ms: f64) -> Self {
        Self {
            name: name.into(),
            created_at_ms: now_ms,
            expires_at_ms: None,
            rotation_due_at_ms: None,
            last_rotated_at_ms: None,
            rotation_count: 0,
        }
    }

    pub fn is_expired(&self, now_ms: f64) -> bool {
        self.expires_at_ms.is_some_and(|exp| now_ms >= exp)
    }

    pub fn needs_rotation(&self, now_ms: f64) -> bool {
        self.rotation_due_at_ms.is_some_and(|due| now_ms >= due)
    }
}

/// Contract for optional package backends (Vault, AWS SM, K8s secrets).
pub trait SecretVaultBackend: Send + Sync {
    fn store(&mut self, name: &str, value: &str) -> SecurityResult<()>;
    fn metadata(&self, name: &str) -> SecurityResult<SecretMetadata>;
    fn rotate(
        &mut self,
        name: &str,
        new_value: &str,
        now_ms: f64,
    ) -> SecurityResult<SecretMetadata>;
}

/// In-core managed vault wrapping [`SecretStore`] with audit-safe metadata.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ManagedSecretVault {
    store: SecretStore,
    metadata: HashMap<String, SecretMetadata>,
}

impl ManagedSecretVault {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, handle: SecretHandle, meta: SecretMetadata) {
        let name = handle.name.clone();
        self.store.register(handle);
        self.metadata.insert(name, meta);
    }

    pub fn resolve(&self, name: &str) -> SecurityResult<String> {
        let meta = self.metadata.get(name);
        if let Some(m) = meta {
            let now = now_ms();
            if m.is_expired(now) {
                return Err(SecurityError::SecretNotFound(format!(
                    "secret '{name}' expired"
                )));
            }
        }
        self.store.resolve(name)
    }

    pub fn list_metadata(&self) -> Vec<SecretMetadata> {
        let mut items: Vec<_> = self.metadata.values().cloned().collect();
        items.sort_by(|a, b| a.name.cmp(&b.name));
        items
    }

    pub fn rotate_literal(
        &mut self,
        name: &str,
        new_value: &str,
    ) -> SecurityResult<SecretMetadata> {
        let handle = self.store.get(name)?;
        let now = now_ms();
        let mut meta = self
            .metadata
            .get(name)
            .cloned()
            .unwrap_or_else(|| SecretMetadata::new(name, now));
        meta.last_rotated_at_ms = Some(now);
        meta.rotation_count += 1;
        if let crate::secrets::SecretSource::Literal { value } = &handle.source {
            let _ = value;
        }
        self.store.register(SecretHandle {
            name: name.to_string(),
            source: crate::secrets::SecretSource::Literal {
                value: new_value.to_string(),
            },
        });
        self.metadata.insert(name.to_string(), meta.clone());
        crate::platform_events::record_secret_rotated(name, meta.rotation_count);
        Ok(meta)
    }
}

fn now_ms() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64() * 1000.0)
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::secrets::SecretSource;

    #[test]
    fn vault_lists_metadata_without_values() {
        let mut vault = ManagedSecretVault::new();
        vault.register(
            SecretHandle {
                name: "robot_cred".into(),
                source: SecretSource::Literal {
                    value: "secret-value".into(),
                },
            },
            SecretMetadata::new("robot_cred", 1.0),
        );
        assert_eq!(vault.list_metadata().len(), 1);
        assert_eq!(vault.resolve("robot_cred").unwrap(), "secret-value");
    }
}
