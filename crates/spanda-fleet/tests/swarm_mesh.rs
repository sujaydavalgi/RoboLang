//! Swarm coordinator mesh relay integration tests.

use spanda_ast::nodes::Program;
use spanda_driver::compile;
use spanda_fleet::{
    coordinate_swarms_mesh, register_fleet_agent, spawn_test_fleet_agent, spawn_test_fleet_mesh,
    FleetAgentRegistry, SwarmCoordinationResult, SwarmState,
};
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

const MESH_READY_TIMEOUT_SECS: u64 = 2;
const MESH_READY_POLL_INTERVAL_MS: u64 = 20;

fn coordinate_mesh_when_ready(
    program: &Program,
    program_path: &str,
    state: &mut SwarmState,
    mesh_url: &str,
) -> SwarmCoordinationResult {
    let deadline = std::time::Instant::now() + Duration::from_secs(MESH_READY_TIMEOUT_SECS);
    loop {
        let result = coordinate_swarms_mesh(program, program_path, state, mesh_url, None);
        if result.success {
            return result;
        }
        if std::time::Instant::now() >= deadline {
            panic!(
                "mesh and/or agent did not become ready within {}s timeout",
                MESH_READY_TIMEOUT_SECS
            );
        }
        thread::sleep(Duration::from_millis(MESH_READY_POLL_INTERVAL_MS));
    }
}

#[test]
#[should_panic(expected = "mesh and/or agent did not become ready within")]
fn coordinate_mesh_when_ready_panics_on_timeout() {
    let (port, _agent) = spawn_test_fleet_agent("ScoutB", None).expect("spawn agent");
    let mut registry = FleetAgentRegistry::default();
    register_fleet_agent(
        &mut registry,
        "ScoutB".into(),
        format!("http://127.0.0.1:{port}"),
        None,
    )
    .expect("register");
    let source = r#"
robot ScoutA {
  mission Patrol { navigate; inspect; }
}
robot ScoutB {
  mission Patrol { navigate; inspect; }
}
fleet Recon { ScoutA; ScoutB; }
swarm ReconLeader {
  fleet Recon;
  policy leader_follow;
}
"#;
    let program = compile(source).expect("compile").program;
    let mut state = SwarmState::default();
    // Reserve an ephemeral port, then release it so mesh HTTP has no server to talk to.
    let ephemeral_listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral listener");
    let sink_port = ephemeral_listener
        .local_addr()
        .expect("ephemeral listener local addr")
        .port();
    drop(ephemeral_listener);
    let unavailable_mesh_url = format!("http://127.0.0.1:{sink_port}/");
    coordinate_mesh_when_ready(&program, "swarm_leader.sd", &mut state, &unavailable_mesh_url);
}

#[test]
fn swarm_leader_follow_relays_via_mesh() {
    let (port, _agent) = spawn_test_fleet_agent("ScoutB", None).expect("spawn agent");
    let mut registry = FleetAgentRegistry::default();
    register_fleet_agent(
        &mut registry,
        "ScoutB".into(),
        format!("http://127.0.0.1:{port}"),
        None,
    )
    .expect("register");
    let (mesh_port, _mesh) = spawn_test_fleet_mesh(&registry).expect("spawn mesh");
    let source = r#"
robot ScoutA {
  mission Patrol { navigate; inspect; }
}
robot ScoutB {
  mission Patrol { navigate; inspect; }
}
fleet Recon { ScoutA; ScoutB; }
swarm ReconLeader {
  fleet Recon;
  policy leader_follow;
}
"#;
    let program = compile(source).expect("compile").program;
    let mut state = SwarmState::default();
    let result = coordinate_mesh_when_ready(
        &program,
        "swarm_leader.sd",
        &mut state,
        &format!("http://127.0.0.1:{mesh_port}"),
    );
    let leader = result
        .swarms
        .iter()
        .find(|swarm| swarm.policy == "leader_follow")
        .expect("leader_follow swarm");
    assert_eq!(leader.remote_relayed, 1);
    assert!(leader.coordination_mode.ends_with("_mesh"));
}

#[test]
fn swarm_round_robin_relays_peer_links_via_mesh() {
    let (port, _agent) = spawn_test_fleet_agent("ScoutB", None).expect("spawn agent");
    let mut registry = FleetAgentRegistry::default();
    register_fleet_agent(
        &mut registry,
        "ScoutB".into(),
        format!("http://127.0.0.1:{port}"),
        None,
    )
    .expect("register");
    let (mesh_port, _mesh) = spawn_test_fleet_mesh(&registry).expect("spawn mesh");
    // ScoutA declares ScoutB as a peer robot for round-robin handoffs (valid Spanda syntax).
    let source = r#"
robot ScoutA {
  robot ScoutB;
  mission Patrol { navigate; inspect; }
}
robot ScoutB {
  mission Patrol { navigate; inspect; }
}
robot ScoutC {
  mission Patrol { navigate; inspect; }
}
fleet Recon { ScoutA; ScoutB; ScoutC; }
swarm ReconSwarm {
  fleet Recon;
  policy round_robin;
}
"#;
    let program = compile(source).expect("compile").program;
    let mut state = SwarmState::default();
    let result = coordinate_mesh_when_ready(
        &program,
        "swarm_round.sd",
        &mut state,
        &format!("http://127.0.0.1:{mesh_port}"),
    );
    let round_robin = result
        .swarms
        .iter()
        .find(|swarm| swarm.policy == "round_robin")
        .expect("round_robin swarm");
    assert_eq!(round_robin.remote_relayed, 1);
    assert!(round_robin.coordination_mode.ends_with("_mesh"));
}

#[test]
fn swarm_leader_follow_relays_to_multiple_followers_via_mesh() {
    let (port_b, _agent_b) = spawn_test_fleet_agent("ScoutB", None).expect("spawn ScoutB");
    let (port_c, _agent_c) = spawn_test_fleet_agent("ScoutC", None).expect("spawn ScoutC");
    let mut registry = FleetAgentRegistry::default();
    register_fleet_agent(
        &mut registry,
        "ScoutB".into(),
        format!("http://127.0.0.1:{port_b}"),
        None,
    )
    .expect("register ScoutB");
    register_fleet_agent(
        &mut registry,
        "ScoutC".into(),
        format!("http://127.0.0.1:{port_c}"),
        None,
    )
    .expect("register ScoutC");
    let (mesh_port, _mesh) = spawn_test_fleet_mesh(&registry).expect("spawn mesh");
    let source = r#"
robot ScoutA {
  mission Patrol { navigate; inspect; }
}
robot ScoutB {
  mission Patrol { navigate; inspect; }
}
robot ScoutC {
  mission Patrol { navigate; inspect; }
}
fleet Recon { ScoutA; ScoutB; ScoutC; }
swarm ReconLeader {
  fleet Recon;
  policy leader_follow;
}
"#;
    let program = compile(source).expect("compile").program;
    let mut state = SwarmState::default();
    let result = coordinate_mesh_when_ready(
        &program,
        "swarm_leader.sd",
        &mut state,
        &format!("http://127.0.0.1:{mesh_port}"),
    );
    let leader = result
        .swarms
        .iter()
        .find(|swarm| swarm.policy == "leader_follow")
        .expect("leader_follow swarm");
    assert_eq!(leader.remote_relayed, 2);
    assert_eq!(leader.remote_failed, 0);
    assert!(leader.coordination_mode.ends_with("_mesh"));
}
