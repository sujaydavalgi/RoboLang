//! Plugin extension points for recovery strategies, validators, and playbooks.
//!
use crate::types::{OrchestratorStrategy, PluginRecoveryExtension};
use std::collections::HashMap;

/// Registry of plugin-contributed recovery extensions.
#[derive(Debug, Clone, Default)]
pub struct RecoveryPluginRegistry {
    extensions: HashMap<String, Vec<PluginRecoveryExtension>>,
}

impl RecoveryPluginRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a plugin recovery extension.
    pub fn register(&mut self, extension: PluginRecoveryExtension) {
        self.extensions
            .entry(extension.extension_kind.clone())
            .or_default()
            .push(extension);
    }

    /// List extensions of a given kind (strategy, validator, playbook, etc.).
    pub fn list(&self, kind: &str) -> Vec<&PluginRecoveryExtension> {
        self.extensions
            .get(kind)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    /// Resolve custom plugin strategy by name.
    pub fn resolve_strategy(&self, name: &str) -> Option<OrchestratorStrategy> {
        self.extensions.get("strategy").and_then(|exts| {
            exts.iter()
                .find(|e| e.name == name)
                .map(|e| OrchestratorStrategy::Custom(e.name.clone()))
        })
    }

    /// All registered extensions.
    pub fn all(&self) -> Vec<&PluginRecoveryExtension> {
        self.extensions.values().flat_map(|v| v.iter()).collect()
    }
}

/// Built-in plugin hook kinds supported by the orchestrator.
pub const PLUGIN_KIND_STRATEGY: &str = "strategy";
pub const PLUGIN_KIND_POLICY: &str = "policy";
pub const PLUGIN_KIND_VALIDATOR: &str = "validator";
pub const PLUGIN_KIND_REPORT: &str = "report";
pub const PLUGIN_KIND_DASHBOARD: &str = "dashboard";
pub const PLUGIN_KIND_PLAYBOOK: &str = "playbook";
pub const PLUGIN_KIND_SIMULATOR: &str = "simulator";
pub const PLUGIN_KIND_INTEGRATION: &str = "integration";
