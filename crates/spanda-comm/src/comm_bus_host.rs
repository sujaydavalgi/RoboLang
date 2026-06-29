//! Extended communication bus host operations used by the interpreter runtime.
//!
use crate::{CommBus, CommEnvelope, InMemoryCommBus, TransportKind};
use spanda_runtime::providers::ProviderRegistry;
use spanda_runtime::security_types::CommTransportSetup;
use spanda_runtime::value::RuntimeValue;
use std::cell::RefCell;
use std::rc::Rc;

/// Communication bus with interpreter host registration and live transport hooks.
pub trait CommBusHost: CommBus {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;

    fn attach_provider_registry(&mut self, registry: Rc<RefCell<ProviderRegistry>>);

    fn register_robot(&mut self, name: &str);

    fn register_device(&mut self, name: &str);

    fn register_agent(&mut self, name: &str);

    fn publish_peer(
        &mut self,
        peer: &str,
        topic: &str,
        value: RuntimeValue,
        transport: TransportKind,
        source_id: Option<&str>,
    );

    fn reconnect_transport(&mut self, transport: TransportKind);

    fn poll_inbound(&mut self, transport: TransportKind) -> Vec<(String, CommEnvelope)>;

    fn configure_transport(&mut self, setup: CommTransportSetup) -> Result<(), String>;
}

/// In-memory comm bus host used for simulation and tests.
#[derive(Debug, Clone, Default)]
pub struct SimCommBusHost {
    inner: InMemoryCommBus,
}

impl SimCommBusHost {
    pub fn new() -> Self {
        Self {
            inner: InMemoryCommBus::new(),
        }
    }
}

impl CommBus for SimCommBusHost {
    fn publish(
        &mut self,
        topic_path: &str,
        message_type: &str,
        value: RuntimeValue,
        transport: TransportKind,
        source_id: Option<&str>,
    ) {
        self.inner
            .publish(topic_path, message_type, value, transport, source_id);
    }

    fn subscribe(&mut self, topic_path: &str, handler: &str) {
        self.inner.subscribe(topic_path, handler);
    }

    fn receive(&mut self, topic_path: &str) -> Option<RuntimeValue> {
        self.inner.receive(topic_path)
    }

    fn receive_envelope(&mut self, topic_path: &str) -> Option<CommEnvelope> {
        self.inner.receive_envelope(topic_path)
    }

    fn call_service(
        &mut self,
        service_name: &str,
        service_type: &str,
        request: Option<RuntimeValue>,
    ) -> RuntimeValue {
        self.inner
            .call_service(service_name, service_type, request)
    }

    fn send_action(
        &mut self,
        action_name: &str,
        action_type: &str,
        goal: RuntimeValue,
    ) -> RuntimeValue {
        self.inner.send_action(action_name, action_type, goal)
    }

    fn discover(
        &self,
        target: spanda_ast::comm_decl::DiscoverTarget,
        filter: &spanda_ast::comm_decl::DiscoverFilter,
    ) -> Vec<String> {
        self.inner.discover(target, filter)
    }

    fn published_messages(&self) -> Vec<crate::PublishedCommMessage> {
        self.inner.published_messages()
    }

    fn inject_fault(&mut self, fault: &str) {
        self.inner.inject_fault(fault);
    }

    fn set_network_config(&mut self, config: crate::SimNetworkConfig) {
        self.inner.set_network_config(config);
    }

    fn active_faults(&self) -> Vec<String> {
        self.inner.active_faults()
    }

    fn subscription_paths(&self) -> Vec<String> {
        self.inner.subscription_paths()
    }

    fn push_inbound(&mut self, topic_path: &str, value: RuntimeValue, source_id: Option<&str>) {
        self.inner.push_inbound(topic_path, value, source_id);
    }
}

impl CommBusHost for SimCommBusHost {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn attach_provider_registry(&mut self, _registry: Rc<RefCell<ProviderRegistry>>) {}

    fn register_robot(&mut self, name: &str) {
        self.inner.register_robot(name);
    }

    fn register_device(&mut self, name: &str) {
        self.inner.register_device(name);
    }

    fn register_agent(&mut self, name: &str) {
        self.inner.register_agent(name);
    }

    fn publish_peer(
        &mut self,
        peer: &str,
        topic: &str,
        value: RuntimeValue,
        transport: TransportKind,
        source_id: Option<&str>,
    ) {
        self.inner
            .publish_peer(peer, topic, value, transport, source_id);
    }

    fn reconnect_transport(&mut self, _transport: TransportKind) {}

    fn poll_inbound(&mut self, _transport: TransportKind) -> Vec<(String, CommEnvelope)> {
        Vec::new()
    }

    fn configure_transport(&mut self, _setup: CommTransportSetup) -> Result<(), String> {
        Ok(())
    }
}

/// Factory for comm bus host instances with provider registry attachment.
pub type CommBusFactory = fn(Rc<RefCell<ProviderRegistry>>) -> Box<dyn CommBusHost>;

/// Default in-memory comm bus factory for direct interpreter use.
pub fn default_comm_bus_factory(registry: Rc<RefCell<ProviderRegistry>>) -> Box<dyn CommBusHost> {
    let mut bus = SimCommBusHost::new();
    bus.attach_provider_registry(registry);
    Box::new(bus)
}

/// Default factory pointer for option wiring.
pub fn default_comm_bus_factory_fn() -> CommBusFactory {
    default_comm_bus_factory
}
