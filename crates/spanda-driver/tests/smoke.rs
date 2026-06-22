//! Native compile-driver smoke tests.
//!
use spanda_driver::{check, compile};

#[test]
fn driver_compiles_minimal_robot() {
    let source = r#"
robot HelloBot {
  actuator speaker: DifferentialDrive;
  behavior hello() {
    speaker.stop();
  }
}
"#;
    let result = compile(source);
    assert!(result.is_ok(), "compile failed: {:?}", result.err());
}

#[test]
fn driver_check_rejects_invalid_syntax() {
    let source = "robot {";
    let result = check(source);
    assert!(result.is_err());
}
