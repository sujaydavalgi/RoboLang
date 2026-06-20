use spanda_core::{check, run, run_tests, RunOptions};

#[test]
fn serialize_json_round_trip() {
    let source = r#"
module telemetry;

export fn snapshot() -> Pose {
  return pose(x: 1.0 m, y: 2.0 m, theta: 0.0 rad);
}

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let p = snapshot();
    let data = serialize(p, "json");
    let restored = deserialize(data, "json");
    let _ = restored;
    wheels.stop();
  }
}
"#;
    check(source).expect("serialize/deserialize should type-check");
    run(source, RunOptions::default()).expect("serialize should run");
}

#[test]
fn in_language_test_block_runs() {
    let source = r#"
module math;

export fn double(x: Int) -> Int {
  return x;
}

test "double returns input" {
  assert(true);
}
"#;
    let result = run_tests(source).expect("tests should execute");
    assert_eq!(result.passed, 1);
    assert_eq!(result.failed, 0);
}

#[test]
fn async_await_module_function() {
    let source = r#"
module maps;

export async fn get_map() -> Pose {
  return pose(x: 0.0 m, y: 0.0 m, theta: 0.0 rad);
}

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let map = await get_map();
    let _ = map;
    wheels.stop();
  }
}
"#;
    check(source).expect("async/await should type-check");
    run(source, RunOptions::default()).expect("async/await should run");
}

#[test]
fn spawn_channel_select() {
    let source = r#"
module comm;

export fn ping() -> Int {
  return 1;
}

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let ch = channel();
    send(ch, 42);
    select {
      recv(ch) => wheels.stop();
    };
    spawn ping();
  }
}
"#;
    check(source).expect("spawn/channel/select should type-check");
    run(source, RunOptions::default()).expect("concurrency primitives should run");
}

#[test]
fn typed_channel_rejects_mismatched_payloads() {
    let source = r#"
module comm;

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let ch = channel();
    send(ch, 42);
    send(ch, "bad");
    wheels.stop();
  }
}
"#;
    let err = check(source).expect_err("type checker should reject mismatched channel payload");
    assert!(
        err.diagnostics()
            .iter()
            .any(|d| d.message.contains("Type mismatch")),
        "expected channel payload type mismatch, got {:?}",
        err.diagnostics()
    );
}

#[test]
fn priority_task_without_every_is_allowed() {
    let source = r#"
robot R {
  actuator wheels: DifferentialDrive;
  task SafetyMonitor critical {
    wheels.stop();
  }
}
"#;
    check(source).expect("priority task without explicit period should type-check");
}

#[test]
fn parallel_block_runs_and_waits_for_spawned_calls() {
    let source = r#"
module comm;

export fn perception() -> Int { return 1; }
export fn planning() -> Int { return 2; }

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    parallel {
      perception();
      planning();
    };
    wheels.stop();
  }
}
"#;
    check(source).expect("parallel block should type-check");
    let result = run(source, RunOptions::default()).expect("parallel block should run");
    assert!(
        result
            .logs
            .iter()
            .any(|l| l.contains("parallel: executing")),
        "expected parallel execution log, got {:?}",
        result.logs
    );
}

#[test]
fn join_future_returns_inner_value() {
    let source = r#"
module comm;

export async fn ping() -> Int { return 7; }

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let f = ping();
    let v = join(f);
    let _ = v;
    wheels.stop();
  }
}
"#;
    check(source).expect("join should type-check for Future");
    run(source, RunOptions::default()).expect("join should resolve Future");
}

#[test]
fn spawn_handle_join_returns_result() {
    let source = r#"
module comm;

export fn ping() -> Int { return 7; }

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let h = spawn ping();
    let v = join(h);
    let _ = v;
    wheels.stop();
  }
}
"#;
    check(source).expect("spawn handle should type-check");
    run(source, RunOptions::default()).expect("join should resolve TaskHandle");
}

#[test]
fn parallel_aggregates_spawn_handles() {
    let source = r#"
module comm;

export fn a() -> Int { return 1; }
export fn b() -> Int { return 2; }

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    parallel {
      let left = spawn a();
      let right = spawn b();
    };
    let _ = _parallel;
    wheels.stop();
  }
}
"#;
    check(source).expect("parallel spawn aggregation should type-check");
    let result = run(source, RunOptions::default()).expect("parallel aggregation should run");
    assert!(
        result
            .logs
            .iter()
            .any(|l| l.contains("parallel: aggregated 2 result")),
        "expected parallel aggregation log, got {:?}",
        result.logs
    );
}
