//! Human interaction, wearable telemetry, and spatial session provider contracts.
//!
use super::types::{ProviderMetadata, ProviderResult};
use crate::value::RuntimeValue;

/// Active AR/XR session metadata returned by spatial session providers.
#[derive(Debug, Clone, PartialEq)]
pub struct SpatialSessionInfo {
    pub session_id: String,
    pub device_id: String,
    pub active: bool,
}

/// Start/stop AR/XR sessions and synchronize spatial anchors.
pub trait SpatialSessionProvider: Send + Sync {
    fn metadata(&self) -> ProviderMetadata;
    fn start_session(&mut self, device_id: &str) -> ProviderResult<SpatialSessionInfo>;
    fn stop_session(&mut self, session_id: &str) -> ProviderResult<()>;
    fn publish_anchor(&mut self, session_id: &str, anchor: RuntimeValue) -> ProviderResult<()>;
    fn session_status(&self, session_id: &str) -> Option<SpatialSessionInfo>;
}

/// Heart rate, battery, connectivity, and industrial wearable telemetry.
pub trait WearableTelemetryProvider: Send + Sync {
    fn metadata(&self) -> ProviderMetadata;
    fn read_telemetry(&mut self, device_id: &str) -> ProviderResult<RuntimeValue>;
    fn connectivity_status(&self, device_id: &str) -> bool;
}

/// Voice, gesture, eye, and pose input events (H3 packages).
pub trait HriInputProvider: Send + Sync {
    fn metadata(&self) -> ProviderMetadata;
    fn poll_events(&mut self) -> ProviderResult<Vec<RuntimeValue>>;
}

/// Robot, mission, and readiness overlay layers for AR/XR headsets (H3 packages).
pub trait OverlayProvider: Send + Sync {
    fn metadata(&self) -> ProviderMetadata;
    fn subscribe_overlay(&mut self, layer: &str, device_id: &str) -> ProviderResult<()>;
}
