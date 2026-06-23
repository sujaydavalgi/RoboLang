//! Phase 30 health polling during trigger-loop runtime tests.

use spanda_core::{run, RunOptions};

#[test]
fn continuous_health_polls_during_trigger_loop() {
    let source = r#"
health_check RoverHealth for robot Rover {
    check gps.status == Healthy;
}

robot Rover {
    sensor gps: GPS;
    actuator wheels: DifferentialDrive;
    safety { max_speed = 1.0 m/s; }

    on hardware GpsFailure {
        wheels.stop();
    }

    every 50ms {
        let tick = true;
    }
}
"#;
    let result = run(
        source,
        RunOptions {
            inject_health_faults: true,
            max_loop_iterations: 3,
            ..Default::default()
        },
    )
    .expect("trigger loop should run");
    let health_logs: Vec<_> = result
        .logs
        .iter()
        .filter(|l| l.contains("health: overall"))
        .collect();
    assert!(
        !health_logs.is_empty(),
        "expected health polling logs during trigger loop, got {:?}",
        result.logs
    );
}
