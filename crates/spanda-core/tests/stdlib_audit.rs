//! Standard library namespace and audit/provenance integration tests.

use spanda_core::{check, compile, run, RunOptions};
use spanda_package::{
    validate_capability, validate_package, ApplicationPermissions, PackageManifest,
};

#[test]
fn std_namespace_imports_resolve() {
    let source = r#"
import std.core;
import std.time;
import std.units;
import std.spatial;
import std.math;
import std.collections;
import std.result;
import std.io;
import std.log;
import std.ai;
import std.robotics;
import std.sensors;
import std.actuators;
import std.safety;
import std.communication;
import std.hardware;
import std.sim;
import std.twin;
import std.hri;
import std.security;
import std.audit;
import std.crypto;

robot Demo {
  behavior run() {}
}
"#;
    check(source).expect("all std namespaces should import");
}

#[test]
fn audit_record_creation_runtime() {
    let source = include_str!("../../../examples/std/audit_log.sd");
    let result = run(source, RunOptions::default()).expect("audit_log example should run");
    assert!(result.logs.iter().any(|l| l.contains("audit.record")));
}

#[test]
fn provenance_and_identity_example() {
    let source = include_str!("../../../examples/std/provenance.sd");
    compile(source).expect("provenance example should compile");
    run(source, RunOptions::default()).expect("provenance example should run");
}

#[test]
fn mock_ledger_anchoring_example() {
    let source = include_str!("../../../examples/std/mock_ledger.sd");
    let result = run(source, RunOptions::default()).expect("mock_ledger example should run");
    assert!(result.logs.iter().any(|l| l.contains("mock_ledger.anchor")));
}

#[test]
fn device_identity_example() {
    let source = include_str!("../../../examples/std/device_identity.sd");
    check(source).expect("device_identity example should type-check");
}

#[test]
fn audit_crypto_builtins_via_spanda_audit() {
    use spanda_core::audit::{sha256, sign, verify_signature};
    let hash = sha256("mission-data");
    assert_eq!(hash.0.len(), 64);
    let sig = sign("payload", "device-key");
    assert!(verify_signature("payload", &sig, "device-key"));
}

#[test]
fn package_audit_capabilities_known() {
    assert!(validate_capability("audit.write").is_ok());
    assert!(validate_capability("audit.read").is_ok());
    assert!(validate_capability("identity.sign").is_ok());
    assert!(validate_capability("identity.verify").is_ok());
    assert!(validate_capability("ledger.anchor").is_ok());
}

#[test]
fn package_metadata_for_audit_libraries() {
    let manifest = PackageManifest::parse_str(
        r#"
[package]
name = "audit_robot"
version = "0.1.0"
license = "Apache-2.0"

[dependencies]
spanda-provenance = "0.1.0"
spanda-ledger = "0.1.0"

[capabilities]
required = [
  "camera.read",
  "lidar.read",
  "network.outbound",
  "audit.write",
  "identity.sign",
  "ledger.anchor"
]

[safety]
level = "simulation_only"
"#,
    )
    .unwrap();
    let perms = ApplicationPermissions::permissive();
    let report = validate_package(&manifest, &perms).unwrap();
    assert!(report.ok());
}
