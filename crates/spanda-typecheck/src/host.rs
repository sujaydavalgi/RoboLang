//! Host hooks for domain-specific type-check validation in `spanda-core`.
//!
use crate::diagnostics::Diagnostic;
use spanda_ast::foundations::{ResourceBudgetDecl, TaskDecl};
use spanda_ast::nodes::{HalMemberDecl, Span, SpandaType};
use std::collections::HashMap;

/// Domain-specific import, library, SoC, and robotics validation supplied by the embedding runtime.
pub trait TypeCheckHost {
    /// Whether an import path resolves via stdlib, FFI, vendor libraries, or AI modules.
    fn import_path_known(&self, path: &str, module_registry_has_export: bool) -> bool;

    /// Whether a SLAM adapter import path is recognized.
    fn slam_import_known(&self, path: &str) -> bool;

    /// When `Some(false)`, the library is known but does not export the sensor type.
    fn library_exports_sensor(&self, library: &str, sensor_type: &str) -> Option<bool>;

    /// Whether a vendor sensor type name is registered in the library catalog.
    fn library_sensor_type_known(&self, sensor_type: &str) -> bool;

    /// Robotic type for each registered vendor sensor type name.
    fn library_sensor_robo_types(&self) -> HashMap<String, SpandaType>;

    /// Resolve the vendor package name backing a sensor type, if any.
    fn library_for_sensor_type(&self, sensor_type: &str) -> Option<String>;

    /// Whether a declared SoC profile identifier is recognized.
    fn soc_profile_known(&self, profile: &str) -> bool;

    /// Validate HAL members against a SoC profile; returns human-readable error messages.
    fn validate_hal_against_soc(&self, profile: &str, members: &[HalMemberDecl]) -> Vec<String>;

    fn validate_fleet_members(
        &self,
        fleet_name: &str,
        members: &[String],
        robot_names: &[String],
    ) -> Option<String>;

    fn validate_swarm_fleet(
        &self,
        swarm_name: &str,
        fleet_name: &str,
        fleet_names: &[String],
    ) -> Option<String>;

    fn validate_mission_decl(
        &self,
        name: &Option<String>,
        duration_hours: Option<f64>,
        steps: &[String],
    ) -> Option<String>;

    /// Whether a security capability name is recognized.
    fn security_capability_known(&self, capability: &str) -> bool;

    fn validate_task_timing(&self, task: &TaskDecl) -> Vec<Diagnostic>;

    fn validate_task_priority(&self, task: &TaskDecl) -> Vec<Diagnostic>;

    fn validate_resource_budget(&self, budget: &ResourceBudgetDecl, span: Span) -> Vec<Diagnostic>;
}
