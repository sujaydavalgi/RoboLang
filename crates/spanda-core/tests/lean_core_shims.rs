//! Guardrails for lean-core shim deprecation in `spanda-core`.
//!
use std::fs;
use std::path::Path;

#[test]
fn transport_live_shim_stays_thin() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/transport_live.rs");
    let source = fs::read_to_string(&path).expect("transport_live.rs");
    let lines = source.lines().count();
    assert!(
        lines <= 80,
        "transport_live.rs should remain a thin shim (got {lines} lines); move logic to spanda-transport-*"
    );
    assert!(
        source.contains("spanda_transport_ros2::live_bridge"),
        "transport_live shim should delegate ROS2 live hooks to spanda-transport-ros2"
    );
    assert!(
        source.contains("spanda_transport_mqtt"),
        "transport_live shim should delegate MQTT live hooks to spanda-transport-mqtt"
    );
}

#[test]
fn transport_live_no_direct_python_bridge() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/transport_live.rs");
    let source = fs::read_to_string(&path).expect("transport_live.rs");
    assert!(
        !source.contains("call_subprocess_bridge"),
        "transport_live should not invoke the Python bridge directly"
    );
    assert!(
        !source.contains("bridge_script_path"),
        "transport_live should not resolve bridge script paths directly"
    );
}
