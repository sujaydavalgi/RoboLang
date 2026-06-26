//! Scheduled executive/compliance report delivery.
//!
use crate::e4::reports_export_internal;
use crate::handlers::{json_ok, now_ms, unauthorized};
use crate::state::ControlCenterState;
use serde::{Deserialize, Serialize};
use spanda_deploy_http::HttpResponse;
use spanda_ops::alerting::send_webhook_body;
use spanda_security::{ApiKeyStore, RbacAction, RbacContext};
use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// One scheduled report delivery job.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportSchedule {
    pub id: String,
    pub profile: String,
    pub format: String,
    pub destination_url: String,
    pub interval_hours: u64,
    pub enabled: bool,
    pub created_at_ms: f64,
    pub last_run_ms: Option<f64>,
    pub last_status: Option<String>,
}

/// Ring buffer of report schedules.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReportScheduleStore {
    pub schedules: VecDeque<ReportSchedule>,
    pub max_entries: usize,
}

impl ReportScheduleStore {
    pub fn new(max_entries: usize) -> Self {
        Self {
            schedules: VecDeque::new(),
            max_entries,
        }
    }

    pub fn push(&mut self, schedule: ReportSchedule) {
        if self.schedules.len() >= self.max_entries {
            self.schedules.pop_front();
        }
        self.schedules.push_back(schedule);
    }

    pub fn list(&self) -> Vec<ReportSchedule> {
        self.schedules.iter().cloned().collect()
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut ReportSchedule> {
        self.schedules.iter_mut().find(|schedule| schedule.id == id)
    }
}

#[derive(Debug, Deserialize)]
struct CreateReportScheduleRequest {
    profile: String,
    #[serde(default = "default_format")]
    format: String,
    destination_url: String,
    #[serde(default = "default_interval_hours")]
    interval_hours: u64,
}

fn default_format() -> String {
    "markdown".into()
}

fn default_interval_hours() -> u64 {
    24
}

fn schedules_path(dir: &Path) -> PathBuf {
    dir.join("control-center-report-schedules.json")
}

pub fn hydrate_report_schedules(state: &mut ControlCenterState) {
    let path = schedules_path(&crate::persistence::default_state_dir());
    if let Ok(content) = fs::read_to_string(path) {
        if let Ok(store) = serde_json::from_str::<ReportScheduleStore>(&content) {
            state.report_schedule_store = store;
        }
    }
}

pub fn persist_report_schedules(state: &ControlCenterState) -> Result<(), String> {
    let dir = crate::persistence::default_state_dir();
    fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    fs::write(
        schedules_path(&dir),
        serde_json::to_string_pretty(&state.report_schedule_store).map_err(|e| e.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn report_schedules_list(state: &ControlCenterState) -> HttpResponse {
    json_ok(&serde_json::json!({
        "version": "v1",
        "schedules": state.report_schedule_store.list(),
    }))
}

pub fn report_schedules_create(
    state: &mut ControlCenterState,
    body: &str,
    ctx: Option<&RbacContext>,
) -> HttpResponse {
    if !ApiKeyStore::check(ctx, RbacAction::Deploy) {
        return unauthorized();
    }
    let request: CreateReportScheduleRequest = match serde_json::from_str(body) {
        Ok(value) => value,
        Err(error) => return crate::handlers::bad_request(&error.to_string()),
    };
    if request.destination_url.trim().is_empty() {
        return crate::handlers::bad_request("destination_url required");
    }
    let schedule = ReportSchedule {
        id: format!("report-schedule-{}", now_ms()),
        profile: request.profile,
        format: request.format,
        destination_url: request.destination_url,
        interval_hours: request.interval_hours.max(1),
        enabled: true,
        created_at_ms: now_ms(),
        last_run_ms: None,
        last_status: None,
    };
    state.report_schedule_store.push(schedule.clone());
    let _ = persist_report_schedules(state);
    json_ok(&serde_json::json!({
        "version": "v1",
        "ok": true,
        "schedule": schedule,
    }))
}

fn due_schedules(state: &ControlCenterState, now: f64) -> Vec<ReportSchedule> {
    state
        .report_schedule_store
        .list()
        .into_iter()
        .filter(|schedule| {
            if !schedule.enabled {
                return false;
            }
            let interval_ms = schedule.interval_hours as f64 * 3_600_000.0;
            schedule
                .last_run_ms
                .map(|last| now - last >= interval_ms)
                .unwrap_or(true)
        })
        .collect()
}

pub fn run_due_report_deliveries(state: &mut ControlCenterState) -> usize {
    let now = now_ms();
    let due = due_schedules(state, now);
    let due_count = due.len();
    let mut delivered = 0usize;
    for schedule in due {
        let query = format!("profile={}&format={}", schedule.profile, schedule.format);
        let response = reports_export_internal(state, &query);
        let status = if response.status == 200 {
            if deliver_report(&schedule.destination_url, &response.body).is_ok() {
                delivered += 1;
                Some("delivered".into())
            } else {
                Some("delivery_failed".into())
            }
        } else {
            Some(format!("export_failed:{}", response.status))
        };
        if let Some(stored) = state.report_schedule_store.get_mut(&schedule.id) {
            stored.last_run_ms = Some(now);
            stored.last_status = status;
        }
    }
    if delivered > 0 || due_count > 0 {
        let _ = persist_report_schedules(state);
    }
    delivered
}

fn deliver_report(destination_url: &str, body: &str) -> Result<(), String> {
    let payload = serde_json::json!({
        "type": "spanda.report.delivery",
        "body": body,
    });
    send_webhook_body(destination_url, &payload.to_string())
}

pub fn spawn_report_scheduler(state: Arc<Mutex<ControlCenterState>>) {
    let interval_secs = std::env::var("SPANDA_REPORT_SCHEDULE_INTERVAL_SECS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0);
    if interval_secs == 0 {
        return;
    }
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(interval_secs));
        if let Ok(mut guard) = state.lock() {
            let _ = run_due_report_deliveries(&mut guard);
        }
    });
}
