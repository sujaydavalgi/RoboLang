//! Transport adapter trait, configuration, and shared stub state.

use crate::security::{TlsTransportSession, TransportSecurityConfig};
use spanda_ast::comm_decl::TransportKind;
use spanda_runtime::RuntimeValue;
use std::collections::{HashMap, VecDeque};

/// Serialize a runtime value into a ROS2-style service request payload string.
pub fn payload_string_for_service(value: &RuntimeValue) -> String {
    // Payload string for service.
    //
    // Parameters:
    // - `value` — input value
    //
    // Returns:
    // Text result.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = payload_string_for_service(value);

    // Match on value and handle each case.
    match value {
        RuntimeValue::String { value } => {
            format!(
                "{{data: \"{}\"}}",
                value.replace('\\', "\\\\").replace('"', "\\\"")
            )
        }
        RuntimeValue::Number { value, .. } => format!("{{value: {value}}}"),
        RuntimeValue::Bool { value } => format!("{{ok: {value}}}"),
        other => format!("{{raw: \"{other:?}\"}}"),
    }
}

/// Connection and security settings shared by all transport adapters.
#[derive(Debug, Clone, Default)]
pub struct TransportConfig {
    pub broker_url: Option<String>,
    pub node_name: Option<String>,
    pub namespace: Option<String>,
    pub domain_id: Option<u32>,
    pub client_id: Option<String>,
    pub security: TransportSecurityConfig,
    pub tls: TlsTransportSession,
}

/// One published message recorded by a transport adapter stub.
#[derive(Debug, Clone)]
pub struct AdapterMessage {
    pub topic: String,
    pub message_type: String,
    pub value: RuntimeValue,
}

/// Pluggable backend for ROS2, MQTT, DDS, and WebSocket transports.
pub trait TransportAdapter {
    fn kind(&self) -> TransportKind;
    fn connect(&mut self, config: &TransportConfig) -> Result<(), String>;
    fn disconnect(&mut self);
    fn is_connected(&self) -> bool;
    fn publish(&mut self, topic: &str, message_type: &str, value: RuntimeValue);
    fn subscribe(&mut self, topic: &str);
    fn receive(&mut self, topic: &str) -> Option<RuntimeValue>;
    fn call_service(
        &mut self,
        service: &str,
        service_type: &str,
        request: Option<RuntimeValue>,
    ) -> RuntimeValue;
    fn send_action(&mut self, action: &str, action_type: &str, goal: RuntimeValue) -> RuntimeValue;
    fn published(&self) -> Vec<AdapterMessage>;
}

/// In-memory publish/subscribe state used by stub and live adapter wrappers.
#[derive(Debug, Default)]
pub struct StubTransportState {
    pub connected: bool,
    pub config: TransportConfig,
    subscriptions: HashMap<String, VecDeque<RuntimeValue>>,
    pub published: Vec<AdapterMessage>,
}

impl StubTransportState {
    pub fn publish(&mut self, topic: &str, message_type: &str, value: RuntimeValue) {
        // Publish.
        //
        // Parameters:
        // - `self` — method receiver
        // - `topic` — input value
        // - `message_type` — input value
        // - `value` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.publish(topic, message_type, value);

        // Append into self.
        self.published.push(AdapterMessage {
            topic: topic.to_string(),
            message_type: message_type.to_string(),
            value: value.clone(),
        });

        // Emit output when get mut provides a buf.
        if let Some(buf) = self.subscriptions.get_mut(topic) {
            buf.push_back(value);
        }
    }

    pub fn subscribe(&mut self, topic: &str) {
        // Subscribe.
        //
        // Parameters:
        // - `self` — method receiver
        // - `topic` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.subscribe(topic);

        // Call entry on the current instance.
        self.subscriptions.entry(topic.to_string()).or_default();
    }

    pub fn receive(&mut self, topic: &str) -> Option<RuntimeValue> {
        // Receive.
        //
        // Parameters:
        // - `self` — method receiver
        // - `topic` — input value
        //
        // Returns:
        // Some value on success, otherwise none.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.receive(topic);

        // Call subscriptions on the current instance.
        self.subscriptions
            .get_mut(topic)
            .and_then(|q| q.pop_front())
    }

    pub fn service_result(service_type: &str) -> RuntimeValue {
        // Service result.
        //
        // Parameters:
        // - `service_type` — input value
        //
        // Returns:
        // RuntimeValue.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = StubTransportState::service_result(service_type);

        // Build a Object runtime value.
        RuntimeValue::Object {
            type_name: service_type.to_string(),
            fields: HashMap::from([("ok".into(), RuntimeValue::Bool { value: true })]),
        }
    }

    pub fn action_result(action_type: &str) -> RuntimeValue {
        // Action result.
        //
        // Parameters:
        // - `action_type` — input value
        //
        // Returns:
        // RuntimeValue.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = StubTransportState::action_result(action_type);

        // Build a Object runtime value.
        RuntimeValue::Object {
            type_name: action_type.to_string(),
            fields: HashMap::from([("success".into(), RuntimeValue::Bool { value: true })]),
        }
    }
}

/// Generate a stub `TransportAdapter` implementation for simulation backends.
#[macro_export]
macro_rules! stub_adapter {
    ($name:ident, $kind:expr) => {
        #[derive(Debug, Default)]
        pub struct $name {
            state: $crate::StubTransportState,
        }

        impl $crate::TransportAdapter for $name {
            fn kind(&self) -> spanda_ast::comm_decl::TransportKind {
                // Kind.
                //
                // Parameters:
                // - `self` — method receiver
                //
                // Returns:
                // TransportKind.
                //
                // Options:
                // None.
                //
                // Example:
                // let result = instance.kind();

                // Produce $kind as the result.
                $kind
            }

            fn connect(&mut self, config: &$crate::TransportConfig) -> Result<(), String> {
                // Connect.
                //
                // Parameters:
                // - `self` — method receiver
                // - `config` — input value
                //
                // Returns:
                // Success value on completion, or an error.
                //
                // Options:
                // None.
                //
                // Example:
                // let result = instance.connect(config);

                // Call connected = true; on the current instance.
                config.security.validate(self.kind().as_str())?;
                if config.security.encryption != spanda_security::policy::EncryptionMode::None
                    && !config.tls.negotiated
                {
                    return Err(format!(
                        "{} adapter requires negotiated TLS session",
                        self.kind().as_str()
                    ));
                }
                self.state.connected = true;
                self.state.config = config.clone();
                Ok(())
            }

            fn disconnect(&mut self) {
                // Disconnect.
                //
                // Parameters:
                // - `self` — method receiver
                //
                // Returns:
                // Nothing.
                //
                // Options:
                // None.
                //
                // Example:
                // let result = instance.disconnect();

                // Call connected = false; on the current instance.
                self.state.connected = false;
            }

            fn is_connected(&self) -> bool {
                //
                // Parameters:
                // - `self` — method receiver
                //
                // Returns:
                // true or false.
                //
                // Options:
                // None.
                //
                // Example:
                // let result = instance.is_connected();

                // Call connected on the current instance.
                self.state.connected
            }

            fn publish(
                &mut self,
                topic: &str,
                message_type: &str,
                value: spanda_runtime::RuntimeValue,
            ) {
                // Publish.
                //
                // Parameters:
                // - `self` — method receiver
                // - `topic` — input value
                // - `message_type` — input value
                // - `value` — input value
                //
                // Returns:
                // Nothing.
                //
                // Options:
                // None.
                //
                // Example:
                // let result = instance.publish(topic, message_type, value);

                // take this path when self.state.connected.
                if self.state.connected {
                    self.state.publish(topic, message_type, value);
                }
            }

            fn subscribe(&mut self, topic: &str) {
                // Subscribe.
                //
                // Parameters:
                // - `self` — method receiver
                // - `topic` — input value
                //
                // Returns:
                // Nothing.
                //
                // Options:
                // None.
                //
                // Example:
                // let result = instance.subscribe(topic);

                // take this path when self.state.connected.
                if self.state.connected {
                    self.state.subscribe(topic);
                }
            }

            fn receive(&mut self, topic: &str) -> Option<spanda_runtime::RuntimeValue> {
                // Receive.
                //
                // Parameters:
                // - `self` — method receiver
                // - `topic` — input value
                //
                // Returns:
                // Some value on success, otherwise none.
                //
                // Options:
                // None.
                //
                // Example:
                // let result = instance.receive(topic);

                // take this path when self.state.connected.
                if self.state.connected {
                    self.state.receive(topic)
                } else {
                    None
                }
            }

            fn call_service(
                &mut self,
                _service: &str,
                service_type: &str,
                _request: Option<spanda_runtime::RuntimeValue>,
            ) -> spanda_runtime::RuntimeValue {
                // Call service.
                //
                // Parameters:
                // - `self` — method receiver
                // - `_service` — input value
                // - `service_type` — input value
                // - `_request` — input value
                //
                // Returns:
                // RuntimeValue.
                //
                // Options:
                // None.
                //
                // Example:
                // let result = instance.call_service(_service, service_type, _request);

                // Produce service result as the result.
                $crate::StubTransportState::service_result(service_type)
            }

            fn send_action(
                &mut self,
                _action: &str,
                action_type: &str,
                _goal: spanda_runtime::RuntimeValue,
            ) -> spanda_runtime::RuntimeValue {
                // Send action.
                //
                // Parameters:
                // - `self` — method receiver
                // - `_action` — input value
                // - `action_type` — input value
                // - `_goal` — input value
                //
                // Returns:
                // RuntimeValue.
                //
                // Options:
                // None.
                //
                // Example:
                // let result = instance.send_action(_action, action_type, _goal);

                // Produce action result as the result.
                $crate::StubTransportState::action_result(action_type)
            }

            fn published(&self) -> Vec<$crate::AdapterMessage> {
                // Published.
                //
                // Parameters:
                // - `self` — method receiver
                //
                // Returns:
                // Vec<AdapterMessage>.
                //
                // Options:
                // None.
                //
                // Example:
                // let result = instance.published();

                // Call clone on the current instance.
                self.state.published.clone()
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_string_for_string_value() {
        let value = RuntimeValue::String {
            value: "hello".into(),
        };
        let payload = payload_string_for_service(&value);
        assert!(payload.contains("hello"));
    }
}
