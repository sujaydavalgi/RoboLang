use spanda_core::{run, RunOptions};

#[test]
fn multiplexes_multiple_tasks_without_entry_behavior() {
    let source = r#"
robot R {
  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;
  safety { max_speed = 1.0 m/s; }

  task sense every 50ms {
    let _ = lidar.read();
  }

  task drive every 100ms {
    wheels.drive(linear: 0.2 m/s, angular: 0.0 rad/s);
  }
}
"#;
    let result = run(
        source,
        RunOptions {
            max_loop_iterations: 4,
            ..Default::default()
        },
    )
    .expect("multiplexed tasks should run");
    assert!(
        result
            .logs
            .iter()
            .any(|l| l.contains("scheduler: multiplexing 2 task")),
        "expected scheduler log, got: {:?}",
        result.logs
    );
    assert!(
        result.logs.iter().any(|l| l.contains("task 'sense': tick")),
        "expected sense task ticks, got: {:?}",
        result.logs
    );
    assert!(
        result.logs.iter().any(|l| l.contains("task 'drive': tick")),
        "expected drive task ticks, got: {:?}",
        result.logs
    );
}

#[test]
fn multitask_example_runs() {
    let source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/types/multitask.sd"
    ))
    .expect("read multitask example");
    run(&source, RunOptions::default()).expect("multitask example should run");
}
