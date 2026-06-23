//! Phase 29 verification, health runtime, and kill-switch integration tests.

use spanda_capability::collect_verification_diagnostics;
use spanda_core::{run, RunOptions};
use spanda_lexer::tokenize;
use spanda_parser::parse;

fn parse_source(source: &str) -> spanda_ast::nodes::Program {
    parse(tokenize(source).expect("tokenize")).expect("parse")
}

#[test]
fn remote_kill_switch_emits_verification_warning() {
    let source = r#"
kill_switch EmergencyStop {
    priority: critical;
    remote_signed;
    action { emergency_stop; }
}

robot Rover {
    actuator wheels: DifferentialDrive;
    safety { max_speed = 1.0 m/s; }
    behavior run() { wheels.stop(); }
}
"#;
    let program = parse_source(source);
    let diags = collect_verification_diagnostics(&program);
    assert!(
        diags
            .iter()
            .any(|d| d.category == "kill-switch" && d.message.contains("remote_signed")),
        "expected remote_signed kill switch diagnostic, got {diags:?}"
    );
    assert!(
        diags.iter().any(|d| d.severity == "error"),
        "expected error severity for unsigned remote kill switch policy"
    );
}

#[test]
fn trigger_kill_switch_sets_emergency_stop() {
    let source = r#"
kill_switch EmergencyStop {
    priority: critical;
    action { emergency_stop; }
}

robot Rover {
    actuator wheels: DifferentialDrive;
    safety { max_speed = 1.0 m/s; }
    behavior run() { wheels.stop(); }
}
"#;
    let result = run(
        source,
        RunOptions {
            trigger_kill_switch: Some("EmergencyStop".into()),
            ..Default::default()
        },
    )
    .expect("sim should run");
    assert!(
        result.state.emergency_stop,
        "kill switch trigger should set emergency_stop"
    );
    assert!(
        result
            .logs
            .iter()
            .any(|l| l.contains("kill_switch: activated")),
        "expected kill switch log, got {:?}",
        result.logs
    );
}
