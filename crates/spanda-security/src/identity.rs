use crate::trust::TrustLevel;
use serde::{Deserialize, Serialize};
use spanda_audit::DeviceIdentity;

/// Extended device identity with trust metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RobotIdentity {
    pub device: DeviceIdentity,
    pub trust: TrustLevel,
}

impl RobotIdentity {
    pub fn new(id: impl Into<String>, public_key: impl Into<String>) -> Self {
        Self {
            device: DeviceIdentity::new(id, public_key),
            trust: TrustLevel::Trusted,
        }
    }

    pub fn with_trust(mut self, trust: TrustLevel) -> Self {
        self.trust = trust;
        self
    }

    pub fn id(&self) -> &str {
        &self.device.id
    }

    pub fn public_key(&self) -> &str {
        &self.device.public_key
    }

    pub fn signing_key(&self) -> String {
        self.device.default_key()
    }
}

impl From<DeviceIdentity> for RobotIdentity {
    fn from(device: DeviceIdentity) -> Self {
        Self {
            device,
            trust: TrustLevel::Trusted,
        }
    }
}
