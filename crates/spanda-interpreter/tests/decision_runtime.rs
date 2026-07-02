//! Distributed decision trace emission and tree action dispatch integration tests.

use spanda_decision::DecisionBackedRuntime;
use spanda_interpreter::{run_program, RunOptions};
use spanda_lexer::tokenize;
use spanda_parser::parse;
use spanda_runtime::replay::{MissionTrace, TraceFrame};
use std::sync::Arc;

fn decision_frames(trace: &MissionTrace) -> Vec<&TraceFrame> {
    trace
        .frames
        .iter()
        .filter(|f| f.payload.get("version").and_then(|v| v.as_u64()) == Some(3))
        .collect()
}

fn run_with_decision_trace(source: &str, options: RunOptions) -> MissionTrace {
    let trace_path =
        std::env::temp_dir().join(format!("spanda_decision_runtime_{}.trace", std::process::id()));
    let _ = std::fs::remove_file(&trace_path);
    let program = parse(tokenize(source).unwrap()).unwrap();
    let mut opts = options;
    opts.record_trace = true;
    opts.decision_trace = true;
    opts.trace_output = Some(trace_path.to_string_lossy().into_owned());
    opts.decision_runtime = Some(Arc::new(DecisionBackedRuntime));
    let result = run_program(&program, opts).expect("run");
    let trace = result
        .mission_trace
        .or_else(|| MissionTrace::load(&trace_path).ok())
        .expect("mission trace");
    let _ = std::fs::remove_file(&trace_path);
    trace
}

#[test]
fn decision_tree_emits_v3_trace_on_health_fault_injection() {
    let source = r#"
hardware H {
    sensors [GPS];
    actuators [DifferentialDrive];
}

decision_tree GPSLossRecovery local {
    when gps.status == Failed {
        else {
            pause_mission;
            enter safe_mode;
        }
    }
}

robot Rover {
    sensor gps: GPS;
    actuator wheels: DifferentialDrive;
    mode safe { }
    safety { max_speed = 1.0 m/s; }
    behavior patrol() {
        loop every 50ms {
            let _ = gps.read();
        }
    }
}
"#;
    let trace = run_with_decision_trace(
        source,
        RunOptions {
            inject_health_faults: true,
            max_loop_iterations: 2,
            ..Default::default()
        },
    );
    let v3 = decision_frames(&trace);
    assert!(
        v3.iter()
            .any(|f| f.event == "decision_tree_eval" && f.payload["layer"] == "local_entity"),
        "expected decision_tree_eval v3 frame, got events: {:?}",
        trace.frames.iter().map(|f| &f.event).collect::<Vec<_>>()
    );
}

#[test]
fn kill_switch_emits_reflex_decision_trace() {
    let source = r#"
kill_switch EStop {
    priority: critical;
    action {
        stop_all_actuators();
    }
}

robot Rover {
    sensor gps: GPS;
    actuator wheels: DifferentialDrive;
    safety { max_speed = 1.0 m/s; }
    behavior patrol() {
        loop every 50ms {
            let _ = gps.read();
        }
    }
}
"#;
    let trace = run_with_decision_trace(
        source,
        RunOptions {
            trigger_kill_switch: Some("EStop".into()),
            max_loop_iterations: 2,
            ..Default::default()
        },
    );
    assert!(
        decision_frames(&trace)
            .iter()
            .any(|f| f.event == "kill_switch_activated" && f.payload["layer"] == "reflex"),
        "expected kill_switch_activated reflex trace"
    );
}

#[test]
fn decision_tree_actions_dispatch_degraded_mode() {
    let source = r#"
hardware H {
    sensors [GPS];
    actuators [DifferentialDrive];
}

decision_tree GPSLossRecovery local {
    when gps.status == Failed {
        enter degraded_mode;
        reduce_speed 0.4 m/s;
    }
}

robot Rover {
    sensor gps: GPS;
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
    let program = parse(tokenize(source).unwrap()).unwrap();
    let result = run_program(
        &program,
        RunOptions {
            inject_health_faults: true,
            decision_trace: true,
            decision_runtime: Some(Arc::new(DecisionBackedRuntime)),
            max_loop_iterations: 2,
            ..Default::default()
        },
    )
    .expect("run");
    assert!(
        result.logs.iter().any(|l| l.contains("mode: entered") && l.contains("degraded")),
        "expected degraded mode entry from decision tree actions, logs: {:?}",
        result.logs
    );
}

#[test]
fn offline_policy_blocks_forbidden_action_at_runtime() {
    let source = r#"
robot Rover {
    local_decision_authority [pause_mission];
    sensor gps: GPS;
    actuator wheels: DifferentialDrive;
    mode safe { }
    safety { max_speed = 1.0 m/s; }
    behavior patrol() {
        loop every 50ms { let _ = gps.read(); }
    }
}

offline_policy RoverOffline {
    max_duration = 30 min;
    allowed_actions [pause_mission];
    forbidden_actions [disable_safety];
}

decision_tree OfflineBlock local {
    when gps.status == Failed {
        disable_safety;
    }
}
"#;
    std::env::set_var("SPANDA_CENTRAL_CONNECTED", "0");
    std::env::set_var("SPANDA_OFFLINE_MINUTES", "5");
    let program = parse(tokenize(source).unwrap()).unwrap();
    let result = run_program(
        &program,
        RunOptions {
            inject_health_faults: true,
            decision_trace: true,
            record_trace: true,
            decision_runtime: Some(Arc::new(DecisionBackedRuntime)),
            max_loop_iterations: 2,
            ..Default::default()
        },
    )
    .expect("run");
    assert!(
        result
            .logs
            .iter()
            .any(|l| l.contains("decision_action_blocked") || l.contains("blocked 'disable_safety'")),
        "expected offline policy block, logs: {:?}",
        result.logs
    );
    std::env::remove_var("SPANDA_CENTRAL_CONNECTED");
    std::env::remove_var("SPANDA_OFFLINE_MINUTES");
}

#[test]
fn central_approval_blocks_until_escalation_granted() {
    let source = r#"
robot Rover {
    local_decision_authority [degraded_mode];
    requires_central_approval [update_firmware];
    sensor gps: GPS;
    actuator wheels: DifferentialDrive;
    mode degraded { }
    safety { max_speed = 1.0 m/s; }
    behavior patrol() {
        loop every 50ms { let _ = gps.read(); }
    }
}

decision_tree FirmwareGate local {
    when gps.status == Failed {
        update_firmware;
    }
}
"#;
    std::env::set_var("SPANDA_CENTRAL_CONNECTED", "1");
    let program = parse(tokenize(source).unwrap()).unwrap();
    let blocked = run_program(
        &program,
        RunOptions {
            inject_health_faults: true,
            decision_trace: true,
            decision_runtime: Some(Arc::new(DecisionBackedRuntime)),
            max_loop_iterations: 2,
            ..Default::default()
        },
    )
    .expect("run");
    assert!(
        blocked.logs.iter().any(|l| l.contains("decision_escalation_pending")
            || l.contains("blocked 'update_firmware'")),
        "expected escalation pending, logs: {:?}",
        blocked.logs
    );
    std::env::set_var("SPANDA_DECISION_ESCALATION_APPROVED", "1");
    let approved = run_program(
        &program,
        RunOptions {
            inject_health_faults: true,
            decision_trace: true,
            decision_runtime: Some(Arc::new(DecisionBackedRuntime)),
            max_loop_iterations: 2,
            ..Default::default()
        },
    )
    .expect("run");
    assert!(
        approved
            .logs
            .iter()
            .any(|l| l.contains("recovery: recorded action 'update_firmware'")),
        "expected action after escalation approval, logs: {:?}",
        approved.logs
    );
    std::env::remove_var("SPANDA_CENTRAL_CONNECTED");
    std::env::remove_var("SPANDA_DECISION_ESCALATION_APPROVED");
}
