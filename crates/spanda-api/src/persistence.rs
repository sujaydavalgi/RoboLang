//! Durable persistence for Control Center runtime state (alerts, traces).
//!
use crate::correlation::TraceLog;
use crate::state::ControlCenterState;
use spanda_ops::{Alert, AlertStore, Incident, IncidentStore};
use spanda_twin_cloud::TwinCloudSnapshot;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct PersistedTwins {
    snapshots: Vec<TwinCloudSnapshot>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct PersistedAlerts {
    alerts: Vec<Alert>,
    max_entries: usize,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct PersistedTraces {
    records: Vec<crate::correlation::TraceRecord>,
    max_entries: usize,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct PersistedIncidents {
    incidents: Vec<Incident>,
    max_entries: usize,
}

/// Directory for Control Center HA state files.
pub fn default_state_dir() -> PathBuf {
    std::env::var("SPANDA_CONTROL_CENTER_STATE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(".spanda"))
}

fn alerts_path(dir: &Path) -> PathBuf {
    dir.join("control-center-alerts.json")
}

fn traces_path(dir: &Path) -> PathBuf {
    dir.join("control-center-traces.json")
}

fn incidents_path(dir: &Path) -> PathBuf {
    dir.join("control-center-incidents.json")
}

fn twins_path(dir: &Path) -> PathBuf {
    dir.join("control-center-twins.json")
}

/// Load alerts and traces from disk into runtime state.
pub fn hydrate_runtime_state(state: &mut ControlCenterState) {
    let dir = default_state_dir();
    if let Ok(content) = fs::read_to_string(alerts_path(&dir)) {
        if let Ok(persisted) = serde_json::from_str::<PersistedAlerts>(&content) {
            state.alert_store = AlertStore::from_records(persisted.max_entries, persisted.alerts);
        }
    }
    if let Ok(content) = fs::read_to_string(traces_path(&dir)) {
        if let Ok(persisted) = serde_json::from_str::<PersistedTraces>(&content) {
            state.trace_log = TraceLog::from_records(persisted.max_entries, persisted.records);
        }
    }
    if let Ok(content) = fs::read_to_string(incidents_path(&dir)) {
        if let Ok(persisted) = serde_json::from_str::<PersistedIncidents>(&content) {
            state.incident_store =
                IncidentStore::from_records(persisted.max_entries, persisted.incidents);
        }
    }
    if let Ok(content) = fs::read_to_string(twins_path(&dir)) {
        if let Ok(persisted) = serde_json::from_str::<PersistedTwins>(&content) {
            state.twin_cloud_store = spanda_twin_cloud::TwinCloudStore::from_records(persisted.snapshots);
        }
    }
    crate::drift_scheduler::hydrate_drift_scans(state);
    crate::report_scheduler::hydrate_report_schedules(state);
}

/// Persist alerts and traces to disk.
pub fn persist_runtime_state(state: &ControlCenterState) -> Result<(), String> {
    let dir = default_state_dir();
    fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    let alerts = PersistedAlerts {
        alerts: state.alert_store.list_owned(),
        max_entries: state.alert_store.max_entries,
    };
    let traces = PersistedTraces {
        records: state.trace_log.list_owned(),
        max_entries: state.trace_log.max_entries,
    };
    let incidents = PersistedIncidents {
        incidents: state.incident_store.list_owned(),
        max_entries: state.incident_store.max_entries,
    };
    fs::write(
        alerts_path(&dir),
        serde_json::to_string_pretty(&alerts).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    fs::write(
        traces_path(&dir),
        serde_json::to_string_pretty(&traces).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    fs::write(
        incidents_path(&dir),
        serde_json::to_string_pretty(&incidents).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let twins = PersistedTwins {
        snapshots: state.twin_cloud_store.list_owned(),
    };
    fs::write(
        twins_path(&dir),
        serde_json::to_string_pretty(&twins).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}
