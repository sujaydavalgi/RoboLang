//! JSON-RPC gateway dispatches SDK program methods.
use spanda_api::e3::rpc_gateway;
use spanda_api::state::ControlCenterState;

#[test]
fn rpc_gateway_lists_entities() {
    let mut state = ControlCenterState::new();
    let body = r#"{"method":"spanda.v1.ControlCenter/ListEntities","params":{}}"#;
    let resp = rpc_gateway(&mut state, body);
    assert_eq!(resp.status, 200);
    assert!(resp.body.contains("entities"));
}

#[test]
fn rpc_gateway_entity_graph() {
    let mut state = ControlCenterState::new();
    let body = r#"{"method":"spanda.v1.ControlCenter/GetEntityGraph","params":{}}"#;
    let resp = rpc_gateway(&mut state, body);
    assert_eq!(resp.status, 200);
    assert!(resp.body.contains("graph"));
}

#[test]
fn rpc_gateway_unknown_method_is_400() {
    let mut state = ControlCenterState::new();
    let resp = rpc_gateway(&mut state, r#"{"method":"spanda.v1.ControlCenter/Nope"}"#);
    assert_eq!(resp.status, 400);
}
