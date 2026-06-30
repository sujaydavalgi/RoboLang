//! Platform event envelope types shared across Spanda subsystems.
//!
//! Canonical event names and categories are defined in `docs/event-model.md` and
//! `scripts/architecture-manifest.yaml` (`event_types`).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{AuditError, AuditResult};

/// Well-known platform event names from `docs/event-model.md`.
pub mod names {
    // Entity
    pub const ENTITY_CREATED: &str = "EntityCreated";
    pub const ENTITY_UPDATED: &str = "EntityUpdated";
    pub const ENTITY_DELETED: &str = "EntityDeleted";
    pub const ENTITY_TAGGED: &str = "EntityTagged";
    pub const ENTITY_RELATED: &str = "EntityRelated";
    // Health
    pub const HEALTH_CHANGED: &str = "HealthChanged";
    pub const HEALTH_CHECK_FAILED: &str = "HealthCheckFailed";
    pub const DEGRADED_MODE_ENTERED: &str = "DegradedModeEntered";
    // Readiness
    pub const READINESS_CHANGED: &str = "ReadinessChanged";
    pub const READINESS_GATE_FAILED: &str = "ReadinessGateFailed";
    // Mission
    pub const MISSION_STARTED: &str = "MissionStarted";
    pub const MISSION_COMPLETED: &str = "MissionCompleted";
    pub const MISSION_ABORTED: &str = "MissionAborted";
    pub const MISSION_PAUSED: &str = "MissionPaused";
    // Recovery
    pub const RECOVERY_TRIGGERED: &str = "RecoveryTriggered";
    pub const RECOVERY_COMPLETED: &str = "RecoveryCompleted";
    pub const RECOVERY_FAILED: &str = "RecoveryFailed";
    // Trust
    pub const TRUST_UPDATED: &str = "TrustUpdated";
    pub const TRUST_GATE_FAILED: &str = "TrustGateFailed";
    // Security
    pub const TAMPER_DETECTED: &str = "TamperDetected";
    pub const SPOOFING_DETECTED: &str = "SpoofingDetected";
    pub const SECRET_ROTATED: &str = "SecretRotated";
    pub const AUTH_FAILED: &str = "AuthFailed";
    // Package
    pub const PACKAGE_INSTALLED: &str = "PackageInstalled";
    pub const PACKAGE_REMOVED: &str = "PackageRemoved";
    pub const PACKAGE_VERIFIED: &str = "PackageVerified";
    // Telemetry
    pub const TRACE_FRAME_RECORDED: &str = "TraceFrameRecorded";
    pub const METRIC_EMITTED: &str = "MetricEmitted";
    pub const LOG_EMITTED: &str = "LogEmitted";
    // Fleet / OTA
    pub const FLEET_MEMBER_JOINED: &str = "FleetMemberJoined";
    pub const FLEET_MEMBER_LEFT: &str = "FleetMemberLeft";
    pub const OTA_ROLLOUT_STARTED: &str = "OtaRolloutStarted";
    pub const OTA_ROLLOUT_COMPLETED: &str = "OtaRolloutCompleted";
}

/// Namespaced platform event type (e.g. `ReadinessChanged`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformEventType(pub String);

impl PlatformEventType {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Common JSON envelope for platform events (telemetry, audit, Control Center).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlatformEvent {
    #[serde(rename = "type")]
    pub event_type: PlatformEventType,
    pub timestamp: DateTime<Utc>,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_id: Option<String>,
    pub payload: Value,
}

impl PlatformEvent {
    pub fn new(
        event_type: impl Into<String>,
        source: impl Into<String>,
        payload: Value,
    ) -> Self {
        Self {
            event_type: PlatformEventType::new(event_type),
            timestamp: Utc::now(),
            source: source.into(),
            entity_id: None,
            payload,
        }
    }

    pub fn with_entity_id(mut self, entity_id: impl Into<String>) -> Self {
        self.entity_id = Some(entity_id.into());
        self
    }

    pub fn with_timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.timestamp = timestamp;
        self
    }

    pub fn namespaced_type(&self) -> String {
        format!("spanda.events.{}", self.event_type.as_str())
    }

    pub fn to_json_string(&self) -> AuditResult<String> {
        serde_json::to_string(self)
            .map_err(|error| AuditError::Serialization(error.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AuditRuntime;
    use serde_json::json;

    #[test]
    fn platform_event_serializes_envelope_fields() {
        let event = PlatformEvent::new(
            "ReadinessChanged",
            "spanda-readiness",
            json!({"score": 0.92}),
        )
        .with_entity_id("robot/warehouse-alpha");

        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["type"], "ReadinessChanged");
        assert_eq!(json["source"], "spanda-readiness");
        assert_eq!(json["entity_id"], "robot/warehouse-alpha");
        assert_eq!(json["payload"]["score"], 0.92);
    }

    #[test]
    fn record_platform_event_round_trips_envelope() {
        let mut rt = AuditRuntime::new("PlatformEvents", vec![]);
        let event = PlatformEvent::new(
            names::ENTITY_CREATED,
            "spanda-api",
            serde_json::json!({"entity_id": "robot/demo"}),
        )
        .with_entity_id("robot/demo");
        rt.record_platform_event(&event).unwrap();
        let exported = rt.export_json().unwrap();
        assert!(exported.contains(names::ENTITY_CREATED));
        assert!(exported.contains("robot/demo"));
        assert!(exported.contains("spanda-api"));
    }
}
