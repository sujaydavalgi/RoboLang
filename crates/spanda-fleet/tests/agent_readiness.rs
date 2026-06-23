//! Integration tests for fleet agent `GET /v1/readiness`.

use spanda_fleet::{
    agent_readiness, agent_upload_program, fleet_entry_for_port, spawn_test_fleet_agent,
};
use std::thread;
use std::time::Duration;

#[test]
fn fleet_agent_readiness_after_program_upload() {
    let robot = "PickerA";
    let (port, _handle) = spawn_test_fleet_agent(robot, None).expect("spawn fleet agent");
    thread::sleep(Duration::from_millis(50));

    let entry = fleet_entry_for_port(robot, port, None);
    let source = include_str!("../../../examples/showcase/fleet_readiness/warehouse.sd");
    agent_upload_program(&entry, source).expect("upload program");

    let body = agent_readiness(&entry, true, false).expect("readiness");
    assert_eq!(body.get("ok").and_then(|v| v.as_bool()), Some(true));
    assert!(body.get("readiness").is_some());
}
