//! Telemetry-store-backed platform event runtime bridge.
//!
use spanda_runtime::platform_event_runtime::{
    set_platform_event_runtime, PlatformEventRuntime,
};
use std::sync::Arc;

/// Persist platform events through the global telemetry store.
#[derive(Debug, Default, Clone, Copy)]
pub struct TelemetryStorePlatformEventRuntime;

impl PlatformEventRuntime for TelemetryStorePlatformEventRuntime {
    fn record_platform_event(&self, event: &spanda_audit::PlatformEvent) {
        let _ = crate::record_platform_event(event);
    }
}

/// Register telemetry-store as the default platform event runtime (CLI bootstrap).
pub fn register() {
    set_platform_event_runtime(Arc::new(TelemetryStorePlatformEventRuntime));
}
