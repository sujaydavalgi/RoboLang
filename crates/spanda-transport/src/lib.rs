//! Transport adapter traits, security policy, and TLS session primitives.
//!
//! Shared types live here so optional transport backend crates can depend on
//! `spanda-transport` without pulling in the full `spanda-core` compiler.

pub mod adapter;
pub mod security;
pub mod tls;

pub use adapter::{
    payload_string_for_service, AdapterMessage, StubTransportState, TransportAdapter,
    TransportConfig,
};
pub use security::{
    effective_transport_policy, TlsTransportSession, TlsTransportStub, TransportSecurityConfig,
};
pub use tls::{
    build_client_config, parse_tls_endpoint, perform_mtls_handshake, MtlsHandshakeResult,
    TlsEndpoint,
};
