//! Bridge implementing `ReadinessRuntime` for spanda-readiness real evaluation.

use spanda_runtime::readiness_runtime::{set_readiness_runtime, ReadinessRuntime};
use std::sync::Arc;

/// Concrete implementation of ReadinessRuntime backed by the real spanda-readiness engine.
#[derive(Debug, Default, Clone, Copy)]
pub struct ReadinessBackedRuntime;

impl ReadinessRuntime for ReadinessBackedRuntime {
    fn evaluate_agent_readiness_json(
        &self,
        source: &str,
        target: Option<&str>,
        include_runtime: bool,
        inject_health_faults: bool,
    ) -> Result<String, String> {
        // Delegate to the full readiness evaluation engine.
        crate::agent::evaluate_agent_readiness_json(
            source,
            target,
            include_runtime,
            inject_health_faults,
        )
    }
}

/// Register the real readiness runtime with the global OnceLock.
///
/// Parameters:
/// None.
///
/// Returns:
/// Unit; idempotent (subsequent calls are silently ignored).
///
/// Options:
/// None.
///
/// Example:
/// spanda_readiness::runtime_bridge::register();
pub fn register() {
    // Inject the real readiness engine into the global runtime slot.
    set_readiness_runtime(Arc::new(ReadinessBackedRuntime));
}
