//! REST handlers for installed Control Center and CLI plugins.

use crate::handlers::{json_ok, bad_request};
use crate::state::ControlCenterState;
use spanda_deploy_http::HttpResponse;
use spanda_plugin::runtime::PluginManager;

const HOST_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn list_control_center_plugins(state: &ControlCenterState) -> HttpResponse {
    let Some(project_root) = state.project_root() else {
        return json_ok(&serde_json::json!({ "plugins": [] }));
    };
    let Ok(manager) = PluginManager::open(&project_root, HOST_VERSION) else {
        return bad_request("plugin store unavailable");
    };
    match manager.list_control_center_plugins() {
        Ok(plugins) => json_ok(&serde_json::json!({ "plugins": plugins })),
        Err(err) => bad_request(&err.to_string()),
    }
}

pub fn list_all_plugins(state: &ControlCenterState) -> HttpResponse {
    let Some(project_root) = state.project_root() else {
        return json_ok(&serde_json::json!({ "plugins": [] }));
    };
    let Ok(manager) = PluginManager::open(&project_root, HOST_VERSION) else {
        return bad_request("plugin store unavailable");
    };
    let plugins: Vec<_> = manager
        .store()
        .list()
        .into_iter()
        .map(|p| {
            serde_json::json!({
                "name": p.name,
                "version": p.version,
                "state": format!("{:?}", p.state).to_lowercase(),
                "plugin_type": p.plugin_type,
                "trust_tier": p.trust_tier,
            })
        })
        .collect();
    json_ok(&serde_json::json!({ "plugins": plugins }))
}
