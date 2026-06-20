use spanda_core::{run, RunOptions};

#[test]
fn trace_scheduler_emits_diagnostics_and_metrics() {
    let source = r#"
robot R {
  actuator wheels: DifferentialDrive;
  task sense every 50ms { wheels.stop(); }
  task drive every 100ms { wheels.stop(); }
}
"#;
    let result = run(
        source,
        RunOptions {
            max_loop_iterations: 3,
            trace_scheduler: true,
            ..Default::default()
        },
    )
    .expect("scheduler trace should run");
    assert!(
        result.logs.iter().any(|l| l.contains("trace-scheduler:")),
        "expected scheduler trace logs, got {:?}",
        result.logs
    );
    assert_eq!(result.metrics.scheduler.multiplexed_tasks, 2);
    assert!(result.metrics.scheduler.scheduler_ticks >= 3);
}

#[test]
fn trace_tasks_records_per_task_ticks() {
    let source = r#"
robot R {
  actuator wheels: DifferentialDrive;
  task SafetyMonitor critical every 50ms { wheels.stop(); }
}
"#;
    let result = run(
        source,
        RunOptions {
            max_loop_iterations: 2,
            trace_tasks: true,
            ..Default::default()
        },
    )
    .expect("task trace should run");
    assert!(
        result.logs.iter().any(|l| l.contains("trace-task:")),
        "expected task trace logs, got {:?}",
        result.logs
    );
    let task = result
        .metrics
        .tasks
        .get("SafetyMonitor")
        .expect("SafetyMonitor metrics");
    assert_eq!(task.priority, "critical");
    assert!(task.ticks >= 2);
}

#[test]
fn replay_trace_logs_twin_frames() {
    let source = r#"
robot R {
  actuator wheels: DifferentialDrive;
  twin Shadow { mirror pose; replay true; }
  safety { max_speed = 1.0 m/s; }
  task sync every 50ms { wheels.drive(linear: 0.1 m/s, angular: 0.0 rad/s); }
}
"#;
    let result = run(
        source,
        RunOptions {
            max_loop_iterations: 2,
            replay_trace: true,
            ..Default::default()
        },
    )
    .expect("replay trace should run");
    assert!(
        result.metrics.replay_frames >= 1,
        "expected replay frames, got {}",
        result.metrics.replay_frames
    );
}
