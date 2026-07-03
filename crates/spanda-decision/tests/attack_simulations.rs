//! Live attack simulation integration tests.

use spanda_decision::{
    run_attack_simulation, AttackScenario,
};

#[test]
fn attack_policy_tampering_blocked() {
    let result = run_attack_simulation(AttackScenario::PolicyTampering);
    assert!(result.blocked, "evidence: {:?}", result.evidence);
    assert_eq!(result.scenario, AttackScenario::PolicyTampering);
}

#[test]
fn attack_replayed_decision_blocked() {
    let result = run_attack_simulation(AttackScenario::ReplayedDecision);
    assert!(result.blocked);
    assert_eq!(
        result.evidence["replay_rejected"].as_bool(),
        Some(true)
    );
}

#[test]
fn attack_fake_coordinator_blocked() {
    let result = run_attack_simulation(AttackScenario::FakeCoordinator);
    assert!(result.blocked);
}

#[test]
fn attack_offline_abuse_blocked() {
    let result = run_attack_simulation(AttackScenario::OfflineAbuse);
    assert!(result.blocked);
    assert_eq!(
        result.evidence["duration_exceeded_blocked"].as_bool(),
        Some(true)
    );
    assert_eq!(
        result.evidence["forbidden_action_blocked"].as_bool(),
        Some(true)
    );
}

#[test]
fn attack_split_brain_safety_wins() {
    let result = run_attack_simulation(AttackScenario::SplitBrainCoordinator);
    assert!(result.blocked);
    assert_eq!(
        result.evidence["winner_action"].as_str(),
        Some("emergency_stop")
    );
}

#[test]
fn attack_compromised_robot_blocked() {
    let result = run_attack_simulation(AttackScenario::CompromisedRobot);
    assert!(result.blocked);
}

#[test]
fn attack_poisoned_telemetry_blocked() {
    let result = run_attack_simulation(AttackScenario::PoisonedTelemetry);
    assert!(result.blocked);
}

#[test]
fn all_attack_scenarios_produce_evidence() {
    let scenarios = [
        AttackScenario::PolicyTampering,
        AttackScenario::FakeCoordinator,
        AttackScenario::ReplayedDecision,
        AttackScenario::CompromisedRobot,
        AttackScenario::PoisonedTelemetry,
        AttackScenario::OfflineAbuse,
        AttackScenario::SplitBrainCoordinator,
    ];
    for scenario in scenarios {
        let result = run_attack_simulation(scenario.clone());
        assert!(
            result.evidence.is_object(),
            "scenario {:?} missing evidence",
            scenario
        );
    }
}
