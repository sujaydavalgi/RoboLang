//! Integration tests for deploy agent `GET /v1/readiness`.

use spanda_ota::{
    agent_entry_for_port, agent_health, agent_readiness, agent_upload_program, deploy_target_key,
    spawn_test_agent,
};
use std::thread;
use std::time::Duration;

#[test]
fn agent_readiness_after_program_upload() {
    let target = deploy_target_key("RoverProgram", "JetsonOrin");
    let (port, _handle) = spawn_test_agent(&target, None).expect("spawn test agent");
    thread::sleep(Duration::from_millis(50));

    let entry = agent_entry_for_port(&target, port, None);
    assert!(agent_health(&entry).expect("health check"));

    let source = include_str!("../../../examples/showcase/readiness/rover.sd");
    agent_upload_program(&entry, source).expect("upload program");

    let body = agent_readiness(&entry, true, false).expect("readiness");
    assert_eq!(body.get("ok").and_then(|v| v.as_bool()), Some(true));
    assert!(body.get("readiness").is_some());
}
