//! Security platform event emission tests.

use spanda_audit::platform_event::names;
use spanda_runtime::platform_event_runtime::{
    set_platform_event_runtime, PlatformEventRuntime,
};
use spanda_security::{record_auth_failed, ManagedSecretVault, SecretHandle, SecretMetadata};
use spanda_security::SecretSource;
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
fn security_platform_events_emit_auth_and_secret_rotation() {
    let capture = Arc::new(CapturePlatformEvents {
        types: Mutex::new(Vec::new()),
    });
    let _ = set_platform_event_runtime(Arc::clone(&capture) as Arc<dyn PlatformEventRuntime>);

    record_auth_failed("missing_or_invalid_token", None);

    let mut vault = ManagedSecretVault::new();
    vault.register(
        SecretHandle {
            name: "robot_cred".into(),
            source: SecretSource::Literal {
                value: "secret-value".into(),
            },
        },
        SecretMetadata::new("robot_cred", 1.0),
    );
    vault.rotate_literal("robot_cred", "rotated-value").expect("rotate");

    let types = capture.types.lock().unwrap();
    assert!(types.iter().any(|t| t == names::AUTH_FAILED));
    assert!(types.iter().any(|t| t == names::SECRET_ROTATED));
}
