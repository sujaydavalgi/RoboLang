use spanda_core::{check, compile, run, RunOptions};

#[test]
fn observe_block_type_checks_and_fuses_sensors() {
    let source = r#"
robot R {
  sensor camera: Camera on "/camera";
  sensor lidar: Lidar on "/scan";
  sensor imu: IMU;
  safety { max_speed = 1.0 m/s; }
  observe {
    camera;
    lidar;
    imu;
  }
  behavior run() {
    let fused = fusion.read();
    let _ = fused.pose;
    let _ = fused.count;
  }
}
"#;
    check(source).expect("observe block should type-check");
    let result = run(
        source,
        RunOptions {
            max_loop_iterations: 1,
            ..Default::default()
        },
    )
    .expect("observe fusion should run");
    assert!(
        result
            .logs
            .iter()
            .any(|l| l.contains("observe: fusing 3 sensor")),
        "expected observe setup log, got: {:?}",
        result.logs
    );
}

#[test]
fn observe_unknown_sensor_rejected_at_typecheck() {
    let source = r#"
robot R {
  sensor lidar: Lidar on "/scan";
  observe { missing; }
  behavior run() {}
}
"#;
    let err = check(source).expect_err("unknown observe sensor should fail");
    assert!(
        err.diagnostics()
            .iter()
            .any(|d| d.message.contains("observe references unknown sensor")),
        "expected observe sensor error, got: {:?}",
        err.diagnostics()
    );
}

#[test]
fn fusion_example_runs() {
    let source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/types/fusion.sd"
    ))
    .expect("read fusion example");
    compile(&source).expect("fusion example should compile");
    run(&source, RunOptions::default()).expect("fusion example should run");
}
