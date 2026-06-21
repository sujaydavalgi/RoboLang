//! Swarm coordinator runtime tests.

use spanda_core::{
    check, compile, coordinate_swarms, load_swarm_state, save_swarm_state, SwarmPolicy, SwarmState,
};

#[test]
fn swarm_round_robin_advances_one_member_per_tick() {
    let source = include_str!("../../../examples/robotics/swarm_coordination.sd");
    check(source).expect("swarm example should type-check");
    let program = compile(source).expect("compile").program;
    let mut state = SwarmState::default();
    let first = coordinate_swarms(&program, "swarm_coordination.sd", &mut state);
    assert!(first.success);
    let round_robin = first
        .swarms
        .iter()
        .find(|swarm| swarm.policy == SwarmPolicy::RoundRobin.as_str())
        .expect("round_robin swarm report");
    assert_eq!(round_robin.steps_advanced, 1);
    assert_eq!(round_robin.members.len(), 1);
    assert_eq!(round_robin.round_robin_cursor, 1);
    let first_member = round_robin.members[0].robot_name.clone();

    let second = coordinate_swarms(&program, "swarm_coordination.sd", &mut state);
    let round_robin = second
        .swarms
        .iter()
        .find(|swarm| swarm.policy == SwarmPolicy::RoundRobin.as_str())
        .expect("round_robin swarm report");
    assert_eq!(round_robin.round_robin_cursor, 2);
    assert_ne!(round_robin.members[0].robot_name, first_member);
}

#[test]
fn swarm_broadcast_advances_all_members() {
    let source = include_str!("../../../examples/robotics/swarm_coordination.sd");
    let program = compile(source).expect("compile").program;
    let mut state = SwarmState::default();
    let result = coordinate_swarms(&program, "swarm_coordination.sd", &mut state);
    let broadcast = result
        .swarms
        .iter()
        .find(|swarm| swarm.policy == SwarmPolicy::Broadcast.as_str())
        .expect("broadcast swarm report");
    assert_eq!(broadcast.members.len(), 3);
    assert_eq!(broadcast.steps_advanced, 3);
}

#[test]
fn swarm_state_persists_round_robin_cursor() {
    let path = std::env::temp_dir().join("spanda-swarm-state-test.json");
    let mut state = SwarmState::default();
    state.round_robin_cursor.insert("ReconSwarm".into(), 2);
    save_swarm_state(&path, &state).expect("save swarm state");
    let loaded = load_swarm_state(&path);
    assert_eq!(loaded.round_robin_cursor.get("ReconSwarm"), Some(&2));
    let _ = std::fs::remove_file(path);
}
