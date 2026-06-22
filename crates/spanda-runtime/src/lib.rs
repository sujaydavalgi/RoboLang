//! Spanda runtime kernel primitives extracted for the Phase 4 lean-core split.
//!
pub mod classification;
pub mod provider_types;
pub mod robotics;
pub mod scheduler;

pub use classification::{
    module_classifications, official_package_names, ModuleClassification, ModuleOwnership,
};
pub use provider_types::{
    ProviderCapability, ProviderCapabilitySet, ProviderError, ProviderId, ProviderMetadata,
    ProviderResult, ProviderSafetyLevel,
};
pub use robotics::{
    FleetRegistry, MissionRuntime, MissionState, ProgramSafetyZoneRegistry,
};
pub use scheduler::{
    advance_wall_tick, elapsed_ms, sleep_until, SchedulerClock,
};
