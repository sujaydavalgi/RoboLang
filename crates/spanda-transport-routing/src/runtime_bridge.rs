//! Routing comm bus host bridge for interpreter injection.
//!
use std::cell::RefCell;
use std::rc::Rc;

use spanda_comm::{CommBusFactory, CommBusHost, CommEnvelope, TransportKind};
use spanda_runtime::providers::ProviderRegistry;
use spanda_runtime::security_types::CommTransportSetup;
use spanda_runtime::value::RuntimeValue;
use spanda_security::policy::{
    AuthenticationMode, EncryptionMode, IntegrityMode,
};
use spanda_transport::adapter::TransportConfig;
use spanda_transport::security::TransportSecurityConfig;

use crate::RoutingCommBus;

impl CommBusHost for RoutingCommBus {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn attach_provider_registry(&mut self, registry: Rc<RefCell<ProviderRegistry>>) {
        RoutingCommBus::attach_provider_registry(self, registry);
    }

    fn register_robot(&mut self, name: &str) {
        RoutingCommBus::register_robot(self, name);
    }

    fn register_device(&mut self, name: &str) {
        RoutingCommBus::register_device(self, name);
    }

    fn register_agent(&mut self, name: &str) {
        RoutingCommBus::register_agent(self, name);
    }

    fn publish_peer(
        &mut self,
        peer: &str,
        topic: &str,
        value: RuntimeValue,
        transport: TransportKind,
        source_id: Option<&str>,
    ) {
        RoutingCommBus::publish_peer(self, peer, topic, value, transport, source_id);
    }

    fn reconnect_transport(&mut self, transport: TransportKind) {
        RoutingCommBus::reconnect_transport(self, transport);
    }

    fn poll_inbound(&mut self, transport: TransportKind) -> Vec<(String, CommEnvelope)> {
        RoutingCommBus::poll_inbound(self, transport)
    }

    fn configure_transport(&mut self, setup: CommTransportSetup) -> Result<(), String> {
        let security = TransportSecurityConfig {
            encryption: map_enc(setup.security.encryption),
            authentication: map_auth(setup.security.authentication),
            integrity: map_int(setup.security.integrity),
            cert_path: setup.security.cert_path,
            key_secret: setup.security.key_secret,
            key_path: setup.security.key_path,
        };
        RoutingCommBus::configure(
            self,
            TransportConfig {
                node_name: setup.node_name,
                broker_url: setup.broker_url,
                namespace: setup.namespace,
                domain_id: setup.domain_id,
                client_id: setup.client_id,
                security,
                ..Default::default()
            },
        )
    }
}

fn map_enc(mode: spanda_runtime::security_types::EncryptionMode) -> EncryptionMode {
    match mode {
        spanda_runtime::security_types::EncryptionMode::None => EncryptionMode::None,
        spanda_runtime::security_types::EncryptionMode::Optional => EncryptionMode::Optional,
        spanda_runtime::security_types::EncryptionMode::Required => EncryptionMode::Required,
    }
}

fn map_auth(mode: spanda_runtime::security_types::AuthenticationMode) -> AuthenticationMode {
    match mode {
        spanda_runtime::security_types::AuthenticationMode::None => AuthenticationMode::None,
        spanda_runtime::security_types::AuthenticationMode::Signed => AuthenticationMode::Signed,
        spanda_runtime::security_types::AuthenticationMode::Mutual => AuthenticationMode::Mutual,
    }
}

fn map_int(mode: spanda_runtime::security_types::IntegrityMode) -> IntegrityMode {
    match mode {
        spanda_runtime::security_types::IntegrityMode::None => IntegrityMode::None,
        spanda_runtime::security_types::IntegrityMode::Required => IntegrityMode::Required,
    }
}

/// Routing comm bus factory for CLI and driver injection.
pub fn routing_comm_bus_factory(registry: Rc<RefCell<ProviderRegistry>>) -> Box<dyn CommBusHost> {
    let mut bus = RoutingCommBus::new();
    bus.attach_provider_registry(registry);
    Box::new(bus)
}

/// Factory pointer for option wiring.
pub fn routing_comm_bus_factory_fn() -> CommBusFactory {
    routing_comm_bus_factory
}
