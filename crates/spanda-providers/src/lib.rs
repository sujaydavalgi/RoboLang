//! Official package provider bootstrap and transport adapter wiring.
//!
pub mod bootstrap;
pub mod package_stubs;
pub mod transport_adapter;

pub use bootstrap::{
    bootstrap_default_providers, bootstrap_providers_for_packages, official_package_for_transport,
    sync_comm_bus_for_official_packages,
};
pub use transport_adapter::{adapter_config_to_runtime, TransportAdapterProvider};
