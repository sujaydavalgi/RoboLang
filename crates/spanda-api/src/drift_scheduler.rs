//! Scheduled operational drift scans with alert dispatch.
//!
use crate::handlers::{json_ok, now_ms, record_alert, unauthorized};
use crate::state::ControlCenterState;
use serde::{Deserialize, Serialize};
use spanda_config::{
    default_snapshots_dir, detect_operational_drift_full, list_config_snapshots,
    load_config_snapshot, DriftSeverity, OperationalDriftReport,
};
use spanda_deploy_http::HttpResponse;
use spanda_ops::{Alert, AlertSeverity, AlertType};
use spanda_security::{ApiKeyStore, RbacAction, RbacContext};
use std::collections::{BTreeMap, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// One recorded operational drift scan.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DriftScanRecord {
    pub id: String,
    pub scanned_at_ms: f64,
    pub baseline_id: String,
    pub passed: bool,
    pub finding_count: usize,
    pub dimensions_checked: Vec<String>,
    pub by_dimension: BTreeMap<String, u32>,
    pub triggered_alert: bool,
    pub source: String,
}

/// Ring buffer of recent drift scan results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftScanStore {
    pub scans: VecDeque<DriftScanRecord>,
    pub max_entries: usize,
}

impl DriftScanStore {
    pub fn new(max_entries: usize) -> Self {
        Self {
            scans: VecDeque::new(),
            max_entries,
        }
    }

    pub fn push(&mut self, record: DriftScanRecord) {
        if self.scans.len() >= self.max_entries {
            self.scans.pop_front();
        }
        self.scans.push_back(record);
    }

    pub fn list(&self) -> Vec<DriftScanRecord> {
        self.scans.iter().cloned().collect()
    }
}

#[derive(Debug, Deserialize)]
struct DriftScanRequest {
    #[serde(default)]
    baseline_id: Option<String>,
}

fn drift_scans_path(dir: &Path) -> PathBuf {
    dir.join("control-center-drift-scans.json")
}

pub fn hydrate_drift_scans(state: &mut ControlCenterState) {
    let path = drift_scans_path(&crate::persistence::default_state_dir());
    if let Ok(content) = fs::read_to_string(path) {
        if let Ok(store) = serde_json::from_str::<DriftScanStore>(&content) {
            state.drift_scan_store = store;
        }
    }
}

pub fn persist_drift_scans(state: &ControlCenterState) -> Result<(), String> {
    let dir = crate::persistence::default_state_dir();
    fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    fs::write(
        drift_scans_path(&dir),
        serde_json::to_string_pretty(&state.drift_scan_store).map_err(|e| e.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn resolve_baseline_id(explicit: Option<&str>) -> Result<String, String> {
    if let Some(id) = explicit.filter(|value| !value.trim().is_empty()) {
        return Ok(id.to_string());
    }
    if let Ok(id) = std::env::var("SPANDA_DRIFT_SCAN_BASELINE_ID") {
        if !id.trim().is_empty() {
            return Ok(id);
        }
    }
    let snapshots =
        list_config_snapshots(&default_snapshots_dir()).map_err(|error| error.to_string())?;
    snapshots
        .into_iter()
        .max_by(|left, right| {
            left.created_at_ms
                .partial_cmp(&right.created_at_ms)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|meta| meta.id)
        .ok_or_else(|| "no configuration snapshots; save one via POST /v1/config/snapshots".into())
}

pub fn build_operational_drift_report(
    state: &ControlCenterState,
    baseline_id: &str,
) -> Result<OperationalDriftReport, String> {
    let current = state
        .resolved
        .as_ref()
        .ok_or_else(|| "no resolved configuration; use --config".to_string())?;
    let baseline = load_config_snapshot(&default_snapshots_dir(), baseline_id)
        .map(|snapshot| snapshot.resolved)
        .map_err(|error| error.to_string())?;
    let program = state
        .program_path
        .as_ref()
        .and_then(|path| crate::program::parse_program_file(path).ok())
        .map(|(program, _, _)| program);
    let agent_findings = program
        .as_ref()
        .map(|program| {
            crate::drift_collect::collect_agent_drift_findings(
                program,
                current,
                state.program_path.as_deref(),
            )
        })
        .unwrap_or_default();
    Ok(detect_operational_drift_full(
        &baseline,
        current,
        program.as_ref(),
        &agent_findings,
    ))
}

pub fn run_and_record_scan(
    state: &mut ControlCenterState,
    baseline_id: Option<String>,
    source: &str,
    dispatch_alert: bool,
) -> Result<DriftScanRecord, String> {
    let baseline = resolve_baseline_id(baseline_id.as_deref())?;
    let report = build_operational_drift_report(state, &baseline)?;
    let mut triggered_alert = false;
    if dispatch_alert && !report.passed {
        let severity = drift_alert_severity(&report);
        if severity != AlertSeverity::Info {
            let alert = Alert {
                id: format!("drift-{}", now_ms()),
                alert_type: AlertType::ConfigDrift,
                severity,
                message: format!(
                    "operational drift detected (baseline {baseline}): {} findings across {} dimensions",
                    report.findings.len(),
                    report.dimensions_checked.len()
                ),
                source: "drift-scan".into(),
                timestamp_ms: now_ms(),
                delivered_via: vec![],
            };
            record_alert(state, alert);
            triggered_alert = true;
        }
    }
    let record = DriftScanRecord {
        id: format!("drift-scan-{}", now_ms()),
        scanned_at_ms: now_ms(),
        baseline_id: baseline,
        passed: report.passed,
        finding_count: report.findings.len(),
        dimensions_checked: report.dimensions_checked.clone(),
        by_dimension: report.by_dimension.clone(),
        triggered_alert,
        source: source.to_string(),
    };
    state.drift_scan_store.push(record.clone());
    let _ = persist_drift_scans(state);
    let _ = crate::persistence::persist_runtime_state(state);
    Ok(record)
}

fn drift_alert_severity(report: &OperationalDriftReport) -> AlertSeverity {
    let has_high = report
        .findings
        .iter()
        .any(|finding| finding.severity >= DriftSeverity::High);
    if has_high {
        AlertSeverity::Critical
    } else if !report.passed {
        AlertSeverity::Warning
    } else {
        AlertSeverity::Info
    }
}

pub fn drift_scans_list(state: &ControlCenterState) -> HttpResponse {
    json_ok(&serde_json::json!({
        "version": "v1",
        "scans": state.drift_scan_store.list(),
    }))
}

pub fn drift_scan_run(
    state: &mut ControlCenterState,
    body: &str,
    ctx: Option<&RbacContext>,
) -> HttpResponse {
    if !ApiKeyStore::check(ctx, RbacAction::Operate) {
        return unauthorized();
    }
    let request: DriftScanRequest = if body.trim().is_empty() {
        DriftScanRequest { baseline_id: None }
    } else {
        match serde_json::from_str(body) {
            Ok(value) => value,
            Err(error) => {
                return crate::handlers::bad_request(&error.to_string());
            }
        }
    };
    match run_and_record_scan(state, request.baseline_id, "api", true) {
        Ok(scan) => json_ok(&serde_json::json!({
            "version": "v1",
            "ok": true,
            "scan": scan,
        })),
        Err(error) => crate::handlers::bad_request(&error),
    }
}

pub fn spawn_drift_scheduler(state: Arc<Mutex<ControlCenterState>>) {
    let interval_secs = std::env::var("SPANDA_DRIFT_SCAN_INTERVAL_SECS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0);
    if interval_secs == 0 {
        return;
    }
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(interval_secs));
        if let Ok(mut guard) = state.lock() {
            let _ = run_and_record_scan(&mut guard, None, "scheduled", true);
        }
    });
}
