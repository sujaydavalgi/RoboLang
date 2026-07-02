//! Twin Cloud SaaS REST handlers — mission twin snapshot registry.

use crate::handlers::{bad_request, json_ok, parse_query};
use crate::program::parse_program_file;
use crate::state::ControlCenterState;
use spanda_deploy_http::HttpResponse;
use spanda_twin_cloud::{
    build_snapshot_from_program, TwinCloudSnapshot, TwinCloudSyncResponse, TWIN_CLOUD_API_VERSION,
};

pub fn route_twin_cloud(
    state: &mut ControlCenterState,
    path: &str,
    method: &str,
    query: &str,
    body: &str,
) -> Option<HttpResponse> {
    if path == "/v1/twins" && method == "GET" {
        return Some(list_twins(state));
    }
    if path == "/v1/twins/sync" && method == "POST" {
        return Some(sync_twin(state, query));
    }
    let rest = path.strip_prefix("/v1/twins/")?;
    if rest.ends_with("/snapshots") && method == "POST" {
        let twin_id = rest.strip_suffix("/snapshots")?;
        return Some(push_snapshot(state, twin_id, body));
    }
    if !rest.contains('/') && method == "GET" {
        return Some(get_twin(state, rest));
    }
    None
}

fn list_twins(state: &ControlCenterState) -> HttpResponse {
    json_ok(
        &state
            .twin_cloud_store
            .list_response(Some(state.tenant_id.as_str())),
    )
}

fn get_twin(state: &ControlCenterState, twin_id: &str) -> HttpResponse {
    let Some(snapshot) = state.twin_cloud_store.get(twin_id) else {
        return twin_not_found(twin_id);
    };
    json_ok(snapshot)
}

fn twin_not_found(twin_id: &str) -> HttpResponse {
    HttpResponse {
        status: 404,
        body: serde_json::json!({
            "ok": false,
            "error": format!("twin '{twin_id}' not found"),
        })
        .to_string(),
    }
}

fn push_snapshot(state: &mut ControlCenterState, twin_id: &str, body: &str) -> HttpResponse {
    let mut snapshot: TwinCloudSnapshot = match serde_json::from_str(body) {
        Ok(value) => value,
        Err(error) => return bad_request(&format!("invalid snapshot JSON: {error}")),
    };
    if snapshot.twin_id.is_empty() {
        snapshot.twin_id = twin_id.to_string();
    } else if snapshot.twin_id != twin_id {
        return bad_request("snapshot twin_id must match path");
    }
    if snapshot.tenant_id.is_empty() {
        snapshot.tenant_id = state.tenant_id.clone();
    }
    let stored = state.twin_cloud_store.upsert(snapshot);
    json_ok(&TwinCloudSyncResponse {
        version: TWIN_CLOUD_API_VERSION.into(),
        twin_id: stored.twin_id.clone(),
        captured_at_ms: stored.captured_at_ms,
        snapshot: stored,
    })
}

fn sync_twin(state: &mut ControlCenterState, query: &str) -> HttpResponse {
    let params = parse_query(query);
    let twin_id = params.get("twin_id").map(String::as_str);
    let (program, _source, label) = match load_program(state) {
        Ok(value) => value,
        Err(message) => return bad_request(&message),
    };
    let snapshot = build_snapshot_from_program(&program, &label, twin_id, state.tenant_id.as_str());
    let stored = state.twin_cloud_store.upsert(snapshot);
    json_ok(&TwinCloudSyncResponse {
        version: TWIN_CLOUD_API_VERSION.into(),
        twin_id: stored.twin_id.clone(),
        captured_at_ms: stored.captured_at_ms,
        snapshot: stored,
    })
}

fn load_program(
    state: &ControlCenterState,
) -> Result<(spanda_ast::nodes::Program, String, String), String> {
    let Some(path) = state.program_path.as_ref() else {
        return Err("no program loaded; use control-center serve --program <file.sd>".into());
    };
    parse_program_file(path).map(|(program, source, label)| (program, source, label))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn patrol_program() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/showcase/mission_twin/patrol.sd")
    }

    #[test]
    fn twin_cloud_sync_stores_snapshot() {
        let program = patrol_program();
        assert!(program.exists());
        let mut state = ControlCenterState::new();
        state.program_path = Some(program);
        let response = sync_twin(&mut state, "");
        assert_eq!(response.status, 200, "{}", response.body);
        let json: serde_json::Value = serde_json::from_str(&response.body).unwrap();
        assert_eq!(json["twin_id"], "patrol");
    }
}
