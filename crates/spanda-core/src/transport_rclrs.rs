//! Compatibility shim: ROS2 transport orchestration (delegates to `spanda-transport-ros2`).
//!
use crate::runtime::RuntimeValue;
use crate::transport_rclrs_daemon::{daemon_publish, daemon_service_call, daemon_subscribe};
use crate::transport_rclrs_native as native;
use spanda_transport_ros2::live_bridge::{
    try_ros2_bridge_publish, try_ros2_bridge_service_call, try_ros2_bridge_subscribe,
};

pub use spanda_transport_ros2::{rclrs_available, rclrs_enabled};

fn payload_string(value: &RuntimeValue) -> String {
    match value {
        RuntimeValue::String { value } => value.clone(),
        RuntimeValue::Number { value, .. } => value.to_string(),
        RuntimeValue::Bool { value } => value.to_string(),
        other => format!("{other:?}"),
    }
}

pub fn try_rclrs_publish(topic: &str, value: &RuntimeValue) -> bool {
    if !rclrs_enabled() {
        return false;
    }
    if native::publish(topic, &payload_string(value)) {
        return true;
    }
    if daemon_publish(topic, value) {
        return true;
    }
    try_ros2_bridge_publish(topic, &payload_string(value))
}

pub fn try_rclrs_subscribe(topic: &str) -> bool {
    if !rclrs_enabled() {
        return false;
    }
    if native::subscribe(topic) {
        return true;
    }
    if daemon_subscribe(topic) {
        return true;
    }
    try_ros2_bridge_subscribe(topic)
}

pub fn try_rclrs_service_call(service: &str, service_type: &str, request: &str) -> bool {
    if !rclrs_enabled() {
        return false;
    }
    if native::service_call(service, service_type, request) {
        return true;
    }
    if daemon_service_call(service, service_type, request) {
        return true;
    }
    try_ros2_bridge_service_call(service, service_type, request)
}

pub fn init_node(name: &str) -> Result<(), String> {
    spanda_transport_ros2::init_node(name)
}

pub fn native_sdk_available() -> bool {
    native::sdk_available()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rclrs_off_by_default() {
        std::env::remove_var("SPANDA_ROS2_RCLRS");
        assert!(!rclrs_enabled());
    }
}
