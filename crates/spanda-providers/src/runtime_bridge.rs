//! Provider-backed implementation of the runtime provider dispatch boundary.
//!
use spanda_runtime::provider_runtime::{ProviderDispatchContext, ProviderRuntime};
use spanda_runtime::providers::ProviderRegistry;
use spanda_runtime::value::RuntimeValue;
use spanda_transport_routing::RoutingCommBus;

use crate::bootstrap::{bootstrap_providers_for_packages, sync_comm_bus_for_official_packages};
use crate::package_dispatch::dispatch_official_package_call;

/// Full provider runtime delegating to `spanda-providers` bootstrap and dispatch.
#[derive(Debug, Default, Clone, Copy)]
pub struct ProviderBackedRuntime;

impl ProviderRuntime for ProviderBackedRuntime {
    fn bootstrap_providers_for_packages(&self, package_names: &[&str]) -> ProviderRegistry {
        bootstrap_providers_for_packages(package_names)
    }

    fn sync_comm_bus(
        &self,
        comm_bus: &mut dyn std::any::Any,
        registry: &mut ProviderRegistry,
    ) {
        if let Some(bus) = comm_bus.downcast_mut::<RoutingCommBus>() {
            sync_comm_bus_for_official_packages(bus, registry);
        }
    }

    fn dispatch_official_package_call(
        &self,
        registry: &mut ProviderRegistry,
        module_path: &str,
        function_name: &str,
        args: &[RuntimeValue],
        context: ProviderDispatchContext<'_>,
    ) -> Option<RuntimeValue> {
        dispatch_official_package_call(
            registry,
            module_path,
            function_name,
            args,
            context.telemetry,
            context.mission_trace,
            context.sim_time_ms,
        )
    }
}
