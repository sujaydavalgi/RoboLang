//! API tests for entity mutation write path.
//!
use spanda_api::handlers::handle_request;
use spanda_api::state::ControlCenterState;
use spanda_config::ConfigResolver;
use spanda_deploy_http::HttpRequest;
use spanda_security::{ApiKeyRecord, Role};
use std::path::PathBuf;

fn warehouse_state() -> ControlCenterState {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("spanda-config/tests/fixtures/warehouse");
    let resolved = ConfigResolver::new()
        .resolve_from_dir(&root)
        .expect("resolve warehouse");
    let mut state = ControlCenterState::new().with_config_path(root.join("spanda.toml"));
    state.api_keys.keys.push(ApiKeyRecord {
        key_id: "entity-mutation-test".into(),
        token: "entity-mutation-test-key".into(),
        role: Role::Administrator,
        label: Some("entity mutation test".into()),
        tenant_id: state.tenant_id.clone(),
    });
    state.resolved = Some(resolved);
    state
}

fn provision_auth() -> Option<String> {
    Some("entity-mutation-test-key".into())
}

#[test]
fn entity_register_and_tag_mutations() {
    let mut state = warehouse_state();
    let (register, _) = handle_request(
        &mut state,
        &HttpRequest {
            method: "POST".into(),
            path: "/v1/entities/register".into(),
            body: r#"{
                "id": "staging-bay",
                "entity_type": "calibration_station",
                "display_name": "Staging Bay",
                "parent_id": "warehouse-a",
                "capabilities": ["calibrate"],
                "tags": ["mutation"]
            }"#
            .into(),
            authorization: provision_auth(),
        },
        "",
    );
    assert_eq!(register.status, 200, "body={}", register.body);
    assert!(register.body.contains("staging-bay"));
    let (tag, _) = handle_request(
        &mut state,
        &HttpRequest {
            method: "POST".into(),
            path: "/v1/entities/staging-bay/tags".into(),
            body: r#"{"add":["verified"]}"#.into(),
            authorization: provision_auth(),
        },
        "",
    );
    assert_eq!(tag.status, 200, "body={}", tag.body);
    assert!(tag.body.contains("verified"));
}

#[test]
fn entity_relate_links_existing_entities() {
    let mut state = warehouse_state();
    let (response, _) = handle_request(
        &mut state,
        &HttpRequest {
            method: "POST".into(),
            path: "/v1/entities/relationships".into(),
            body: r#"{
                "from_id": "rover-001",
                "to_id": "gps-001",
                "kind": "depends_on",
                "label": "runtime_link"
            }"#
            .into(),
            authorization: provision_auth(),
        },
        "",
    );
    assert_eq!(response.status, 200, "body={}", response.body);
    assert!(response.body.contains("depends_on"));
}
