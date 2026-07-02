//! CLI platform event runtime that dispatches plugin hooks for canonical events.

use spanda_audit::PlatformEvent;
use spanda_package::manifest::find_project_root;
use spanda_plugin::bridge::hook_for_platform_event;
use spanda_plugin::runtime::PluginManager;
use spanda_runtime::platform_event_runtime::PlatformEventRuntime;
use spanda_telemetry_store::platform_event_bridge::TelemetryStorePlatformEventRuntime;
use std::env;
use std::path::PathBuf;
use std::sync::Arc;

const HOST_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug)]
pub struct CompositePlatformEventRuntime {
    telemetry: TelemetryStorePlatformEventRuntime,
    project_root: PathBuf,
}

impl CompositePlatformEventRuntime {
    pub fn new() -> Self {
        Self {
            telemetry: TelemetryStorePlatformEventRuntime,
            project_root: find_project_root(
                &env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            )
            .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from("."))),
        }
    }
}

impl PlatformEventRuntime for CompositePlatformEventRuntime {
    fn record_platform_event(&self, event: &PlatformEvent) {
        self.telemetry.record_platform_event(event);
        let Some(hook) = hook_for_platform_event(event.event_type.as_str()) else {
            return;
        };
        if let Ok(mut manager) = PluginManager::open(&self.project_root, HOST_VERSION) {
            let payload = serde_json::json!({
                "event_type": event.event_type.as_str(),
                "source": event.source,
                "payload": event.payload,
                "entity_id": event.entity_id,
                "timestamp": event.timestamp,
            });
            let _ = manager.dispatch_hook_to_enabled(hook, payload);
        }
    }
}

pub fn register_platform_event_runtime() {
    spanda_runtime::platform_event_runtime::set_platform_event_runtime(Arc::new(
        CompositePlatformEventRuntime::new(),
    ));
}

pub fn dispatch_report_hook(path: &str) {
    let Some(hook) = spanda_plugin::bridge::hook_for_report_request(path) else {
        return;
    };
    let project_root =
        find_project_root(&env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
            .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    if let Ok(mut manager) = PluginManager::open(&project_root, HOST_VERSION) {
        let _ = manager.dispatch_hook_to_enabled(hook, serde_json::json!({ "path": path }));
    }
}
