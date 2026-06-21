//! Remote fleet peer relay integration tests.

use spanda_core::{
    compile, fleet_entry_for_port, load_fleet_agent_registry, orchestrate_fleets_remote,
    register_fleet_agent, relay_peer_deliveries, save_fleet_agent_registry, spawn_test_fleet_agent,
    FleetAgentRegistry, PeerDelivery,
};
use std::path::PathBuf;
use std::time::Duration;

#[test]
fn relays_peer_delivery_to_registered_fleet_agent() {
    let (port, _handle) = spawn_test_fleet_agent("ScoutB", None).expect("spawn fleet agent");
    let mut registry = FleetAgentRegistry::default();
    register_fleet_agent(
        &mut registry,
        "ScoutB".into(),
        format!("http://127.0.0.1:{port}"),
        None,
    )
    .expect("register fleet agent");

    let deliveries = vec![PeerDelivery {
        from_robot: "ScoutA".into(),
        to_robot: "ScoutB".into(),
        topic: "mission_step".into(),
        step: "inspect".into(),
        delivered: true,
    }];
    let (relayed, failed) = relay_peer_deliveries(&deliveries, &registry);
    assert_eq!(relayed, 1);
    assert_eq!(failed, 0);
}

#[test]
fn orchestrate_remote_switches_to_distributed_peer_mesh() {
    let (port, _handle) = spawn_test_fleet_agent("ScoutB", None).expect("spawn fleet agent");
    let source = r#"
robot ScoutA {
  robot ScoutB;
  mission Patrol { navigate; inspect; }
}

robot ScoutB {
  mission Patrol { navigate; inspect; }
}

fleet Recon { ScoutA; ScoutB; }
"#;
    let program = compile(source).expect("compile").program;
    let mut registry = FleetAgentRegistry::default();
    register_fleet_agent(
        &mut registry,
        "ScoutB".into(),
        format!("http://127.0.0.1:{port}"),
        None,
    )
    .expect("register fleet agent");

    std::thread::sleep(Duration::from_millis(50));
    let result = orchestrate_fleets_remote(&program, "peer_fleet.sd", &registry);
    assert!(result.success);
    let fleet = &result.fleets[0];
    assert_eq!(fleet.coordination_mode, "distributed_peer_mesh");
    assert_eq!(fleet.remote_relayed, 1);
    assert_eq!(fleet.remote_failed, 0);
}

#[test]
fn fleet_agent_registry_round_trip() {
    let path = PathBuf::from(".spanda/test-fleet-agents-roundtrip.json");
    let mut registry = FleetAgentRegistry::default();
    register_fleet_agent(
        &mut registry,
        "Rover".into(),
        "http://127.0.0.1:8766".into(),
        Some("secret".into()),
    )
    .expect("register");
    save_fleet_agent_registry(&path, &registry).expect("save");
    let loaded = load_fleet_agent_registry(&path);
    assert_eq!(loaded.agents.len(), 1);
    assert_eq!(loaded.agents[0].robot_name, "Rover");
    let _ = std::fs::remove_file(path);
}

#[test]
fn fleet_entry_for_port_builds_local_url() {
    let entry = fleet_entry_for_port("ScoutA", 9999, Some("tok".into()));
    assert_eq!(entry.url, "http://127.0.0.1:9999");
    assert_eq!(entry.robot_name, "ScoutA");
}
