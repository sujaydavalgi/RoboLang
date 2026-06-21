//! Trust boundary model for cross-domain secure communication validation.

use crate::error::{SecurityError, SecurityResult};
use crate::policy::{AuthenticationMode, EncryptionMode, IntegrityMode};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::str::FromStr;

/// Named trust boundary declared in Spanda source (`trust_boundary robot_to_robot;`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustBoundaryKind {
    RobotInternal,
    RobotToRobot,
    RobotToCloud,
    OperatorToRobot,
}

impl FromStr for TrustBoundaryKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "robot_internal" => Ok(Self::RobotInternal),
            "robot_to_robot" => Ok(Self::RobotToRobot),
            "robot_to_cloud" => Ok(Self::RobotToCloud),
            "operator_to_robot" => Ok(Self::OperatorToRobot),
            other => Err(format!("unknown trust boundary '{other}'")),
        }
    }
}

/// Map a transport name to the trust boundary it typically crosses.
pub fn boundary_for_transport_name(transport: &str) -> Option<TrustBoundaryKind> {
    match transport {
        "local" | "sim" | "ble" => Some(TrustBoundaryKind::RobotInternal),
        "ros2" | "dds" | "mqtt" => Some(TrustBoundaryKind::RobotToRobot),
        "websocket" => Some(TrustBoundaryKind::OperatorToRobot),
        "wifi" | "cellular" => Some(TrustBoundaryKind::RobotToCloud),
        _ => None,
    }
}

impl TrustBoundaryKind {
    pub fn required_encryption(self) -> EncryptionMode {
        match self {
            Self::RobotInternal => EncryptionMode::Optional,
            Self::RobotToRobot | Self::RobotToCloud | Self::OperatorToRobot => {
                EncryptionMode::Required
            }
        }
    }

    pub fn required_authentication(self) -> AuthenticationMode {
        match self {
            Self::OperatorToRobot => AuthenticationMode::Mutual,
            _ => AuthenticationMode::None,
        }
    }

    pub fn requires_verified_actuator(self) -> bool {
        matches!(
            self,
            Self::RobotToRobot | Self::RobotToCloud | Self::OperatorToRobot
        )
    }
}

/// Registry of declared trust boundaries for compile-time and runtime checks.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrustBoundaryRegistry {
    boundaries: HashSet<TrustBoundaryKind>,
}

impl TrustBoundaryRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn declare(&mut self, boundary: TrustBoundaryKind) {
        self.boundaries.insert(boundary);
    }

    pub fn contains(&self, boundary: TrustBoundaryKind) -> bool {
        self.boundaries.contains(&boundary)
    }

    pub fn validate_channel(
        &self,
        boundary: TrustBoundaryKind,
        encryption: EncryptionMode,
        authentication: AuthenticationMode,
        integrity: IntegrityMode,
        message_type: &str,
    ) -> SecurityResult<()> {
        let req_enc = boundary.required_encryption();
        if req_enc == EncryptionMode::Required && encryption != EncryptionMode::Required {
            return Err(SecurityError::SecureEndpoint {
                endpoint: boundary.as_str().into(),
                reason: format!(
                    "encryption required for {message_type} crossing {}",
                    boundary.as_str()
                ),
            });
        }
        let req_auth = boundary.required_authentication();
        if req_auth == AuthenticationMode::Mutual && authentication != AuthenticationMode::Mutual {
            return Err(SecurityError::SecureEndpoint {
                endpoint: boundary.as_str().into(),
                reason: format!(
                    "mutual authentication required for {message_type} crossing {}",
                    boundary.as_str()
                ),
            });
        }
        if boundary.requires_verified_actuator()
            && message_type == "SafeAction"
            && (encryption != EncryptionMode::Required || integrity != IntegrityMode::Required)
        {
            return Err(SecurityError::SecureEndpoint {
                endpoint: boundary.as_str().into(),
                reason: "SafeAction crossing trust boundary requires encryption and integrity"
                    .into(),
            });
        }
        Ok(())
    }
}

impl TrustBoundaryKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RobotInternal => "robot_internal",
            Self::RobotToRobot => "robot_to_robot",
            Self::RobotToCloud => "robot_to_cloud",
            Self::OperatorToRobot => "operator_to_robot",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn robot_to_robot_requires_encryption() {
        let reg = TrustBoundaryRegistry::new();
        let err = reg
            .validate_channel(
                TrustBoundaryKind::RobotToRobot,
                EncryptionMode::None,
                AuthenticationMode::None,
                IntegrityMode::None,
                "Velocity",
            )
            .unwrap_err();
        assert!(matches!(err, SecurityError::SecureEndpoint { .. }));
    }
}
