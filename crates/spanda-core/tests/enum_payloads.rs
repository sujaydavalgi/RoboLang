use spanda_core::{check, run, RunOptions};

#[test]
fn enum_payload_constructor_and_match_bindings() {
    let source = r#"
enum Command {
  Stop,
  Drive(Float, Float)
}

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let cmd = Drive(0.3, 0.0);
    match cmd {
      Stop => wheels.stop();
      Drive(_speed, _turn) => wheels.drive(linear: 0.3 m/s, angular: 0.0 rad/s);
    };
  }
}
"#;
    check(source).expect("enum payload program should type-check");
    run(source, RunOptions::default()).expect("enum payload match should run");
}

#[test]
fn enum_payload_arity_mismatch_rejected() {
    let source = r#"
enum Command { Drive(Float, Float) }
robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let x = Drive(1.0);
  }
}
"#;
    let err = check(source).expect_err("wrong arity should fail");
    assert!(
        err.diagnostics()
            .iter()
            .any(|d| d.message.contains("expects 2 payload")),
        "got {:?}",
        err.diagnostics()
    );
}
