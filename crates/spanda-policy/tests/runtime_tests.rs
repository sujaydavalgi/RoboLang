//! Runtime policy enforcement tests.

use chrono::NaiveTime;
use spanda_lexer::tokenize;
use spanda_parser::parse;
use spanda_policy::{
    build_runtime_policy_monitor, check_runtime_policy_motion, RuntimePolicyMonitor,
};

fn parse_source(source: &str) -> spanda_ast::nodes::Program {
    let tokens = tokenize(source).expect("tokenize");
    parse(tokens).expect("parse")
}

#[test]
fn runtime_max_speed_blocks_excess_motion() {
    let source = r#"
policy SlowPolicy {
    max_speed = 0.5 m/s;
}

hardware RoverV1 {
  sensors [ GPS ];
  actuators [ DifferentialDrive ];
}

robot Rover {
  uses hardware RoverV1;
  actuator wheels: DifferentialDrive;
  safety { max_speed = 1.0 m/s; }
}
"#;
    let program = parse_source(source);
    let monitor = build_runtime_policy_monitor(&program, "SlowPolicy").unwrap();
    assert!(check_runtime_policy_motion(&monitor, 0.4).is_ok());
    let violation = check_runtime_policy_motion(&monitor, 0.8).unwrap_err();
    assert_eq!(violation.rule, "max_speed");
}

#[test]
fn runtime_operation_hours_respects_clock_override() {
    let monitor = RuntimePolicyMonitor {
        policy_name: "HoursPolicy".into(),
        max_speed_mps: None,
        operation_hours: Some("06:00-22:00".into()),
        clock_override: Some(NaiveTime::from_hms_opt(3, 0, 0).unwrap()),
    };
    let violation = check_runtime_policy_motion(&monitor, 0.1).unwrap_err();
    assert_eq!(violation.rule, "operation_hours");
    let mut allowed = monitor.clone();
    allowed.clock_override = Some(NaiveTime::from_hms_opt(12, 0, 0).unwrap());
    assert!(check_runtime_policy_motion(&allowed, 0.1).is_ok());
}
