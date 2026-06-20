use crate::capability::CapabilitySet;
use serde::{Deserialize, Serialize};

/// Application-level permissions for package validation and runtime gating.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PackagePermissions {
    pub capabilities: CapabilitySet,
}

impl PackagePermissions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn permissive() -> Self {
        Self {
            capabilities: CapabilitySet::permissive(),
        }
    }

    pub fn from_capabilities(caps: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let mut set = CapabilitySet::new();
        set.grant_all(caps);
        Self {
            capabilities: set,
        }
    }
}
