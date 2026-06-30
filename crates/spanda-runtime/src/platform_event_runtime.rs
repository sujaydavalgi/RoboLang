//! Injectable platform event persistence for subsystems below the telemetry-store layer.
//!
use spanda_audit::{AuditRuntime, PlatformEvent};
use std::sync::{Arc, OnceLock};

/// Extension points for persisting canonical platform event envelopes.
pub trait PlatformEventRuntime: Send + Sync {
    /// Persist one platform event when telemetry persistence is enabled.
    fn record_platform_event(&self, event: &PlatformEvent);
}

/// No-op platform event runtime for tests and runs without telemetry wiring.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopPlatformEventRuntime;

impl PlatformEventRuntime for NoopPlatformEventRuntime {
    fn record_platform_event(&self, _event: &PlatformEvent) {}
}

static PLATFORM_EVENT_RUNTIME: OnceLock<Arc<dyn PlatformEventRuntime>> = OnceLock::new();

/// Install the process-wide platform event runtime from CLI or service bootstrap.
pub fn set_platform_event_runtime(runtime: Arc<dyn PlatformEventRuntime>) {
    let _ = PLATFORM_EVENT_RUNTIME.set(runtime);
}

/// Shared platform event runtime used by subsystem publishers.
pub fn platform_event_runtime() -> Arc<dyn PlatformEventRuntime> {
    PLATFORM_EVENT_RUNTIME
        .get()
        .cloned()
        .unwrap_or_else(|| Arc::new(NoopPlatformEventRuntime))
}

/// Record a platform event to optional audit storage and the injected telemetry runtime.
pub fn publish_platform_event(audit: Option<&mut AuditRuntime>, event: &PlatformEvent) {
    if let Some(rt) = audit {
        let _ = rt.record_platform_event(event);
    }
    platform_event_runtime().record_platform_event(event);
}

#[cfg(test)]
mod tests {
    use super::*;
    use spanda_audit::platform_event::names;
    use spanda_audit::AuditRuntime;
    use serde_json::json;

    #[test]
    fn publish_platform_event_records_audit() {
        let mut audit = AuditRuntime::new("PlatformEventTest", vec![]);
        let event = PlatformEvent::new(names::TRUST_UPDATED, "spanda-trust", json!({"score": 90}))
            .with_entity_id("robot/demo");
        publish_platform_event(Some(&mut audit), &event);
        let exported = audit.export_json().unwrap();
        assert!(exported.contains(names::TRUST_UPDATED));
        assert!(exported.contains("robot/demo"));
    }
}
