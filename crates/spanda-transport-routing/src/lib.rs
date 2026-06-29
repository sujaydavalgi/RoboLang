//! Routes publish/subscribe/service calls across transport adapters and in-memory simulation.
//!
pub mod live_bridges;
pub mod runtime_bridge;
pub mod transport_live;

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use spanda_comm::{
    CommBus, CommEnvelope, DiscoverFilter, DiscoverTarget, InMemoryCommBus, PublishedCommMessage,
    SimNetworkConfig, TransportKind,
};
use spanda_runtime::providers::transport_types::TransportConfig as RuntimeTransportConfig;
use spanda_runtime::providers::{ProviderRegistry, TransportProvider};
use spanda_runtime::value::RuntimeValue;
use spanda_security::policy::EncryptionMode;
use spanda_transport_dds::DdsTransportAdapter;
use spanda_transport_mqtt::MqttTransportAdapter;
use spanda_transport_ros2::Ros2TransportAdapter;
use spanda_transport_websocket::WebsocketTransportAdapter;

use spanda_transport::adapter::{TransportAdapter, TransportConfig};
use spanda_transport::security::TransportSecurityConfig;
use spanda_transport::wire::{decode_wire_value, encode_wire_value};

fn adapter_config_to_runtime(config: &TransportConfig) -> RuntimeTransportConfig {
    // Description:

    //     Adapter config to runtime.

    //

    // Inputs:

    //     config: &TransportConfig

    //         Caller-supplied config.

    //

    // Outputs:

    //     result: RuntimeTransportConfig

    //         Return value from `adapter_config_to_runtime`.

    //

    // Example:

    //     let result = spanda_transport_routing::adapter_config_to_runtime(config);

    RuntimeTransportConfig {
        broker_url: config.broker_url.clone(),
        node_name: config.node_name.clone(),
        namespace: config.namespace.clone(),
        domain_id: config.domain_id,
        client_id: config.client_id.clone(),
    }
}

pub struct RoutingCommBus {
    memory: InMemoryCommBus,
    ros2: Ros2TransportAdapter,
    mqtt: MqttTransportAdapter,
    dds: DdsTransportAdapter,
    websocket: WebsocketTransportAdapter,
    config: TransportConfig,
    providers: Option<Rc<RefCell<ProviderRegistry>>>,
    registry_keys: HashMap<TransportKind, String>,
    registry_backed: HashSet<TransportKind>,
}

impl std::fmt::Debug for RoutingCommBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RoutingCommBus")
            .field("memory", &self.memory)
            .field("config", &self.config)
            .field("registry_backed", &self.registry_backed)
            .finish_non_exhaustive()
    }
}

impl Default for RoutingCommBus {
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
        //     let result = spanda_transport_routing::default();
        Self::new()
    }
}

impl RoutingCommBus {
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
        //     let value = spanda_transport_routing::new();
        Self {
            memory: InMemoryCommBus::new(),
            ros2: Ros2TransportAdapter::default(),
            mqtt: MqttTransportAdapter::default(),
            dds: DdsTransportAdapter::default(),
            websocket: WebsocketTransportAdapter::default(),
            config: TransportConfig::default(),
            providers: None,
            registry_keys: HashMap::new(),
            registry_backed: HashSet::new(),
        }
    }

    pub fn attach_provider_registry(&mut self, registry: Rc<RefCell<ProviderRegistry>>) {
        // Description:

        //     Attach provider registry.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     registry: Rc<RefCell<ProviderRegistry>>

        //         Caller-supplied registry.

        //

        // Outputs:

        //     None.

        //

        // Example:

        //     let result = spanda_transport_routing::attach_provider_registry(&mut self, registry);

        self.providers = Some(registry);
    }

    pub fn mark_registry_backed(&mut self, kind: TransportKind, key: String) {
        // Description:

        //     Mark registry backed.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     kind: TransportKind

        //         Caller-supplied kind.

        //     key: String

        //         Caller-supplied key.

        //

        // Outputs:

        //     None.

        //

        // Example:

        //     let result = spanda_transport_routing::mark_registry_backed(&mut self, kind, key);

        self.registry_backed.insert(kind);
        self.registry_keys.insert(kind, key);
    }

    pub fn clear_registry_backed(&mut self) {
        // Description:

        //     Clear registry backed.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //

        // Outputs:

        //     None.

        //

        // Example:

        //     let result = spanda_transport_routing::clear_registry_backed(&mut self);

        self.registry_backed.clear();
        self.registry_keys.clear();
    }

    pub fn is_registry_backed(&self, kind: TransportKind) -> bool {
        // Description:

        //     Is registry backed.

        //

        // Inputs:

        //     &self: value

        //         Caller-supplied &self.

        //     kind: TransportKind

        //         Caller-supplied kind.

        //

        // Outputs:

        //     result: bool

        //         Return value from `is_registry_backed`.

        //

        // Example:

        //     let result = spanda_transport_routing::is_registry_backed(&self, kind);

        self.registry_backed.contains(&kind)
    }

    fn uses_registry_transport(&self, kind: TransportKind) -> bool {
        // Description:

        //     Uses registry transport.

        //

        // Inputs:

        //     &self: value

        //         Caller-supplied &self.

        //     kind: TransportKind

        //         Caller-supplied kind.

        //

        // Outputs:

        //     result: bool

        //         Return value from `uses_registry_transport`.

        //

        // Example:

        //     let result = spanda_transport_routing::uses_registry_transport(&self, kind);

        self.registry_backed.contains(&kind) && self.providers.is_some()
    }

    fn with_registry_transport<F, R>(&self, kind: TransportKind, f: F) -> Option<R>
    where
        F: FnOnce(&mut dyn TransportProvider) -> R,
    {
        // Description:

        //     With registry transport.

        //

        // Inputs:

        //     &self: value

        //         Caller-supplied &self.

        //     kind: TransportKind

        //         Caller-supplied kind.

        //     f: F

        //         Caller-supplied f.

        //

        // Outputs:

        //     result: Option<R> where F: FnOnce(&mut dyn TransportProvider) -> R,

        //         Return value from `with_registry_transport`.

        //

        // Example:

        //     let result = spanda_transport_routing::with_registry_transport(&self, kind, f);

        if !self.uses_registry_transport(kind) {
            return None;
        }
        let key = self.registry_keys.get(&kind)?.clone();
        let providers = self.providers.as_ref()?;
        providers.borrow_mut().with_transport(&key, f)
    }

    fn publish_external(
        &mut self,
        kind: TransportKind,
        topic_path: &str,
        message_type: &str,
        value: RuntimeValue,
    ) {
        // Description:

        //     Publish external.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     kind: TransportKind

        //         Caller-supplied kind.

        //     opic_path: &str

        //         Caller-supplied opic path.

        //     essage_type: &str

        //         Caller-supplied essage type.

        //     value: RuntimeValue

        //         Caller-supplied value.

        //

        // Outputs:

        //     None.

        //

        // Example:

        //     let result = spanda_transport_routing::publish_external(&mut self, kind, opic_path, essage_type, value);

        if self.uses_registry_transport(kind) {
            let _ = self.with_registry_transport(kind, |provider| {
                if provider.is_connected() {
                    provider.publish(topic_path, message_type, value);
                }
            });
            return;
        }
        if let Some(adapter) = self.adapter_mut(kind) {
            adapter.publish(topic_path, message_type, value);
        }
    }

    fn subscribe_external(&mut self, kind: TransportKind, topic_path: &str) {
        // Description:

        //     Subscribe external.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     kind: TransportKind

        //         Caller-supplied kind.

        //     opic_path: &str

        //         Caller-supplied opic path.

        //

        // Outputs:

        //     None.

        //

        // Example:

        //     let result = spanda_transport_routing::subscribe_external(&mut self, kind, opic_path);

        if let Some(()) = self.with_registry_transport(kind, |provider| {
            provider.subscribe(topic_path);
        }) {
            return;
        }
        if let Some(adapter) = self.adapter_mut(kind) {
            adapter.subscribe(topic_path);
        }
    }

    fn receive_external(&mut self, kind: TransportKind, topic_path: &str) -> Option<RuntimeValue> {
        // Description:

        //     Receive external.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     kind: TransportKind

        //         Caller-supplied kind.

        //     opic_path: &str

        //         Caller-supplied opic path.

        //

        // Outputs:

        //     result: Option<RuntimeValue>

        //         Return value from `receive_external`.

        //

        // Example:

        //     let result = spanda_transport_routing::receive_external(&mut self, kind, opic_path);

        if let Some(value) = self
            .with_registry_transport(kind, |provider| {
                if provider.is_connected() {
                    provider.receive(topic_path)
                } else {
                    None
                }
            })
            .flatten()
        {
            return Some(value);
        }
        if let Some(adapter) = self.adapter_mut(kind) {
            if adapter.is_connected() {
                return adapter.receive(topic_path);
            }
        }
        None
    }

    fn connect_external(&mut self, kind: TransportKind, config: &TransportConfig) {
        // Description:

        //     Connect external.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     kind: TransportKind

        //         Caller-supplied kind.

        //     config: &TransportConfig

        //         Caller-supplied config.

        //

        // Outputs:

        //     None.

        //

        // Example:

        //     let result = spanda_transport_routing::connect_external(&mut self, kind, config);

        if let Some(()) = self.with_registry_transport(kind, |provider| {
            let runtime_config = adapter_config_to_runtime(config);
            let _ = provider.connect(&runtime_config);
        }) {
            return;
        }
        let _ = match kind {
            TransportKind::Ros2 => self.ros2.connect(config),
            TransportKind::Mqtt => self.mqtt.connect(config),
            TransportKind::Dds => self.dds.connect(config),
            TransportKind::Websocket => self.websocket.connect(config),
            TransportKind::Local | TransportKind::Sim => Ok(()),
        };
    }

    fn disconnect_external(&mut self, kind: TransportKind) {
        // Description:

        //     Disconnect external.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     kind: TransportKind

        //         Caller-supplied kind.

        //

        // Outputs:

        //     None.

        //

        // Example:

        //     let result = spanda_transport_routing::disconnect_external(&mut self, kind);

        if let Some(()) = self.with_registry_transport(kind, |provider| {
            provider.disconnect();
        }) {
            return;
        }
        match kind {
            TransportKind::Ros2 => self.ros2.disconnect(),
            TransportKind::Mqtt => self.mqtt.disconnect(),
            TransportKind::Dds => self.dds.disconnect(),
            TransportKind::Websocket => self.websocket.disconnect(),
            TransportKind::Local | TransportKind::Sim => {}
        }
    }

    pub fn configure(&mut self, config: TransportConfig) -> Result<(), String> {
        // Description:

        //     Configure.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     config: TransportConfig

        //         Caller-supplied config.

        //

        // Outputs:

        //     result: Result<(), String>

        //         Return value from `configure`.

        //

        // Example:

        //     let result = spanda_transport_routing::configure(&mut self, config);

        let mut config = config;
        if TransportSecurityConfig::url_requires_tls(config.broker_url.as_deref())
            && config.security.encryption == EncryptionMode::None
        {
            config.security.encryption = EncryptionMode::Required;
        }
        config.security.validate("transport")?;
        config
            .tls
            .connect(&config.security, config.broker_url.as_deref())?;
        self.config = config.clone();
        self.ros2.connect(&config)?;
        self.mqtt.connect(&TransportConfig {
            broker_url: config
                .broker_url
                .clone()
                .or(Some("mqtt://localhost:1883".into())),
            client_id: config.client_id.clone().or(Some("spanda".into())),
            ..config.clone()
        })?;
        self.dds.connect(&TransportConfig {
            domain_id: config.domain_id.or(Some(0)),
            ..config.clone()
        })?;
        self.websocket.connect(&TransportConfig {
            broker_url: config
                .broker_url
                .clone()
                .or(Some("ws://localhost:9090".into())),
            ..config
        })?;
        Ok(())
    }

    pub fn adapter(&self, kind: TransportKind) -> Option<&dyn TransportAdapter> {
        // Description:

        //     Adapter.

        //

        // Inputs:

        //     &self: value

        //         Caller-supplied &self.

        //     kind: TransportKind

        //         Caller-supplied kind.

        //

        // Outputs:

        //     result: Option<&dyn TransportAdapter>

        //         Return value from `adapter`.

        //

        // Example:

        //     let result = spanda_transport_routing::adapter(&self, kind);
        if self.uses_registry_transport(kind) {
            return None;
        }
        match kind {
            TransportKind::Ros2 => Some(&self.ros2),
            TransportKind::Mqtt => Some(&self.mqtt),
            TransportKind::Dds => Some(&self.dds),
            TransportKind::Websocket => Some(&self.websocket),
            TransportKind::Local | TransportKind::Sim => None,
        }
    }

    pub fn adapter_mut(&mut self, kind: TransportKind) -> Option<&mut dyn TransportAdapter> {
        // Description:

        //     Adapter mut.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     kind: TransportKind

        //         Caller-supplied kind.

        //

        // Outputs:

        //     result: Option<&mut dyn TransportAdapter>

        //         Return value from `adapter_mut`.

        //

        // Example:

        //     let result = spanda_transport_routing::adapter_mut(&mut self, kind);
        if self.uses_registry_transport(kind) {
            return None;
        }
        match kind {
            TransportKind::Ros2 => Some(&mut self.ros2),
            TransportKind::Mqtt => Some(&mut self.mqtt),
            TransportKind::Dds => Some(&mut self.dds),
            TransportKind::Websocket => Some(&mut self.websocket),
            TransportKind::Local | TransportKind::Sim => None,
        }
    }

    pub fn memory(&self) -> &InMemoryCommBus {
        // Description:

        //     Memory.

        //

        // Inputs:

        //     &self: value

        //         Caller-supplied &self.

        //

        // Outputs:

        //     result: &InMemoryCommBus

        //         Return value from `memory`.

        //

        // Example:

        //     let result = spanda_transport_routing::memory(&self);
        &self.memory
    }

    pub fn memory_mut(&mut self) -> &mut InMemoryCommBus {
        // Description:

        //     Memory mut.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //

        // Outputs:

        //     result: &mut InMemoryCommBus

        //         Return value from `memory_mut`.

        //

        // Example:

        //     let result = spanda_transport_routing::memory_mut(&mut self);
        &mut self.memory
    }

    fn is_external_connected(&self, kind: TransportKind) -> bool {
        // Description:

        //     Is external connected.

        //

        // Inputs:

        //     &self: value

        //         Caller-supplied &self.

        //     kind: TransportKind

        //         Caller-supplied kind.

        //

        // Outputs:

        //     result: bool

        //         Return value from `is_external_connected`.

        //

        // Example:

        //     let result = spanda_transport_routing::is_external_connected(&self, kind);

        if let Some(connected) =
            self.with_registry_transport(kind, |provider| provider.is_connected())
        {
            return connected;
        }
        self.adapter(kind)
            .map(|adapter| adapter.is_connected())
            .unwrap_or(false)
    }

    pub fn register_robot(&mut self, name: impl Into<String>) {
        // Description:

        //     Register robot.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     name: impl Into<String>

        //         Caller-supplied name.

        //

        // Outputs:

        //     None.

        //

        // Example:

        //     let result = spanda_transport_routing::register_robot(&mut self, name);
        self.memory.register_robot(name);
    }

    pub fn register_agent(&mut self, name: impl Into<String>) {
        // Description:

        //     Register agent.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     name: impl Into<String>

        //         Caller-supplied name.

        //

        // Outputs:

        //     None.

        //

        // Example:

        //     let result = spanda_transport_routing::register_agent(&mut self, name);
        self.memory.register_agent(name);
    }

    pub fn register_device(&mut self, name: impl Into<String>) {
        // Description:

        //     Register device.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     name: impl Into<String>

        //         Caller-supplied name.

        //

        // Outputs:

        //     None.

        //

        // Example:

        //     let result = spanda_transport_routing::register_device(&mut self, name);
        self.memory.register_device(name);
    }

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

        //     &mut self: value

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

        //     let result = spanda_transport_routing::publish_peer(&mut self, peer, opic, value, ranspor, source_id);
        self.memory
            .publish_peer(peer, topic, value, transport, source_id);
    }

    /// Poll external transport adapters for inbound messages on subscribed topics.
    pub fn poll_inbound(&mut self, transport: TransportKind) -> Vec<(String, CommEnvelope)> {
        // Description:
        //     Poll inbound.
        //
        // Inputs:
        //     &mut self: value
        //         Caller-supplied &mut self.
        //     ranspor: TransportKind
        //         Caller-supplied ranspor.
        //
        // Outputs:
        //     result: Vec<(String, CommEnvelope)>
        //         Return value from `poll_inbound`.
        //
        // Example:
        //     let result = spanda_transport_routing::poll_inbound(&mut self, ranspor);
        let paths = self.memory.subscription_paths();
        let mut inbound = Vec::new();
        let kinds = [
            transport,
            TransportKind::Ros2,
            TransportKind::Mqtt,
            TransportKind::Dds,
            TransportKind::Websocket,
        ];

        // Process each filesystem path.
        for path in paths {
            // Process each kind.
            for kind in kinds {
                if !self.is_external_connected(kind) {
                    continue;
                }

                // Emit output when receive provides a value.
                if let Some(value) = self.receive_external(kind, &path) {
                    let (value, source_id) =
                        decode_wire_value(&self.config, value).unwrap_or_else(|_| {
                            (
                                RuntimeValue::String {
                                    value: "<wire-decode-failed>".into(),
                                },
                                None,
                            )
                        });
                    let envelope = CommEnvelope {
                        value: value.clone(),
                        source_id,
                    };
                    self.memory
                        .push_inbound(&path, value, envelope.source_id.as_deref());
                    inbound.push((path.clone(), envelope));
                }
            }
        }
        inbound
    }

    /// Connect the active transport adapter and resubscribe all in-memory topic paths.
    pub fn reconnect_transport(&mut self, transport: TransportKind) {
        // Description:
        //     Reconnect transport.
        //
        // Inputs:
        //     &mut self: value
        //         Caller-supplied &mut self.
        //     ranspor: TransportKind
        //         Caller-supplied ranspor.
        //
        // Outputs:
        //     None.
        //
        // Example:
        //     let result = spanda_transport_routing::reconnect_transport(&mut self, ranspor);

        let paths = self.memory.subscription_paths();
        let config = self.config.clone();

        // Tear down transports that are no longer the active kind.
        for kind in [
            TransportKind::Ros2,
            TransportKind::Mqtt,
            TransportKind::Dds,
            TransportKind::Websocket,
        ] {
            if kind != transport {
                self.disconnect_external(kind);
            }
        }

        // Connect the target transport when it is not already live.
        match transport {
            TransportKind::Ros2 if !self.is_external_connected(TransportKind::Ros2) => {
                self.connect_external(TransportKind::Ros2, &config);
            }
            TransportKind::Mqtt if !self.is_external_connected(TransportKind::Mqtt) => {
                self.connect_external(
                    TransportKind::Mqtt,
                    &TransportConfig {
                        broker_url: config
                            .broker_url
                            .clone()
                            .or(Some("mqtt://localhost:1883".into())),
                        client_id: config.client_id.clone().or(Some("spanda".into())),
                        ..config.clone()
                    },
                );
            }
            TransportKind::Dds if !self.is_external_connected(TransportKind::Dds) => {
                self.connect_external(
                    TransportKind::Dds,
                    &TransportConfig {
                        domain_id: config.domain_id.or(Some(0)),
                        ..config.clone()
                    },
                );
            }
            TransportKind::Websocket if !self.is_external_connected(TransportKind::Websocket) => {
                self.connect_external(
                    TransportKind::Websocket,
                    &TransportConfig {
                        broker_url: config
                            .broker_url
                            .clone()
                            .or(Some("ws://localhost:9090".into())),
                        ..config
                    },
                );
            }
            TransportKind::Local | TransportKind::Sim => return,
            _ => {}
        }

        // Resubscribe every topic path on the newly active transport.
        for path in paths {
            self.subscribe_external(transport, &path);
        }
    }
}

impl CommBus for RoutingCommBus {
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
        //     &mut self: value
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
        //     let result = spanda_transport_routing::publish(&mut self, opic_path, essage_type, value, ranspor, source_id);
        self.memory.publish(
            topic_path,
            message_type,
            value.clone(),
            transport,
            source_id,
        );

        // Encrypt for external transport adapters when TLS is enabled.
        let config = self.config.clone();
        if matches!(transport, TransportKind::Local | TransportKind::Sim) {
            return;
        }
        let wire_value = encode_wire_value(
            &config,
            topic_path,
            message_type,
            &value,
            source_id,
            transport,
        )
        .unwrap_or(value);
        self.publish_external(transport, topic_path, message_type, wire_value);
    }

    fn subscribe(&mut self, topic_path: &str, handler: &str) {
        // Description:

        //     Subscribe.

        //

        // Inputs:

        //     &mut self: value

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

        //     let result = spanda_transport_routing::subscribe(&mut self, opic_path, handler);
        self.memory.subscribe(topic_path, handler);
    }

    fn receive(&mut self, topic_path: &str) -> Option<RuntimeValue> {
        // Description:

        //     Receive.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     opic_path: &str

        //         Caller-supplied opic path.

        //

        // Outputs:

        //     result: Option<RuntimeValue>

        //         Return value from `receive`.

        //

        // Example:

        //     let result = spanda_transport_routing::receive(&mut self, opic_path);
        self.memory.receive(topic_path)
    }

    fn receive_envelope(&mut self, topic_path: &str) -> Option<CommEnvelope> {
        // Description:

        //     Receive envelope.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     opic_path: &str

        //         Caller-supplied opic path.

        //

        // Outputs:

        //     result: Option<CommEnvelope>

        //         Return value from `receive_envelope`.

        //

        // Example:

        //     let result = spanda_transport_routing::receive_envelope(&mut self, opic_path);
        self.memory.receive_envelope(topic_path)
    }

    fn call_service(
        &mut self,
        service_name: &str,
        service_type: &str,
        request: Option<RuntimeValue>,
    ) -> RuntimeValue {
        // Description:

        //     Call service.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     service_name: &str

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

        //     let result = spanda_transport_routing::call_service(&mut self, service_name, service_type, reques);
        self.memory
            .call_service(service_name, service_type, request.clone())
    }

    fn send_action(
        &mut self,
        action_name: &str,
        action_type: &str,
        goal: RuntimeValue,
    ) -> RuntimeValue {
        // Description:

        //     Send action.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     action_name: &str

        //         Caller-supplied action name.

        //     action_type: &str

        //         Caller-supplied action type.

        //     goal: RuntimeValue

        //         Caller-supplied goal.

        //

        // Outputs:

        //     result: RuntimeValue

        //         Return value from `send_action`.

        //

        // Example:

        //     let result = spanda_transport_routing::send_action(&mut self, action_name, action_type, goal);
        self.memory.send_action(action_name, action_type, goal)
    }

    fn discover(&self, target: DiscoverTarget, filter: &DiscoverFilter) -> Vec<String> {
        // Description:

        //     Discover.

        //

        // Inputs:

        //     &self: value

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

        //     let result = spanda_transport_routing::discover(&self, arge, filter);
        self.memory.discover(target, filter)
    }

    fn published_messages(&self) -> Vec<PublishedCommMessage> {
        // Description:

        //     Published messages.

        //

        // Inputs:

        //     &self: value

        //         Caller-supplied &self.

        //

        // Outputs:

        //     result: Vec<PublishedCommMessage>

        //         Return value from `published_messages`.

        //

        // Example:

        //     let result = spanda_transport_routing::published_messages(&self);
        self.memory.published_messages()
    }

    fn inject_fault(&mut self, fault: &str) {
        // Description:

        //     Inject fault.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     faul: &str

        //         Caller-supplied faul.

        //

        // Outputs:

        //     None.

        //

        // Example:

        //     let result = spanda_transport_routing::inject_fault(&mut self, faul);
        self.memory.inject_fault(fault);
    }

    fn set_network_config(&mut self, config: SimNetworkConfig) {
        // Description:

        //     Set network config.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     config: SimNetworkConfig

        //         Caller-supplied config.

        //

        // Outputs:

        //     None.

        //

        // Example:

        //     let result = spanda_transport_routing::set_network_config(&mut self, config);
        self.memory.set_network_config(config);
    }

    fn active_faults(&self) -> Vec<String> {
        // Description:

        //     Active faults.

        //

        // Inputs:

        //     &self: value

        //         Caller-supplied &self.

        //

        // Outputs:

        //     result: Vec<String>

        //         Return value from `active_faults`.

        //

        // Example:

        //     let result = spanda_transport_routing::active_faults(&self);
        self.memory.active_faults()
    }

    fn subscription_paths(&self) -> Vec<String> {
        // Description:

        //     Subscription paths.

        //

        // Inputs:

        //     &self: value

        //         Caller-supplied &self.

        //

        // Outputs:

        //     result: Vec<String>

        //         Return value from `subscription_paths`.

        //

        // Example:

        //     let result = spanda_transport_routing::subscription_paths(&self);
        self.memory.subscription_paths()
    }

    fn push_inbound(&mut self, topic_path: &str, value: RuntimeValue, source_id: Option<&str>) {
        // Description:

        //     Push inbound.

        //

        // Inputs:

        //     &mut self: value

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

        //     let result = spanda_transport_routing::push_inbound(&mut self, opic_path, value, source_id);
        self.memory.push_inbound(topic_path, value, source_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ros2_adapter_publish_when_connected() {
        // Description:
        //     Ros2 adapter publish when connected.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     None.
        //
        // Example:
        //     let result = spanda_transport_routing::ros2_adapter_publish_when_connected();

        let mut adapter = Ros2TransportAdapter::default();
        assert!(!adapter.is_connected());
        adapter
            .connect(&TransportConfig {
                node_name: Some("spanda".into()),
                ..Default::default()
            })
            .unwrap();
        adapter.publish("/scan", "Scan", RuntimeValue::Bool { value: true });
        assert_eq!(adapter.published().len(), 1);
        assert_eq!(adapter.published()[0].topic, "/scan");
    }

    #[test]
    fn routing_bus_delegates_ros2_publish() {
        // Description:
        //     Routing bus delegates ros2 publish.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     None.
        //
        // Example:
        //     let result = spanda_transport_routing::routing_bus_delegates_ros2_publish();

        let mut bus = RoutingCommBus::new();
        bus.configure(TransportConfig {
            node_name: Some("bot".into()),
            ..Default::default()
        })
        .unwrap();
        bus.publish(
            "/cmd_vel",
            "Velocity",
            RuntimeValue::Bool { value: true },
            TransportKind::Ros2,
            None,
        );
        assert_eq!(bus.published_messages().len(), 1);
        assert_eq!(bus.ros2.published().len(), 1);
    }

    #[test]
    fn sim_transport_stays_in_memory_only() {
        // Description:
        //     Sim transport stays in memory only.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     None.
        //
        // Example:
        //     let result = spanda_transport_routing::sim_transport_stays_in_memory_only();

        let mut bus = RoutingCommBus::new();
        bus.publish(
            "/local",
            "String",
            RuntimeValue::Bool { value: true },
            TransportKind::Sim,
            None,
        );
        assert_eq!(bus.published_messages().len(), 1);
        assert!(bus.ros2.published().is_empty());
    }

    #[test]
    fn reconnect_transport_disconnects_inactive_adapters() {
        // Description:
        //     Reconnect transport disconnects inactive adapters.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     None.
        //
        // Example:
        //     let result = spanda_transport_routing::reconnect_transport_disconnects_inactive_adapters();

        let mut bus = RoutingCommBus::new();
        bus.configure(TransportConfig::default()).unwrap();
        bus.subscribe("/scan", "handler");
        bus.reconnect_transport(TransportKind::Mqtt);
        assert!(bus.mqtt.is_connected());
        bus.reconnect_transport(TransportKind::Dds);
        assert!(!bus.mqtt.is_connected());
        assert!(bus.dds.is_connected());
    }

    #[test]
    fn reconnect_transport_resubscribes_on_dds() {
        // Description:
        //     Reconnect transport resubscribes on dds.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     None.
        //
        // Example:
        //     let result = spanda_transport_routing::reconnect_transport_resubscribes_on_dds();

        let mut bus = RoutingCommBus::new();
        bus.configure(TransportConfig::default()).unwrap();
        bus.subscribe("/scan", "handler");
        bus.reconnect_transport(TransportKind::Dds);
        assert!(bus.dds.is_connected());
    }
}
