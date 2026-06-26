//! Integration tests for operational policy evaluation.

use spanda_lexer::tokenize;
use spanda_parser::parse;
use spanda_policy::{evaluate_policy, evaluate_readiness_with_policy, list_policies};
use spanda_readiness::{evaluate_readiness, ReadinessOptions};
use std::path::PathBuf;

fn repo_path(parts: &[&str]) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../..");
    for part in parts {
        path.push(part);
    }
    path
}

fn parse_file(path: PathBuf) -> spanda_ast::nodes::Program {
    let source = std::fs::read_to_string(&path).unwrap();
    let tokens = tokenize(&source).unwrap();
    parse(tokens).unwrap()
}

#[test]
fn warehouse_policy_declared_and_lists() {
    let program = parse_file(repo_path(&[
        "examples",
        "showcase",
        "policy",
        "warehouse.sd",
    ]));
    assert_eq!(list_policies(&program), vec!["WarehousePolicy".to_string()]);
}

#[test]
fn warehouse_policy_passes_showcase_program() {
    let program = parse_file(repo_path(&[
        "examples",
        "showcase",
        "policy",
        "warehouse.sd",
    ]));
    let report = evaluate_policy(&program, "WarehousePolicy", "warehouse.sd").unwrap();
    assert!(report.passed, "{:?}", report.violations);
}

#[test]
fn readiness_rover_fails_requires_kill_switch_policy() {
    let program = parse_file(repo_path(&[
        "examples",
        "showcase",
        "readiness",
        "rover.sd",
    ]));
    let tokens =
        tokenize("policy StrictOps { requires_kill_switch; min_readiness_score = 50; }").unwrap();
    let policy_program = parse(tokens).unwrap();
    let Program::Program {
        operational_policies,
        ..
    } = policy_program;
    let mut merged = program.clone();
    let Program::Program {
        operational_policies: ref mut target,
        ..
    } = merged;
    *target = operational_policies;
    let report = evaluate_policy(&merged, "StrictOps", "rover.sd").unwrap();
    assert!(!report.passed);
    assert!(report
        .violations
        .iter()
        .any(|v| v.rule == "requires_kill_switch"));
}

#[test]
fn readiness_merges_operational_policy_factor() {
    let program = parse_file(repo_path(&[
        "examples",
        "showcase",
        "policy",
        "warehouse.sd",
    ]));
    let base = evaluate_readiness(&program, &ReadinessOptions::default());
    let (merged, policy_report) = evaluate_readiness_with_policy(
        &program,
        "warehouse.sd",
        &ReadinessOptions::default(),
        "WarehousePolicy",
        base,
    )
    .unwrap();
    assert!(policy_report.passed);
    assert!(merged
        .score
        .factors
        .iter()
        .any(|factor| factor.factor == "OperationalPolicy"));
}

use spanda_ast::nodes::Program;
