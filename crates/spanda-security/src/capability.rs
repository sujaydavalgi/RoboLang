use crate::error::{SecurityError, SecurityResult};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Known package/runtime capability identifiers.
pub fn known_capabilities() -> &'static [&'static str] {
    &[
        "network.outbound",
        "network.inbound",
        "camera.read",
        "lidar.read",
        "imu.read",
        "gps.read",
        "motion.propose",
        "actuator.execute",
        "actuator.execute.safe",
        "serial.port",
        "storage.read",
        "storage.write",
        "ai.inference",
        "ros2.publish",
        "ros2.subscribe",
        "audit.write",
        "audit.read",
        "identity.sign",
        "identity.verify",
        "ledger.anchor",
    ]
}

pub fn is_known_capability(cap: &str) -> bool {
    known_capabilities().contains(&cap)
}

/// Granted capability token (maps to package `[capabilities]` and robot `permissions`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Permission {
    pub capability: String,
}

impl Permission {
    pub fn new(capability: impl Into<String>) -> Self {
        Self {
            capability: capability.into(),
        }
    }
}

/// Set of granted capabilities with runtime enforcement.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapabilitySet {
    granted: HashSet<String>,
    permissive: bool,
}

impl CapabilitySet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn permissive() -> Self {
        Self {
            granted: known_capabilities()
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
            permissive: true,
        }
    }

    pub fn grant(&mut self, capability: impl Into<String>) {
        self.granted.insert(capability.into());
    }

    pub fn grant_all(&mut self, caps: impl IntoIterator<Item = impl Into<String>>) {
        for cap in caps {
            self.grant(cap);
        }
    }

    pub fn has(&self, capability: &str) -> bool {
        self.permissive || self.granted.contains(capability)
    }

    pub fn require(&self, capability: &str) -> SecurityResult<()> {
        if self.has(capability) {
            Ok(())
        } else {
            Err(SecurityError::CapabilityDenied(capability.to_string()))
        }
    }

    pub fn granted(&self) -> impl Iterator<Item = &str> {
        self.granted.iter().map(String::as_str)
    }
}

/// Maps high-level runtime operations to required package capabilities.
pub fn capability_for_operation(operation: &str) -> Option<&'static str> {
    match operation {
        "audit.record" | "audit.append" => Some("audit.write"),
        "audit.export" | "audit.read" => Some("audit.read"),
        "sign" | "identity.sign" => Some("identity.sign"),
        "verify_signature" | "identity.verify" => Some("identity.verify"),
        "ledger.anchor" => Some("ledger.anchor"),
        "actuator.execute" => Some("actuator.execute"),
        "network.publish" => Some("network.outbound"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_enforcement() {
        let mut caps = CapabilitySet::new();
        caps.grant("audit.write");
        assert!(caps.require("audit.write").is_ok());
        assert!(caps.require("ledger.anchor").is_err());
    }
}
