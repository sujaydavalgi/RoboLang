use crate::adapter::framework_import_paths;
use crate::registry::all_import_paths;

/// Built-in module import paths (Phase 1 module system).
pub fn builtin_import_paths() -> &'static [&'static str] {
    &[
        "sensors.lidar",
        "sensors.camera",
        "sensors.imu",
        "motion.drive",
        "motion.arm",
        "navigation.planning",
        "navigation.path_planning",
        "navigation.localize",
        "navigation.slam",
        "safety.validate",
        "ai.reasoning",
        "ai.openai",
        "robotics.ros2",
        "communication.mqtt",
        "vision.opencv",
        "vision.yolo",
        "vision.core",
        "manipulation.grasp",
        "hri.dialogue",
        "twin.sync",
        "sim.gazebo",
        "sim.webots",
        "ledger.mock",
        "provenance.core",
        "identity.core",
        "supply_chain.trace",
        "std.core",
        "std.time",
        "std.units",
        "std.spatial",
        "std.math",
        "std.collections",
        "std.result",
        "std.io",
        "std.log",
        "std.ai",
        "std.robotics",
        "std.sensors",
        "std.actuators",
        "std.safety",
        "std.communication",
        "std.hardware",
        "std.sim",
        "std.twin",
        "std.hri",
        "std.security",
        "std.audit",
        "std.crypto",
        "std.network",
    ]
}

/// Resolve whether an import path is known (builtin, std, or package registry).
pub fn resolve_package_import(path: &str) -> bool {
    if builtin_import_paths().contains(&path) {
        return true;
    }
    all_import_paths().contains(&path)
}

/// All registered import paths for tooling / LSP.
pub fn all_registered_import_paths() -> Vec<String> {
    let mut paths: Vec<String> = builtin_import_paths()
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    paths.extend(all_import_paths().iter().map(|s| (*s).to_string()));
    paths.extend(framework_import_paths().iter().map(|s| (*s).to_string()));
    paths.sort_unstable();
    paths.dedup();
    paths
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_navigation_imports() {
        assert!(resolve_package_import("navigation.path_planning"));
        assert!(resolve_package_import("robotics.ros2"));
        assert!(resolve_package_import("ai.openai"));
    }
}
