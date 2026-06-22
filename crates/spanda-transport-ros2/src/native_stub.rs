//! Stub native rclrs loader for WASM targets.

pub fn sdk_available() -> bool {
    false
}

pub fn init_node(_name: &str) -> Result<(), String> {
    Err("native rclrs is unavailable on wasm32".into())
}

pub fn publish(_topic: &str, _payload: &str) -> bool {
    false
}

pub fn subscribe(_topic: &str) -> bool {
    false
}

pub fn service_call(_service: &str, _service_type: &str, _request: &str) -> bool {
    false
}
