use crate::category::PackageCategory;
use serde::Serialize;

/// Stub registry entry for local resolution (no public registry yet).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RegistryEntry {
    pub name: &'static str,
    pub description: &'static str,
    pub versions: &'static [&'static str],
    pub category: PackageCategory,
    pub license: &'static str,
    pub import_paths: &'static [&'static str],
}

/// Local stub registry — framework packages available for dependency resolution.
pub static LOCAL_REGISTRY: &[RegistryEntry] = &[
    RegistryEntry {
        name: "spanda-ros2",
        description: "ROS 2 integration framework",
        versions: &["0.1.0", "0.2.0"],
        category: PackageCategory::Ros2,
        license: "Apache-2.0",
        import_paths: &["robotics.ros2"],
    },
    RegistryEntry {
        name: "spanda-vision",
        description: "Computer vision utilities",
        versions: &["0.1.0"],
        category: PackageCategory::Vision,
        license: "Apache-2.0",
        import_paths: &["vision.core"],
    },
    RegistryEntry {
        name: "spanda-navigation",
        description: "Path planning and navigation",
        versions: &["0.1.0"],
        category: PackageCategory::Navigation,
        license: "Apache-2.0",
        import_paths: &["navigation.path_planning"],
    },
    RegistryEntry {
        name: "spanda-mqtt",
        description: "MQTT pub/sub transport",
        versions: &["0.1.0"],
        category: PackageCategory::Mqtt,
        license: "Apache-2.0",
        import_paths: &["communication.mqtt"],
    },
    RegistryEntry {
        name: "spanda-lidar-rplidar",
        description: "RPLidar driver adapter",
        versions: &["0.1.0"],
        category: PackageCategory::Sensors,
        license: "MIT",
        import_paths: &["sensors.lidar.rplidar"],
    },
    RegistryEntry {
        name: "spanda-openai",
        description: "OpenAI provider adapter",
        versions: &["0.1.0"],
        category: PackageCategory::Ai,
        license: "Apache-2.0",
        import_paths: &["ai.openai"],
    },
];

pub fn search_registry(query: &str) -> Vec<&'static RegistryEntry> {
    let q = query.to_lowercase();
    LOCAL_REGISTRY
        .iter()
        .filter(|e| {
            e.name.contains(&q)
                || e.description.to_lowercase().contains(&q)
                || e.category.as_str().contains(&q)
        })
        .collect()
}

pub fn find_registry_entry(name: &str) -> Option<&'static RegistryEntry> {
    LOCAL_REGISTRY.iter().find(|e| e.name == name)
}

pub fn all_import_paths() -> Vec<&'static str> {
    let mut paths: Vec<&'static str> = LOCAL_REGISTRY
        .iter()
        .flat_map(|e| e.import_paths.iter().copied())
        .collect();
    paths.extend(
        super::adapter::framework_packages()
            .iter()
            .flat_map(|p| p.import_paths.iter().copied()),
    );
    paths.sort_unstable();
    paths.dedup();
    paths
}
