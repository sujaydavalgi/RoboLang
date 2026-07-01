//! Spoofing platform event emission tests.

use spanda_audit::platform_event::names;
use spanda_runtime::platform_event_runtime::{
    set_platform_event_runtime, PlatformEventRuntime,
};
use spanda_spoofing::{generate_trace_spoof_check, MissionTrace, TraceFrame};
use std::sync::{Arc, Mutex};

struct CapturePlatformEvents {
    types: Mutex<Vec<String>>,
}

impl PlatformEventRuntime for CapturePlatformEvents {
    fn record_platform_event(&self, event: &spanda_audit::PlatformEvent) {
        self.types
            .lock()
            .unwrap()
            .push(event.event_type.as_str().to_string());
    }
}

#[test]
fn trace_spoof_check_emits_spoofing_detected() {
    let capture = Arc::new(CapturePlatformEvents {
        types: Mutex::new(Vec::new()),
    });
    let _ = set_platform_event_runtime(Arc::clone(&capture) as Arc<dyn PlatformEventRuntime>);

    let trace = MissionTrace {
        version: 1,
        source: "spoof.trace".into(),
        deterministic: true,
        frames: vec![TraceFrame {
            sim_time_ms: 100.0,
            event: "emit gps.spoofed".into(),
            payload: serde_json::json!({}),
        }],
    };
    let _report = generate_trace_spoof_check(&trace, "spoof.trace");

    let types = capture.types.lock().unwrap();
    assert!(types.iter().any(|t| t == names::SPOOFING_DETECTED));
}
