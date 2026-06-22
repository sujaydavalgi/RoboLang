//! Robotics platform primitives: mission lifecycle, fleet grouping, and navigation helpers.
//!
//! Core language constructs (`mission`, `fleet`, `safety_zone`) are parsed into AST nodes in
//! [`crate::foundations`]. This module holds shared runtime state and validation helpers.

pub use spanda_ast::robotics_decl::*;
pub use spanda_runtime::robotics::{
    FleetRegistry, MissionRuntime, MissionState, ProgramSafetyZoneRegistry,
};

/// Validate certification standard identifiers at parse/type-check time.
pub fn validate_certification_standard(name: &str) -> Option<String> {
    if CertificationStandard::parse_ident(name).is_some() {
        return None;
    }
    Some(format!(
        "unknown certification standard '{name}' (expected ISO13849, IEC61508, or ISO26262)"
    ))
}

/// Validate fleet member names against declared robots.
pub fn validate_fleet_members(
    fleet_name: &str,
    members: &[String],
    robot_names: &[String],
) -> Option<String> {
    for member in members {
        if !robot_names.iter().any(|r| r == member) {
            return Some(format!(
                "fleet '{fleet_name}' references unknown robot '{member}'"
            ));
        }
    }
    None
}

/// Validate swarm declarations reference declared fleet groups.
pub fn validate_swarm_fleet(
    swarm_name: &str,
    fleet_name: &str,
    fleet_names: &[String],
) -> Option<String> {
    if fleet_names.iter().any(|name| name == fleet_name) {
        return None;
    }
    Some(format!(
        "swarm '{swarm_name}' references unknown fleet '{fleet_name}'"
    ))
}

/// Validate mission declarations have either duration or steps.
pub fn validate_mission_decl(
    name: &Option<String>,
    duration_hours: Option<f64>,
    steps: &[String],
) -> Option<String> {
    if duration_hours.is_none() && steps.is_empty() {
        let label = name
            .as_deref()
            .map(|n| format!("mission '{n}'"))
            .unwrap_or_else(|| "mission".into());
        return Some(format!(
            "{label} requires at least one of duration or mission steps"
        ));
    }
    None
}
