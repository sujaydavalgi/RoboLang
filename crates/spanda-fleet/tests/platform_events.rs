//! Fleet platform event emission tests.

use spanda_audit::platform_event::names;
use spanda_fleet::{deregister_fleet_agent, register_fleet_agent, FleetAgentRegistry};
use spanda_runtime::platform_event_runtime::{
    set_platform_event_runtime, PlatformEventRuntime,
};
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
fn fleet_registry_lifecycle_emits_member_left() {
    let capture = Arc::new(CapturePlatformEvents {
        types: Mutex::new(Vec::new()),
    });
    let _ = set_platform_event_runtime(Arc::clone(&capture) as Arc<dyn PlatformEventRuntime>);

    let mut registry = FleetAgentRegistry::default();
    register_fleet_agent(
        &mut registry,
        "Rover-A".into(),
        "http://127.0.0.1:9100".into(),
        None,
    )
    .expect("register");
    register_fleet_agent(
        &mut registry,
        "Rover-A".into(),
        "http://127.0.0.1:9101".into(),
        None,
    )
    .expect("replace");
    assert!(deregister_fleet_agent(&mut registry, "Rover-A"));

    let types = capture.types.lock().unwrap();
    assert_eq!(
        types
            .iter()
            .filter(|t| t.as_str() == names::FLEET_MEMBER_LEFT)
            .count(),
        2
    );
}
