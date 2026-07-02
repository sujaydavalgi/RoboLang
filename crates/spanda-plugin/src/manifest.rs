//! `spanda.plugin.toml` manifest parsing and validation.

use crate::capability::{validate_capability_list, CapabilitySet};
use crate::error::{PluginError, PluginResult};
use crate::types::{PluginType, CURRENT_API_VERSION, DEFAULT_SPANDA_VERSION_REQ};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const MANIFEST_FILENAME: &str = "spanda.plugin.toml";

/// Root manifest structure for `spanda.plugin.toml`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginManifest {
    pub plugin: PluginSection,
    #[serde(default)]
    pub compatibility: CompatibilitySection,
    #[serde(default)]
    pub capabilities: CapabilitiesSection,
    #[serde(default)]
    pub security: SecuritySection,
    #[serde(default)]
    pub hooks: HooksSection,
    #[serde(default)]
    pub control_center: ControlCenterSection,
    #[serde(default)]
    pub cli: CliSection,
    #[serde(default)]
    pub recovery: RecoverySection,
}

/// `[plugin]` metadata block.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginSection {
    pub name: String,
    pub version: String,
    pub publisher: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(rename = "type")]
    pub plugin_type: String,
}

/// `[compatibility]` version constraints.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompatibilitySection {
    #[serde(default = "default_spanda_version")]
    pub spanda_version: String,
    #[serde(default = "default_api_version")]
    pub api_version: String,
}

fn default_spanda_version() -> String {
    DEFAULT_SPANDA_VERSION_REQ.to_string()
}

fn default_api_version() -> String {
    CURRENT_API_VERSION.to_string()
}

impl Default for CompatibilitySection {
    fn default() -> Self {
        Self {
            spanda_version: default_spanda_version(),
            api_version: default_api_version(),
        }
    }
}

/// `[capabilities]` requested host permissions.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CapabilitiesSection {
    #[serde(default)]
    pub requires: Vec<String>,
}

/// `[security]` sandbox and trust policy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecuritySection {
    #[serde(default)]
    pub signed: bool,
    #[serde(default = "default_true")]
    pub sandbox: bool,
    #[serde(default)]
    pub network: bool,
    #[serde(default = "default_filesystem")]
    pub filesystem: String,
}

fn default_true() -> bool {
    true
}

fn default_filesystem() -> String {
    "read-only".to_string()
}

impl Default for SecuritySection {
    fn default() -> Self {
        Self {
            signed: false,
            sandbox: true,
            network: false,
            filesystem: default_filesystem(),
        }
    }
}

/// Optional lifecycle hook declarations.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct HooksSection {
    #[serde(default)]
    pub enabled: Vec<String>,
}

/// Control Center UI contribution metadata.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ControlCenterSection {
    #[serde(default)]
    pub panels: Vec<UiPanelDecl>,
    #[serde(default)]
    pub entity_tabs: Vec<UiPanelDecl>,
    #[serde(default)]
    pub routes: Vec<UiRouteDecl>,
}

/// One dashboard panel or entity tab contribution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiPanelDecl {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub component: Option<String>,
}

/// One Control Center route contribution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiRouteDecl {
    pub path: String,
    pub title: String,
}

/// CLI command contribution metadata.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CliSection {
    #[serde(default)]
    pub commands: Vec<CliCommandDecl>,
}

/// `[recovery]` orchestrator extension contributions.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct RecoverySection {
    #[serde(default)]
    pub extensions: Vec<RecoveryExtensionDecl>,
}

/// One recovery strategy, playbook, validator, or dashboard extension.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecoveryExtensionDecl {
    pub kind: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub trigger: Option<String>,
}

/// One namespaced CLI command exposed by a plugin.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CliCommandDecl {
    pub name: String,
    #[serde(default)]
    pub namespace: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

impl PluginManifest {
    /// Parse manifest text into a validated struct.
    pub fn parse_str(content: &str) -> PluginResult<Self> {
        // Deserialize TOML and run structural validation.
        //
        // Parameters:
        // - `content` — raw `spanda.plugin.toml` source
        //
        // Returns:
        // Parsed manifest, or a parse/validation error.
        //
        // Options:
        // None.
        //
        // Example:
        // let manifest = PluginManifest::parse_str(toml_text)?;

        let manifest: Self = toml::from_str(content)?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Load manifest from a directory containing `spanda.plugin.toml`.
    pub fn load_from_dir(dir: &Path) -> PluginResult<Self> {
        // Read manifest file from plugin root directory.
        //
        // Parameters:
        // - `dir` — plugin package directory
        //
        // Returns:
        // Parsed manifest.
        //
        // Options:
        // None.
        //
        // Example:
        // let manifest = PluginManifest::load_from_dir(plugin_dir)?;

        let path = dir.join(MANIFEST_FILENAME);
        let content = std::fs::read_to_string(&path)?;
        Self::parse_str(&content)
    }

    /// Validate manifest fields after deserialization.
    pub fn validate(&self) -> PluginResult<()> {
        // Reject empty names and unknown plugin types.
        if self.plugin.name.trim().is_empty() {
            return Err(PluginError::Manifest(
                "plugin.name must not be empty".into(),
            ));
        }
        if self.plugin.version.trim().is_empty() {
            return Err(PluginError::Manifest(
                "plugin.version must not be empty".into(),
            ));
        }
        if PluginType::parse_str(&self.plugin.plugin_type).is_none() {
            return Err(PluginError::Manifest(format!(
                "unknown plugin type: {}",
                self.plugin.plugin_type
            )));
        }
        validate_capability_list(&self.capabilities.requires)?;
        Ok(())
    }

    /// Parsed plugin type enum.
    pub fn plugin_type(&self) -> PluginType {
        PluginType::parse_str(&self.plugin.plugin_type).unwrap_or(PluginType::Readiness)
    }

    /// Requested capabilities as a set.
    pub fn capability_set(&self) -> CapabilitySet {
        CapabilitySet::from_names(&self.capabilities.requires)
    }

    /// Resolve artifact path relative to plugin directory.
    pub fn artifact_path(&self, plugin_dir: &Path, format: &str) -> PathBuf {
        match format {
            "wasm" => plugin_dir.join("plugin.wasm"),
            "native" => {
                #[cfg(target_os = "macos")]
                {
                    plugin_dir.join("libplugin.dylib")
                }
                #[cfg(target_os = "linux")]
                {
                    plugin_dir.join("libplugin.so")
                }
                #[cfg(target_os = "windows")]
                {
                    plugin_dir.join("plugin.dll")
                }
                #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
                {
                    plugin_dir.join("libplugin.so")
                }
            }
            "typescript" => plugin_dir.join("index.js"),
            other => plugin_dir.join(format!("plugin.{other}")),
        }
    }
}
