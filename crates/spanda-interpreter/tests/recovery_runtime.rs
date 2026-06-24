//! Runtime recovery action dispatch integration tests.

use spanda_interpreter::{run_program, RunOptions};
use spanda_lexer::tokenize;
use spanda_parser::parse;

#[test]
fn recovery_policy_dispatches_degraded_mode_on_health_fault() {
    let source = r#"
hardware H {
    sensors [GPS, Lidar];
    actuators [DifferentialDrive];
}

recovery_policy RoverRecovery {
    on gps.failed {
        enter degraded_mode;
        reduce_speed 0.4 m/s;
    }
}

on anomaly NavigationFault severity High {
    enter degraded_mode;
}

anomaly_detector NavigationFault {
    expected gps.accuracy <= 3 m;
}

robot Rover {
    sensor gps: GPS;
    sensor lidar: Lidar;
    actuator wheels: DifferentialDrive;
    mode degraded { }
    safety { max_speed = 1.0 m/s; }
    behavior patrol() {
        loop every 50ms {
            let _ = gps.read();
        }
    }
}
"#;
    std::env::set_var("SPANDA_OPERATOR_APPROVAL", "1");
    let program = parse(tokenize(source).unwrap()).unwrap();
    let result = run_program(
        &program,
        RunOptions {
            inject_health_faults: true,
            max_loop_iterations: 3,
            ..Default::default()
        },
    )
    .expect("run");
    assert!(
        result
            .logs
            .iter()
            .any(|l| l.contains("recovery:") || l.contains("mode: entered")),
        "expected recovery or mode dispatch logs, got: {:?}",
        result.logs
    );
}
