//! OTA execute rolls back when post-deploy readiness fails.

use spanda_ota::{
    agent_entry_for_port, build_deploy_bundle, deploy_target_key, execute_remote_rollout,
    register_agent, save_agent_registry, spawn_test_agent, DeployAgentRegistry, DeployPlan,
    RolloutOptions, RolloutStepStatus, RolloutStrategy,
};
use std::thread;
use std::time::Duration;

#[test]
fn execute_remote_rollout_rolls_back_on_readiness_fail() {
    let target = deploy_target_key("RoverProgram", "JetsonOrin");
    let (port, _handle) = spawn_test_agent(&target, None).expect("spawn test agent");
    thread::sleep(Duration::from_millis(100));

    let agents_path = std::env::temp_dir().join(format!(
        "spanda-ota-readiness-rollback-{}.json",
        std::process::id()
    ));
    let mut registry = DeployAgentRegistry::default();
    let entry = agent_entry_for_port(&target, port, None);
    register_agent(&mut registry, target.clone(), entry.url, None).expect("register agent");
    save_agent_registry(&agents_path, &registry).expect("save registry");
    std::env::set_var(
        "SPANDA_DEPLOY_AGENTS",
        agents_path.to_string_lossy().to_string(),
    );

    let plan = DeployPlan {
        program: "rollback-test".into(),
        version: "9.9.9".into(),
        program_hash: None,
        assignments: vec![spanda_ota::DeployAssignment {
            robot_name: "RoverProgram".into(),
            hardware: "JetsonOrin".into(),
        }],
        certifications: vec![],
        certification_proof: None,
    };
    let bundle = build_deploy_bundle(&plan);
    let options = RolloutOptions {
        strategy: RolloutStrategy::All,
        version: "9.9.9".into(),
        dry_run: false,
        rollback_on_readiness_fail: true,
        readiness_runtime: true,
        readiness_inject_faults: true,
        ..RolloutOptions::default()
    };
    let result = execute_remote_rollout(&plan, &options, &registry, &bundle);
    assert!(
        result
            .steps
            .iter()
            .any(|step| step.status == RolloutStepStatus::RolledBack),
        "expected rolled_back step, got {:?}",
        result.steps
    );
    assert!(!result.success);
    let _ = std::fs::remove_file(agents_path);
}
