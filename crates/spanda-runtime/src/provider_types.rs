//! Shared types for lean-core provider contracts.
//!
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Stable identifier for a registered provider implementation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProviderId {
    pub package: String,
    pub name: String,
}

impl ProviderId {
    pub fn new(package: impl Into<String>, name: impl Into<String>) -> Self {
        // Build a provider id from package and implementation name.
        //
        // Parameters:
        // - `package` — owning package name (e.g. `spanda-gps`)
        // - `name` — provider implementation name within the package
        //
        // Returns:
        // Stable provider identifier.
        //
        // Options:
        // None.
        //
        // Example:

        // let id = ProviderId::new("spanda-gps", "nmea");

        Self {
            package: package.into(),
            name: name.into(),
        }
    }
}

/// Capability tokens a provider may require from the runtime or other packages.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProviderCapability(pub String);

impl ProviderCapability {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

/// Safety tier declared by a provider package.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderSafetyLevel {
    Experimental,
    Development,
    Production,
}

/// Metadata describing a registered provider.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProviderMetadata {
    pub id: ProviderId,
    pub description: String,
    pub safety_level: ProviderSafetyLevel,
    pub capabilities_required: Vec<ProviderCapability>,
    pub hardware_requirements: Vec<String>,
}

/// Runtime error returned by provider operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderError {
    pub provider: ProviderId,
    pub message: String,
}

impl ProviderError {
    pub fn new(provider: ProviderId, message: impl Into<String>) -> Self {
        Self {
            provider,
            message: message.into(),
        }
    }
}

pub type ProviderResult<T> = Result<T, ProviderError>;

/// In-memory capability set used by the provider registry.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProviderCapabilitySet {
    inner: HashSet<String>,
}

impl ProviderCapabilitySet {
    pub fn new() -> Self {
        Self {
            inner: HashSet::new(),
        }
    }

    pub fn insert(&mut self, cap: impl Into<String>) {
        self.inner.insert(cap.into());
    }

    pub fn contains(&self, cap: &str) -> bool {
        self.inner.contains(cap)
    }

    pub fn iter(&self) -> impl Iterator<Item = &String> {
        self.inner.iter()
    }
}
