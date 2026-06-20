//! Integration tests for the spanda-package crate.

use semver::Version;
use spanda_package::{
    find_project_root, parse_version_req, validate_capability, validate_package, version_satisfies,
    ApplicationPermissions, DependencyDetail, DependencySpec, Lockfile, PackageManifest,
    ResolveOptions, SafetyLevel,
};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn spanda_toml_parsing_full() {
    let content = r#"
[package]
name = "warehouse_robot"
version = "0.1.0"
description = "Warehouse robot controller"
license = "Apache-2.0"

[dependencies]
spanda-ros2 = "0.1.0"
local-lib = { path = "../local-lib" }

[hardware]
targets = ["RoverV1", "JetsonOrin"]

[capabilities]
uses = ["network.outbound", "camera.read"]
required = ["lidar.read", "motion.propose"]

[requires_hardware]
memory = ">=2GB"
gpu = ">=1 TOPS"
sensors = ["Camera", "Lidar"]

[safety]
level = "hardware_safe"
requires_review = false
can_control_actuators = true

[adapter]
provides = ["LidarAdapter", "Topic<LidarScan>"]
requires = ["serial.port", "lidar.read"]

categories = ["robotics", "navigation"]
"#;
    let m = PackageManifest::parse_str(content).unwrap();
    assert_eq!(m.package.name, "warehouse_robot");
    assert_eq!(m.dependencies.len(), 2);
    assert_eq!(m.safety.level, SafetyLevel::HardwareSafe);
    assert_eq!(m.adapter.provides.len(), 2);
}

#[test]
fn dependency_parsing_local_and_git() {
    let spec = DependencySpec::Detail(DependencyDetail {
        version: None,
        path: Some(PathBuf::from("../lib")),
        git: None,
        branch: None,
        tag: None,
        rev: None,
    });
    assert_eq!(
        spec.source_kind(),
        spanda_package::DependencySourceKind::Local
    );

    let git_spec = DependencySpec::Detail(DependencyDetail {
        version: None,
        path: None,
        git: Some("https://github.com/example/pkg".into()),
        branch: Some("main".into()),
        tag: None,
        rev: None,
    });
    assert_eq!(
        git_spec.source_kind(),
        spanda_package::DependencySourceKind::Git
    );
}

#[test]
fn version_constraint_validation() {
    let req = parse_version_req("^0.1.0").unwrap();
    assert!(version_satisfies(&Version::new(0, 1, 9), &req));
    assert!(!version_satisfies(&Version::new(0, 2, 0), &req));
}

#[test]
fn lockfile_generation_roundtrip() {
    let root = TempDir::new().unwrap();
    let content = r#"
[package]
name = "app"
version = "0.1.0"

[dependencies]
spanda-ros2 = "0.1.0"
"#;
    fs::write(root.path().join("spanda.toml"), content).unwrap();
    let manifest = PackageManifest::load_from_dir(root.path()).unwrap();
    let result =
        spanda_package::resolve_dependencies(root.path(), &manifest, &ResolveOptions::default())
            .unwrap();
    let lockfile = Lockfile::new(&manifest, result.lockfile_deps);
    lockfile.save_to_dir(root.path()).unwrap();
    let loaded = Lockfile::load_from_dir(root.path()).unwrap();
    assert_eq!(loaded.package.name, "app");
    assert!(loaded.dependencies.contains_key("spanda-ros2"));
}

#[test]
fn capability_validation_known_and_unknown() {
    assert!(validate_capability("camera.read").is_ok());
    assert!(validate_capability("unknown.cap").is_err());
}

#[test]
fn package_safety_level_validation() {
    let manifest = PackageManifest::parse_str(
        r#"
[package]
name = "unsafe"
version = "0.1.0"

[safety]
level = "simulation_only"
can_control_actuators = true
"#,
    )
    .unwrap();
    let perms = ApplicationPermissions::permissive();
    assert!(validate_package(&manifest, &perms).is_err());
}

#[test]
fn find_project_root_walks_up() {
    let root = TempDir::new().unwrap();
    fs::write(
        root.path().join("spanda.toml"),
        "[package]\nname=\"a\"\nversion=\"0.1.0\"\n",
    )
    .unwrap();
    let nested = root.path().join("src").join("sub");
    fs::create_dir_all(&nested).unwrap();
    let found = find_project_root(&nested).unwrap();
    assert_eq!(found, root.path());
}
