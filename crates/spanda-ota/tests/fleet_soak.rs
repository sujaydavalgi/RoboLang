//! Production-style OTA fleet soak — multi-agent version bumps across deploy agents.

use spanda_driver::{build_deploy_plan, compile};
use spanda_ota::{
    agent_entry_for_port, agent_status, build_deploy_bundle, deploy_target_key,
    execute_remote_rollout, register_agent, save_agent_registry, sign_deploy_bundle,
    spawn_test_agent, DeployAgentRegistry, RolloutOptions, RolloutStrategy,
};
use std::thread;
use std::time::Duration;

fn spawn_fleet_agent(robot: &str, hardware: &str) -> spanda_ota::DeployAgentEntry {
    let target = deploy_target_key(robot, hardware);
    let (port, _handle) = spawn_test_agent(&target, None).expect("spawn test agent");
    thread::sleep(Duration::from_millis(50));
    agent_entry_for_port(&target, port, None)
}

fn build_fleet_plan(version: &str) -> spanda_ota::DeployPlan {
    let source = include_str!("../../../examples/robotics/ota_deployment.sd");
    let program = compile(source)
        .expect("compile ota deployment program")
        .program;
    let program_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/robotics/ota_deployment.sd"
    );
    let mut plan = build_deploy_plan(&program, program_path, version);
    plan.assignments = vec![
        spanda_ota::DeployAssignment {
            robot_name: "RoverAlpha".into(),
            hardware: "JetsonOrin".into(),
        },
        spanda_ota::DeployAssignment {
            robot_name: "RoverBeta".into(),
            hardware: "JetsonOrin".into(),
        },
    ];
    plan
}

#[test]
fn fleet_soak_multi_agent_version_bumps() {
    let entry_a = spawn_fleet_agent("RoverAlpha", "JetsonOrin");
    let entry_b = spawn_fleet_agent("RoverBeta", "JetsonOrin");

    let agents_path = std::env::temp_dir().join(format!(
        "spanda-fleet-soak-agents-{}.json",
        std::process::id()
    ));
    let mut registry = DeployAgentRegistry::default();
    register_agent(
        &mut registry,
        entry_a.target.clone(),
        entry_a.url.clone(),
        None,
    )
    .expect("register alpha");
    register_agent(
        &mut registry,
        entry_b.target.clone(),
        entry_b.url.clone(),
        None,
    )
    .expect("register beta");
    save_agent_registry(&agents_path, &registry).expect("save registry");
    std::env::set_var(
        "SPANDA_DEPLOY_AGENTS",
        agents_path.to_string_lossy().to_string(),
    );

    let versions = ["4.0.0", "4.1.0", "4.2.0"];
    for version in versions {
        let plan = build_fleet_plan(version);
        let mut bundle = build_deploy_bundle(&plan);
        sign_deploy_bundle(&mut bundle, "fleet-soak-signing-key").expect("sign bundle");
        let result = execute_remote_rollout(
            &plan,
            &RolloutOptions {
                strategy: RolloutStrategy::All,
                version: version.into(),
                dry_run: false,
                ..RolloutOptions::default()
            },
            &registry,
            &bundle,
        );
        assert!(
            result.success,
            "fleet soak rollout {version} failed: {:?}",
            result.steps
        );
        for entry in [&entry_a, &entry_b] {
            let status = agent_status(entry).expect("status");
            assert_eq!(status.current_version, version);
        }
    }

    let _ = std::fs::remove_file(agents_path);
}

#[test]
fn fleet_soak_staged_canary_progression() {
    let entry_a = spawn_fleet_agent("RoverAlpha", "JetsonOrin");
    let entry_b = spawn_fleet_agent("RoverBeta", "JetsonOrin");

    let mut registry = DeployAgentRegistry::default();
    register_agent(
        &mut registry,
        entry_a.target.clone(),
        entry_a.url.clone(),
        None,
    )
    .expect("register alpha");
    register_agent(
        &mut registry,
        entry_b.target.clone(),
        entry_b.url.clone(),
        None,
    )
    .expect("register beta");

    let plan = build_fleet_plan("5.0.0");
    let bundle = build_deploy_bundle(&plan);

    let canary = execute_remote_rollout(
        &plan,
        &RolloutOptions {
            strategy: RolloutStrategy::Canary,
            canary_percent: 50,
            version: "5.0.0".into(),
            dry_run: false,
            ..RolloutOptions::default()
        },
        &registry,
        &bundle,
    );
    assert!(canary.success);
    assert_eq!(canary.steps.len(), 2);
    let deployed = canary
        .steps
        .iter()
        .filter(|step| step.status == spanda_ota::RolloutStepStatus::Deployed)
        .count();
    assert_eq!(deployed, 1, "canary should deploy one of two agents");

    let full = execute_remote_rollout(
        &plan,
        &RolloutOptions {
            strategy: RolloutStrategy::All,
            version: "5.0.0".into(),
            dry_run: false,
            ..RolloutOptions::default()
        },
        &registry,
        &bundle,
    );
    assert!(full.success);
    assert!(full
        .steps
        .iter()
        .all(|step| step.status == spanda_ota::RolloutStepStatus::Deployed));
}
