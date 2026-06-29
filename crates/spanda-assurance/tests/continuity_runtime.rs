//! Runtime continuity takeover dispatch integration tests.

use spanda_assurance::AssuranceBackedRuntime;
use spanda_interpreter::{execute_continuity_on_program, ContinuityRunOptions};
use spanda_lexer::tokenize;
use spanda_parser::parse;
use spanda_runtime::{parse_trigger, ContinuityContext, SuccessionScope, TakeoverMode};
use std::sync::Arc;

fn assurance_continuity_options() -> ContinuityRunOptions {
    ContinuityRunOptions {
        assurance_runtime: Some(Arc::new(AssuranceBackedRuntime)),
        ..Default::default()
    }
}

fn warehouse_program() -> spanda_ast::nodes::Program {
    let source = include_str!("../../../examples/showcase/continuity/warehouse.sd");
    parse(tokenize(source).unwrap()).unwrap()
}

#[test]
fn continuity_runtime_resumes_successor_at_checkpoint() {
    let program = warehouse_program();
    let context = ContinuityContext {
        mission: "WarehouseInventoryScan".into(),
        failed_entity: "ScannerAlpha".into(),
        trigger: parse_trigger("robot_failed"),
        progress_percent: 72.0,
        scope: SuccessionScope::Fleet,
        current_step: None,
        checkpoints: Vec::new(),
    };
    let checkpoint_path = std::env::temp_dir().join("spanda-continuity-runtime-test.json");
    std::env::set_var(
        "SPANDA_CONTINUITY_CHECKPOINTS",
        checkpoint_path.to_string_lossy().as_ref(),
    );
    let outcome = execute_continuity_on_program(
        &program,
        &context,
        ContinuityRunOptions {
            robot_name: Some("ScannerAlpha".into()),
            successor: Some("ScannerBeta".into()),
            ..assurance_continuity_options()
        },
    )
    .expect("continuity execution");
    assert!(outcome.takeover.succeeded);
    assert!(
        outcome
            .logs
            .iter()
            .any(|line| line.contains("checkpoint persisted")),
        "expected checkpoint persistence log, got: {:?}",
        outcome.logs
    );
    let successor_outcome = execute_continuity_on_program(
        &program,
        &context,
        ContinuityRunOptions {
            robot_name: Some("ScannerBeta".into()),
            successor: Some("ScannerBeta".into()),
            ..assurance_continuity_options()
        },
    )
    .expect("successor continuity");
    assert!(
        successor_outcome
            .logs
            .iter()
            .any(|line| line.contains("resuming") || line.contains("resume")),
        "expected successor resume log, got: {:?}",
        successor_outcome.logs
    );
    assert!(successor_outcome.checkpoint_count >= 1);
    std::env::remove_var("SPANDA_CONTINUITY_CHECKPOINTS");
    let _ = std::fs::remove_file(checkpoint_path);
}

#[test]
fn continuity_runtime_restart_mode_restarts_mission() {
    let source = r#"
mission_plan Patrol {
    step alpha;
    step beta;
}

continuity_policy PatrolContinuity {
    on battery.critical {
        restart mission;
    }
}

robot RoverA {
    behavior patrol() { }
}

robot RoverB {
    behavior patrol() { }
}
"#;
    let program = parse(tokenize(source).unwrap()).unwrap();
    let context = ContinuityContext {
        mission: "Patrol".into(),
        failed_entity: "RoverA".into(),
        trigger: parse_trigger("battery_critical"),
        progress_percent: 40.0,
        scope: SuccessionScope::Robot,
        current_step: None,
        checkpoints: Vec::new(),
    };
    let outcome = execute_continuity_on_program(
        &program,
        &context,
        ContinuityRunOptions {
            robot_name: Some("RoverB".into()),
            successor: Some("RoverB".into()),
            ..assurance_continuity_options()
        },
    )
    .expect("restart continuity");
    assert!(
        outcome
            .logs
            .iter()
            .any(|line| line.contains("restarted") || line.contains("restart")),
        "expected restart log, got: {:?}",
        outcome.logs
    );
}

#[test]
fn continuity_runtime_human_takeover_requires_approval() {
    let source = r#"
mission_plan Patrol { step alpha; }

continuity_policy HumanContinuity {
    on robot.failed { human takeover; }
}

robot RoverA { behavior patrol() { } }
robot RoverB { behavior patrol() { } }
"#;
    let program = parse(tokenize(source).unwrap()).unwrap();
    let context = ContinuityContext {
        mission: "Patrol".into(),
        failed_entity: "RoverA".into(),
        trigger: parse_trigger("robot_failed"),
        progress_percent: 72.0,
        scope: SuccessionScope::Robot,
        current_step: None,
        checkpoints: Vec::new(),
    };
    std::env::remove_var("SPANDA_OPERATOR_APPROVAL");
    let blocked = execute_continuity_on_program(
        &program,
        &context,
        ContinuityRunOptions {
            robot_name: Some("RoverB".into()),
            successor: Some("RoverB".into()),
            grant_operator_approval: false,
            ..assurance_continuity_options()
        },
    )
    .expect("human takeover without approval");
    assert_eq!(
        blocked.takeover.mode,
        TakeoverMode::HumanTakeover
    );
    assert!(
        blocked
            .logs
            .iter()
            .any(|line| line.contains("awaiting operator approval")),
        "expected approval gate log, got: {:?}",
        blocked.logs
    );
}

#[test]
fn continuity_auto_triggers_during_run_on_health_fault() {
    let source = r#"
mission_plan Patrol {
    step alpha;
    step beta;
}

continuity_policy FleetContinuity {
    on robot.failed {
        resume from checkpoint;
        reassign mission;
    }
}

fleet PatrolFleet {
    RoverA;
    RoverB;
}

robot RoverA {
    behavior patrol() {
        loop every 50ms {
            let _ = 1;
        }
    }
}

robot RoverB {
    behavior patrol() {
        loop every 50ms {
            let _ = 1;
        }
    }
}
"#;
    let program = parse(tokenize(source).unwrap()).unwrap();
    let result = spanda_interpreter::run_program(
        &program,
        spanda_interpreter::RunOptions {
            inject_health_faults: true,
            max_loop_iterations: 2,
            assurance_runtime: Some(Arc::new(AssuranceBackedRuntime)),
            ..Default::default()
        },
    )
    .expect("run with continuity auto-trigger");
    assert!(
        result
            .logs
            .iter()
            .any(|line| line.contains("continuity: auto-triggered")),
        "expected auto continuity log, got: {:?}",
        result.logs
    );
}
