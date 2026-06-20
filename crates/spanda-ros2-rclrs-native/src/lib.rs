//! In-process ROS 2 publish/subscribe via [rclrs](https://docs.rs/rclrs).
//!
//! Uses dynamic message introspection so builds do not depend on the yanked
//! `std_msgs` crates.io package. A sourced ROS 2 install is still required at
//! runtime (`AMENT_PREFIX_PATH`).

use rclrs::dynamic_message::{SimpleValueMut, ValueMut};
use rclrs::{Context, DynamicMessage, MessageTypeName, RclrsError, SpinOptions};
use std::ffi::CStr;
use std::os::raw::c_char;
use std::process::Command;
use std::time::Duration;

const STRING_MSG: &str = "std_msgs/msg/String";

fn string_message_type() -> Option<MessageTypeName> {
    MessageTypeName::try_from(STRING_MSG).ok()
}

fn set_string_payload(message: &mut DynamicMessage, payload: &str) -> bool {
    let Some(ValueMut::Simple(SimpleValueMut::BoundedString(field))) = message.get_mut("data") else {
        return false;
    };
    field.try_assign(payload).is_ok()
}

pub fn sdk_available() -> bool {
    Context::default_from_env().is_ok()
}

pub fn init_node(name: &str) -> Result<(), String> {
    let context = Context::default_from_env().map_err(|e: RclrsError| e.to_string())?;
    let mut executor = context.create_basic_executor();
    executor
        .create_node(name)
        .map(|_| ())
        .map_err(|e: RclrsError| e.to_string())
}

pub fn publish(topic: &str, payload: &str) -> bool {
    let Some(type_name) = string_message_type() else {
        return false;
    };
    let Ok(context) = Context::default_from_env() else {
        return false;
    };
    let mut executor = context.create_basic_executor();
    let Ok(node) = executor.create_node("spanda_rclrs") else {
        return false;
    };
    let Ok(publisher) = node.create_dynamic_publisher(type_name.clone(), topic) else {
        return false;
    };
    let Ok(mut message) = DynamicMessage::new(type_name) else {
        return false;
    };
    if !set_string_payload(&mut message, payload) {
        return false;
    }
    publisher.publish(message).is_ok()
}

pub fn subscribe(topic: &str) -> bool {
    let Some(type_name) = string_message_type() else {
        return false;
    };
    let Ok(context) = Context::default_from_env() else {
        return false;
    };
    let mut executor = context.create_basic_executor();
    let Ok(node) = executor.create_node("spanda_rclrs") else {
        return false;
    };
    let Ok(_subscription) = node.create_dynamic_subscription(
        type_name,
        topic,
        |_msg: DynamicMessage, _info: rclrs::MessageInfo| {},
    ) else {
        return false;
    };
    executor
        .spin(SpinOptions::default().timeout(Duration::from_millis(50)))
        .is_empty()
}

pub fn service_call(service: &str, service_type: &str, request: &str) -> bool {
    Command::new("ros2")
        .args(["service", "call", service, service_type, request])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

#[no_mangle]
pub unsafe extern "C" fn spanda_ros2_rclrs_sdk_available() -> bool {
    sdk_available()
}

#[no_mangle]
pub unsafe extern "C" fn spanda_ros2_rclrs_publish(
    topic: *const c_char,
    payload: *const c_char,
) -> bool {
    let Some(topic) = (!topic.is_null()).then(|| CStr::from_ptr(topic).to_string_lossy()) else {
        return false;
    };
    let Some(payload) = (!payload.is_null()).then(|| CStr::from_ptr(payload).to_string_lossy()) else {
        return false;
    };
    publish(&topic, &payload)
}

#[no_mangle]
pub unsafe extern "C" fn spanda_ros2_rclrs_subscribe(topic: *const c_char) -> bool {
    let Some(topic) = (!topic.is_null()).then(|| CStr::from_ptr(topic).to_string_lossy()) else {
        return false;
    };
    subscribe(&topic)
}

#[no_mangle]
pub unsafe extern "C" fn spanda_ros2_rclrs_service_call(
    service: *const c_char,
    service_type: *const c_char,
    request: *const c_char,
) -> bool {
    let Some(service) = (!service.is_null()).then(|| CStr::from_ptr(service).to_string_lossy()) else {
        return false;
    };
    let Some(service_type) = (!service_type.is_null())
        .then(|| CStr::from_ptr(service_type).to_string_lossy())
    else {
        return false;
    };
    let Some(request) = (!request.is_null()).then(|| CStr::from_ptr(request).to_string_lossy()) else {
        return false;
    };
    service_call(&service, &service_type, &request)
}

#[no_mangle]
pub unsafe extern "C" fn spanda_ros2_rclrs_init_node(name: *const c_char) -> i32 {
    let Some(name) = (!name.is_null()).then(|| CStr::from_ptr(name).to_string_lossy()) else {
        return 1;
    };
    match init_node(&name) {
        Ok(()) => 0,
        Err(_) => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sdk_probe_does_not_panic_without_ros() {
        let _ = sdk_available();
    }

    #[test]
    fn string_message_type_parses() {
        assert!(string_message_type().is_some());
    }
}
