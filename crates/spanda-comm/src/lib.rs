//! Spanda communication bus trait, in-memory transport, and comm safety helpers.
//!

pub mod comm_bus_host;

pub use comm_bus_host::{
    default_comm_bus_factory, default_comm_bus_factory_fn, CommBusFactory, CommBusHost,
    SimCommBusHost,
};

pub use spanda_ast::comm_decl::*;

use spanda_ast::nodes::SpandaType;
use spanda_runtime::value::RuntimeValue;
use std::collections::{HashMap, VecDeque};

pub use spanda_typecheck::{is_comm_capability, MessageRegistry, COMM_CAPABILITIES};

// ── CommBus trait (transport abstraction) ────────────────────────────────────

#[derive(Debug, Clone)]
pub struct CommEnvelope {
    pub value: RuntimeValue,
    pub source_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PublishedCommMessage {
    pub topic_path: String,
    pub message_type: String,
    pub value: RuntimeValue,
    pub transport: TransportKind,
    pub source_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SimNetworkConfig {
    pub delay_ms: f64,
    pub packet_loss: f64,
}

impl Default for SimNetworkConfig {
    fn default() -> Self {
        // Description:
        //     Provide the default value for this type.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     result: Self
        //         Return value from `default`.
        //
        // Example:
        //     let result = spanda_comm::default();

        // Assemble the struct fields and return it.
        Self {
            delay_ms: 0.0,
            packet_loss: 0.0,
        }
    }
}

pub trait CommBus {
    fn publish(
        &mut self,
        topic_path: &str,
        message_type: &str,
        value: RuntimeValue,
        transport: TransportKind,
        source_id: Option<&str>,
    );
    fn subscribe(&mut self, topic_path: &str, handler: &str);
    fn receive(&mut self, topic_path: &str) -> Option<RuntimeValue>;
    fn receive_envelope(&mut self, topic_path: &str) -> Option<CommEnvelope>;
    fn call_service(
        &mut self,
        service_name: &str,
        service_type: &str,
        request: Option<RuntimeValue>,
    ) -> RuntimeValue;
    fn send_action(
        &mut self,
        action_name: &str,
        action_type: &str,
        goal: RuntimeValue,
    ) -> RuntimeValue;
    fn discover(&self, target: DiscoverTarget, filter: &DiscoverFilter) -> Vec<String>;
    fn published_messages(&self) -> Vec<PublishedCommMessage>;
    fn inject_fault(&mut self, fault: &str);
    fn set_network_config(&mut self, config: SimNetworkConfig);
    fn active_faults(&self) -> Vec<String>;
    fn subscription_paths(&self) -> Vec<String>;
    fn push_inbound(&mut self, topic_path: &str, value: RuntimeValue, source_id: Option<&str>);
}

/// In-memory pub/sub bus for simulation and tests.
#[derive(Debug, Clone, Default)]
pub struct InMemoryCommBus {
    subscriptions: HashMap<String, Vec<String>>,
    buffers: HashMap<String, VecDeque<CommEnvelope>>,
    published: Vec<PublishedCommMessage>,
    discovered_robots: Vec<String>,
    discovered_agents: Vec<String>,
    discovered_devices: Vec<String>,
    network: SimNetworkConfig,
    faults: Vec<String>,
}

impl InMemoryCommBus {
    pub fn new() -> Self {
        // Description:
        //     Construct a new instance.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     result: Self
        //         Return value from `new`.
        //
        // Example:
        //     let value = spanda_comm::new();

        // Assemble the struct fields and return it.
        Self {
            discovered_robots: vec!["RoverA".into(), "RoverB".into()],
            discovered_agents: vec!["Vision".into(), "Planner".into(), "Navigator".into()],
            discovered_devices: vec!["Camera".into(), "IMU".into(), "Lidar".into()],
            ..Default::default()
        }
    }

    pub fn register_robot(&mut self, name: impl Into<String>) {
        // Description:
        //     Register robot.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     name: impl Into<String>
        //         Caller-supplied name.
        //
        // Outputs:
        //     None.
        //
        // Example:
        //     let result = spanda_comm::register_robot(&mut self, name);

        // Append into self.
        self.discovered_robots.push(name.into());
    }

    pub fn register_agent(&mut self, name: impl Into<String>) {
        // Description:
        //     Register agent.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     name: impl Into<String>
        //         Caller-supplied name.
        //
        // Outputs:
        //     None.
        //
        // Example:
        //     let result = spanda_comm::register_agent(&mut self, name);

        // Append into self.
        self.discovered_agents.push(name.into());
    }

    pub fn register_device(&mut self, name: impl Into<String>) {
        // Description:
        //     Register device.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     name: impl Into<String>
        //         Caller-supplied name.
        //
        // Outputs:
        //     None.
        //
        // Example:
        //     let result = spanda_comm::register_device(&mut self, name);

        // Append into self.
        self.discovered_devices.push(name.into());
    }

    pub fn active_faults(&self) -> Vec<String> {
        // Description:
        //     Active faults.
        //
        // Inputs:
        //     &self: input value
        //         Caller-supplied &self.
        //
        // Outputs:
        //     result: Vec<String>
        //         Return value from `active_faults`.
        //
        // Example:
        //     let result = spanda_comm::active_faults(&self);

        // Call clone on the current instance.
        self.faults.clone()
    }

    pub fn subscription_paths(&self) -> Vec<String> {
        // Description:
        //     Subscription paths.
        //
        // Inputs:
        //     &self: input value
        //         Caller-supplied &self.
        //
        // Outputs:
        //     result: Vec<String>
        //         Return value from `subscription_paths`.
        //
        // Example:
        //     let result = spanda_comm::subscription_paths(&self);

        // Collect filtered entries into a new list.
        self.subscriptions.keys().cloned().collect()
    }

    pub fn push_inbound(&mut self, topic_path: &str, value: RuntimeValue, source_id: Option<&str>) {
        // Description:
        //     Push inbound.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     opic_path: &str
        //         Caller-supplied opic path.
        //     value: RuntimeValue
        //         Caller-supplied value.
        //     source_id: Option<&str>
        //         Caller-supplied source id.
        //
        // Outputs:
        //     None.
        //
        // Example:
        //     let result = spanda_comm::push_inbound(&mut self, opic_path, value, source_id);

        // Queue the inbound envelope for subscribers.
        self.buffers
            .entry(topic_path.to_string())
            .or_default()
            .push_back(CommEnvelope {
                value,
                source_id: source_id.map(str::to_string),
            });
    }

    /// Deliver a message to a peer robot topic namespace (`/{peer}/{topic}`).
    pub fn publish_peer(
        &mut self,
        peer: &str,
        topic: &str,
        value: RuntimeValue,
        transport: TransportKind,
        source_id: Option<&str>,
    ) {
        // Description:
        //     Publish peer.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     peer: &str
        //         Caller-supplied peer.
        //     opic: &str
        //         Caller-supplied opic.
        //     value: RuntimeValue
        //         Caller-supplied value.
        //     ranspor: TransportKind
        //         Caller-supplied ranspor.
        //     source_id: Option<&str>
        //         Caller-supplied source id.
        //
        // Outputs:
        //     None.
        //
        // Example:
        //     let result = spanda_comm::publish_peer(&mut self, peer, opic, value, ranspor, source_id);

        // Resolve the filesystem path for the next step.
        let path = format!("/{peer}/{topic}");
        self.publish(
            &path,
            "PeerMessage",
            value,
            transport,
            source_id.or(Some(peer)),
        );
    }
}

impl CommBus for InMemoryCommBus {
    fn publish(
        &mut self,
        topic_path: &str,
        message_type: &str,
        value: RuntimeValue,
        transport: TransportKind,
        source_id: Option<&str>,
    ) {
        // Description:
        //     Publish.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     opic_path: &str
        //         Caller-supplied opic path.
        //     essage_type: &str
        //         Caller-supplied essage type.
        //     value: RuntimeValue
        //         Caller-supplied value.
        //     ranspor: TransportKind
        //         Caller-supplied ranspor.
        //     source_id: Option<&str>
        //         Caller-supplied source id.
        //
        // Outputs:
        //     None.
        //
        // Example:
        //     let result = spanda_comm::publish(&mut self, opic_path, essage_type, value, ranspor, source_id);

        // take the branch when any equals "NetworkOutage").
        if self.faults.iter().any(|f| f == "NetworkOutage") {
            return;
        }

        // Take this path when self.network.packet loss > 0.0.
        if self.network.packet_loss > 0.0 {
            let hash = topic_path.len() + message_type.len();

            // Take this path when (hash as f64 * 0.13).fract() < self.network.packet loss.
            if (hash as f64 * 0.13).fract() < self.network.packet_loss {
                return;
            }
        }
        self.published.push(PublishedCommMessage {
            topic_path: topic_path.to_string(),
            message_type: message_type.to_string(),
            value: value.clone(),
            transport,
            source_id: source_id.map(str::to_string),
        });

        // Emit output when get mut provides a buf.
        if let Some(buf) = self.buffers.get_mut(topic_path) {
            buf.push_back(CommEnvelope {
                value,
                source_id: source_id.map(str::to_string),
            });
        }
    }

    fn subscribe(&mut self, topic_path: &str, handler: &str) {
        // Description:
        //     Subscribe.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     opic_path: &str
        //         Caller-supplied opic path.
        //     handler: &str
        //         Caller-supplied handler.
        //
        // Outputs:
        //     None.
        //
        // Example:
        //     let result = spanda_comm::subscribe(&mut self, opic_path, handler);

        // Call subscriptions on the current instance.
        self.subscriptions
            .entry(topic_path.to_string())
            .or_default()
            .push(handler.to_string());
        self.buffers.entry(topic_path.to_string()).or_default();
    }

    fn receive(&mut self, topic_path: &str) -> Option<RuntimeValue> {
        // Description:
        //     Receive.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     opic_path: &str
        //         Caller-supplied opic path.
        //
        // Outputs:
        //     result: Option<RuntimeValue>
        //         Return value from `receive`.
        //
        // Example:
        //     let result = spanda_comm::receive(&mut self, opic_path);

        // Transform self and continue the chain.
        self.buffers
            .get_mut(topic_path)
            .and_then(|q| q.pop_front())
            .map(|env| env.value)
    }

    fn receive_envelope(&mut self, topic_path: &str) -> Option<CommEnvelope> {
        // Description:
        //     Receive envelope.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     opic_path: &str
        //         Caller-supplied opic path.
        //
        // Outputs:
        //     result: Option<CommEnvelope>
        //         Return value from `receive_envelope`.
        //
        // Example:
        //     let result = spanda_comm::receive_envelope(&mut self, opic_path);

        // Pop the next queued envelope for this topic path.
        self.buffers.get_mut(topic_path).and_then(|q| q.pop_front())
    }

    fn call_service(
        &mut self,
        _service_name: &str,
        service_type: &str,
        _request: Option<RuntimeValue>,
    ) -> RuntimeValue {
        // Description:
        //     Call service.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     _service_name: &str
        //         Caller-supplied service name.
        //     service_type: &str
        //         Caller-supplied service type.
        //     request: Option<RuntimeValue>
        //         Caller-supplied request.
        //
        // Outputs:
        //     result: RuntimeValue
        //         Return value from `call_service`.
        //
        // Example:
        //     let result = spanda_comm::call_service(&mut self, _service_name, service_type, _reques);

        // Build a Object runtime value.
        RuntimeValue::Object {
            type_name: service_type.to_string(),
            fields: HashMap::from([("ok".into(), RuntimeValue::Bool { value: true })]),
        }
    }

    fn send_action(
        &mut self,
        _action_name: &str,
        action_type: &str,
        _goal: RuntimeValue,
    ) -> RuntimeValue {
        // Description:
        //     Send action.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     _action_name: &str
        //         Caller-supplied action name.
        //     action_type: &str
        //         Caller-supplied action type.
        //     _goal: RuntimeValue
        //         Caller-supplied goal.
        //
        // Outputs:
        //     result: RuntimeValue
        //         Return value from `send_action`.
        //
        // Example:
        //     let result = spanda_comm::send_action(&mut self, _action_name, action_type, _goal);

        // Build a Object runtime value.
        RuntimeValue::Object {
            type_name: action_type.to_string(),
            fields: HashMap::from([("success".into(), RuntimeValue::Bool { value: true })]),
        }
    }

    fn discover(&self, target: DiscoverTarget, filter: &DiscoverFilter) -> Vec<String> {
        // Description:
        //     Discover.
        //
        // Inputs:
        //     &self: input value
        //         Caller-supplied &self.
        //     arge: DiscoverTarget
        //         Caller-supplied arge.
        //     filter: &DiscoverFilter
        //         Caller-supplied filter.
        //
        // Outputs:
        //     result: Vec<String>
        //         Return value from `discover`.
        //
        // Example:
        //     let result = spanda_comm::discover(&self, arge, filter);

        // Compute base for the following logic.
        let base = match target {
            DiscoverTarget::Robots => self.discovered_robots.clone(),
            DiscoverTarget::Agents => self.discovered_agents.clone(),
            DiscoverTarget::Devices => self.discovered_devices.clone(),
        };

        // Emit output when capability provides a cap.
        if let Some(cap) = &filter.capability {
            base.into_iter()
                .filter(|n| n.to_lowercase().contains(&cap.to_lowercase()))
                .collect()
        } else {
            base
        }
    }

    fn published_messages(&self) -> Vec<PublishedCommMessage> {
        // Description:
        //     Published messages.
        //
        // Inputs:
        //     &self: input value
        //         Caller-supplied &self.
        //
        // Outputs:
        //     result: Vec<PublishedCommMessage>
        //         Return value from `published_messages`.
        //
        // Example:
        //     let result = spanda_comm::published_messages(&self);

        // Call clone on the current instance.
        self.published.clone()
    }

    fn inject_fault(&mut self, fault: &str) {
        // Description:
        //     Inject fault.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     faul: &str
        //         Caller-supplied faul.
        //
        // Outputs:
        //     None.
        //
        // Example:
        //     let result = spanda_comm::inject_fault(&mut self, faul);

        // Append into self.
        self.faults.push(fault.to_string());
    }

    fn set_network_config(&mut self, config: SimNetworkConfig) {
        // Description:
        //     Set network config.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     config: SimNetworkConfig
        //         Caller-supplied config.
        //
        // Outputs:
        //     None.
        //
        // Example:
        //     let result = spanda_comm::set_network_config(&mut self, config);

        // Call network = config; on the current instance.
        self.network = config;
    }

    fn active_faults(&self) -> Vec<String> {
        // Description:
        //     Active faults.
        //
        // Inputs:
        //     &self: input value
        //         Caller-supplied &self.
        //
        // Outputs:
        //     result: Vec<String>
        //         Return value from `active_faults`.
        //
        // Example:
        //     let result = spanda_comm::active_faults(&self);

        // Call clone on the current instance.
        self.faults.clone()
    }

    fn subscription_paths(&self) -> Vec<String> {
        // Description:
        //     Subscription paths.
        //
        // Inputs:
        //     &self: input value
        //         Caller-supplied &self.
        //
        // Outputs:
        //     result: Vec<String>
        //         Return value from `subscription_paths`.
        //
        // Example:
        //     let result = spanda_comm::subscription_paths(&self);

        // Collect filtered entries into a new list.
        self.subscriptions.keys().cloned().collect()
    }

    fn push_inbound(&mut self, topic_path: &str, value: RuntimeValue, source_id: Option<&str>) {
        // Description:
        //     Push inbound.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     opic_path: &str
        //         Caller-supplied opic path.
        //     value: RuntimeValue
        //         Caller-supplied value.
        //     source_id: Option<&str>
        //         Caller-supplied source id.
        //
        // Outputs:
        //     None.
        //
        // Example:
        //     let result = spanda_comm::push_inbound(&mut self, opic_path, value, source_id);

        // Queue the inbound envelope for subscribers.
        self.buffers
            .entry(topic_path.to_string())
            .or_default()
            .push_back(CommEnvelope {
                value,
                source_id: source_id.map(str::to_string),
            });
    }
}

// ── Safety communication wrappers ────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum CommSafetyStage {
    ActionProposal,
    SafeAction,
    CommandMessage,
    Actuator,
}

pub fn validate_comm_safety_chain(
    stage: CommSafetyStage,
    value: &RuntimeValue,
) -> Result<(), String> {
    // Description:
    //     Validate comm safety chain.
    //
    // Inputs:
    //     stage: CommSafetyStage
    //         Caller-supplied stage.
    //     value: &RuntimeValue
    //         Caller-supplied value.
    //
    // Outputs:
    //     result: Result<(), String>
    //         Return value from `validate_comm_safety_chain`.
    //
    // Example:
    //     let result = spanda_comm::validate_comm_safety_chain(stage, value);

    // Match on stage and handle each case.
    match stage {
        CommSafetyStage::ActionProposal => {
            // Keep entries that match the expected pattern.
            if !matches!(value, RuntimeValue::Object { type_name, .. } if type_name == "ActionProposal")
            {
                return Err("Expected ActionProposal before safety validation".into());
            }
        }
        CommSafetyStage::SafeAction => {
            // Keep entries that match the expected pattern.
            if !matches!(value, RuntimeValue::Object { type_name, .. } if type_name == "SafeAction")
            {
                return Err("Expected SafeAction before command conversion".into());
            }
        }
        CommSafetyStage::CommandMessage => {
            // Keep entries that match the expected pattern.
            if !matches!(value, RuntimeValue::Object { type_name, .. } if type_name == "CommandMessage")
            {
                return Err("Expected CommandMessage before actuator dispatch".into());
            }
        }
        CommSafetyStage::Actuator => {}
    }
    Ok(())
}

// ── Network bandwidth estimation from QoS ────────────────────────────────────

pub fn estimate_topic_bandwidth_mbps(rate_hz: f64, message_size_bytes: f64) -> f64 {
    // Description:
    //     Estimate topic bandwidth mbps.
    //
    // Inputs:
    //     rate_hz: f64
    //         Caller-supplied rate hz.
    //     essage_size_bytes: f64
    //         Caller-supplied essage size bytes.
    //
    // Outputs:
    //     result: f64
    //         Return value from `estimate_topic_bandwidth_mbps`.
    //
    // Example:
    //     let result = spanda_comm::estimate_topic_bandwidth_mbps(rate_hz, essage_size_bytes);

    // Produce 0 as the result.
    (rate_hz * message_size_bytes * 8.0) / 1_000_000.0
}

pub fn default_message_size(message_type: &str) -> f64 {
    // Description:
    //     Default message size.
    //
    // Inputs:
    //     essage_type: &str
    //         Caller-supplied essage type.
    //
    // Outputs:
    //     result: f64
    //         Return value from `default_message_size`.
    //
    // Example:
    //     let result = spanda_comm::default_message_size(essage_type);

    // Match on message type and handle each case.
    match message_type {
        "Scan" | "LidarScan" | "LidarReading" => 64_000.0,
        "Pose" | "Velocity" => 128.0,
        "PathPlan" | "NavigationFeedback" => 4_096.0,
        _ => 512.0,
    }
}

pub fn qos_to_spanda_type(qos: &QosDecl) -> SpandaType {
    // Description:
    //     Qos to spanda type.
    //
    // Inputs:
    //     qos: &QosDecl
    //         Caller-supplied qos.
    //
    // Outputs:
    //     result: SpandaType
    //         Return value from `qos_to_spanda_type`.
    //
    // Example:
    //     let result = spanda_comm::qos_to_spanda_type(qos);

    // Compute value for the following logic.
    let _ = qos;
    SpandaType::Named { name: "QoS".into() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spanda_ast::foundations::FieldDecl;
    use spanda_ast::nodes::Span;

    #[test]
    fn message_registry_builtin_and_custom() {
        // Description:
        //     Message registry builtin and custom.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_comm::message_registry_builtin_and_custom();

        let mut reg = MessageRegistry::new();
        assert!(reg.is_known("Velocity"));
        let decl = MessageDecl::MessageDecl {
            name: "LidarReading".into(),
            fields: vec![FieldDecl {
                name: "scan".into(),
                type_name: "LidarScan".into(),
                span: Span {
                    start: spanda_ast::nodes::SourceLocation {
                        line: 1,
                        column: 1,
                        offset: 0,
                    },
                    end: spanda_ast::nodes::SourceLocation {
                        line: 1,
                        column: 1,
                        offset: 0,
                    },
                },
            }],
            version: Some(1),
            span: Span {
                start: spanda_ast::nodes::SourceLocation {
                    line: 1,
                    column: 1,
                    offset: 0,
                },
                end: spanda_ast::nodes::SourceLocation {
                    line: 1,
                    column: 1,
                    offset: 0,
                },
            },
        };
        reg.register(&decl);
        assert!(reg.is_known("LidarReading"));
    }

    #[test]
    fn in_memory_bus_pub_sub() {
        // Description:
        //     In memory bus pub sub.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_comm::in_memory_bus_pub_sub();

        let mut bus = InMemoryCommBus::new();
        bus.subscribe("/scan", "handler");
        bus.publish(
            "/scan",
            "Scan",
            RuntimeValue::Scan {
                nearest_distance: 1.5,
            },
            TransportKind::Sim,
            None,
        );
        let msg = bus.receive("/scan");
        assert!(msg.is_some());
        assert_eq!(bus.published_messages().len(), 1);
    }

    #[test]
    fn discover_robots_with_capability() {
        // Description:
        //     Discover robots with capability.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_comm::discover_robots_with_capability();

        let bus = InMemoryCommBus::new();
        let results = bus.discover(
            DiscoverTarget::Robots,
            &DiscoverFilter {
                capability: Some("Rover".into()),
            },
        );
        assert!(!results.is_empty());
    }

    #[test]
    fn bandwidth_estimate() {
        // Description:
        //     Bandwidth estimate.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_comm::bandwidth_estimate();

        let mbps = estimate_topic_bandwidth_mbps(20.0, 64000.0);
        assert!(mbps > 10.0);
    }
}
