//! Integration tests for Recovery Orchestrator.
//!
use spanda_config::entity::{EntityHealthStatus, EntityKind, EntityRecord, EntityRegistry};
use spanda_lexer::tokenize;
use spanda_parser::parse;
use spanda_recovery::{
    build_recovery_graph, decide_recovery, default_playbooks, evaluate_policy, load_playbooks,
    match_playbooks, OrchestratorStrategy, RecoveryEscalationLevel, RecoveryHistoryStore,
    RecoveryOrchestrator, RecoveryOrchestratorRequest, RecoverySimulationMode,
};

const ROVER: &str = include_str!("../../../examples/showcase/self_healing/rover.sd");

fn parse_rover() -> spanda_ast::nodes::Program {
    parse(tokenize(ROVER).unwrap()).unwrap()
}

fn sample_registry() -> EntityRegistry {
    let mut registry = EntityRegistry::default();
    registry.entities.insert(
        "robot-1".into(),
        EntityRecord {
            id: "robot-1".into(),
            entity_type: EntityKind::Robot,
            health_status: EntityHealthStatus::Degraded,
            ..Default::default()
        },
    );
    registry.entities.insert(
        "camera-1".into(),
        EntityRecord {
            id: "camera-1".into(),
            entity_type: EntityKind::Camera,
            parent_id: Some("robot-1".into()),
            health_status: EntityHealthStatus::Warning,
            ..Default::default()
        },
    );
    registry
}

#[test]
fn recovery_planning_produces_orchestrated_plan() {
    let program = parse_rover();
    let registry = sample_registry();
    let orchestrator = RecoveryOrchestrator::new();
    let request = RecoveryOrchestratorRequest {
        entity_id: Some("robot-1".into()),
        failure: Some("gps_loss".into()),
        mode: RecoverySimulationMode::Plan,
        ..Default::default()
    };
    let report = orchestrator.plan_recovery(&program, &registry, None, &request);
    assert!(!report.plans.is_empty());
    assert!(report.plans[0].decision.can_recover);
}

#[test]
fn dependency_graph_includes_entities() {
    let registry = sample_registry();
    let graph = build_recovery_graph(&registry, Some("robot-1"));
    assert!(!graph.nodes.is_empty());
    assert!(graph.nodes.iter().any(|n| n.id == "robot-1"));
}

#[test]
fn policy_evaluation_respects_max_level() {
    let policy = spanda_recovery::EntityRecoveryPolicy {
        entity_id: "robot-1".into(),
        max_escalation_level: RecoveryEscalationLevel::Level2RestartPackage,
        ..Default::default()
    };
    let (ok, _) = evaluate_policy(
        &policy,
        &OrchestratorStrategy::RestartRobot,
        RecoveryEscalationLevel::Level4RecoverRobot,
    );
    assert!(!ok);
}

#[test]
fn simulation_mode_generates_evidence() {
    let program = parse_rover();
    let registry = sample_registry();
    let orchestrator = RecoveryOrchestrator::new();
    let request = RecoveryOrchestratorRequest {
        entity_id: Some("robot-1".into()),
        failure: Some("sensor_failure".into()),
        mode: RecoverySimulationMode::Simulate,
        ..Default::default()
    };
    let report = orchestrator.simulate_recovery(&program, &registry, None, &request, None);
    assert!(!report.evidence.is_empty() || !report.plans.is_empty());
}

#[test]
fn playbook_matching_finds_sensor_playbook() {
    let playbooks = default_playbooks();
    let matched = match_playbooks(&playbooks, "lidar sensor failed");
    assert!(!matched.is_empty());
    assert_eq!(matched[0].name, "sensor_failure");
}

#[test]
fn decision_engine_explains_recovery() {
    let registry = sample_registry();
    let entity = registry.get("robot-1").unwrap();
    let decision = decide_recovery(entity, "gps_loss", &[], &registry, None);
    assert!(decision.can_recover);
    assert!(!decision.explanations.is_empty());
}

#[test]
fn predictive_indicators_from_degraded_health() {
    let registry = sample_registry();
    let indicators = spanda_recovery::scan_predictive_indicators(&registry, None);
    assert!(!indicators.is_empty());
}

#[test]
fn metrics_from_history() {
    let history = RecoveryHistoryStore::default();
    let orchestrator = RecoveryOrchestrator::new();
    let metrics = orchestrator.get_metrics(&parse_rover());
    assert!(metrics.recovery_confidence >= 0.0);
    let _ = history;
}

#[test]
fn plugin_registry_accepts_extensions() {
    let mut registry = spanda_recovery::RecoveryPluginRegistry::new();
    registry.register(spanda_recovery::PluginRecoveryExtension {
        plugin_id: "test-plugin".into(),
        extension_kind: "strategy".into(),
        name: "custom_retry".into(),
        description: "Custom retry strategy".into(),
    });
    assert!(registry.resolve_strategy("custom_retry").is_some());
}

#[test]
fn load_playbooks_includes_defaults() {
    let playbooks = load_playbooks(None);
    assert!(playbooks.iter().any(|p| p.name == "battery_low"));
}
