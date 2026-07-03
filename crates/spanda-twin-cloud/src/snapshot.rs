//! Twin Cloud snapshot envelope wrapping a digital mission twin report.

use serde::{Deserialize, Serialize};
use spanda_ast::nodes::Program;
use spanda_readiness::{evaluate_mission_twin, MissionTwinReport};

/// Twin Cloud REST API version label.
pub const TWIN_CLOUD_API_VERSION: &str = "v1";

/// Mission twin snapshot stored in Twin Cloud SaaS.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TwinCloudSnapshot {
    pub version: String,
    pub twin_id: String,
    pub tenant_id: String,
    pub program: String,
    pub captured_at_ms: u64,
    pub mission_twin: MissionTwinReport,
}

/// Summary row returned by twin list endpoints.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TwinCloudSummary {
    pub twin_id: String,
    pub tenant_id: String,
    pub program: String,
    pub captured_at_ms: u64,
    pub mission_ready: bool,
    pub readiness_score: u32,
    #[serde(default)]
    pub history_count: usize,
}

/// History response envelope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TwinCloudHistoryResponse {
    pub version: String,
    pub twin_id: String,
    pub snapshots: Vec<TwinCloudSnapshot>,
}

/// List twins response envelope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TwinCloudListResponse {
    pub version: String,
    pub twins: Vec<TwinCloudSummary>,
}

/// Sync/push acknowledgement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TwinCloudSyncResponse {
    pub version: String,
    pub twin_id: String,
    pub captured_at_ms: u64,
    pub snapshot: TwinCloudSnapshot,
}

/// Derive a stable twin id from a program label (basename without extension).
pub fn default_twin_id(source_label: &str) -> String {
    let stem = std::path::Path::new(source_label)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or(source_label);
    stem.replace('.', "-").to_lowercase()
}

/// Evaluate mission twin and wrap as a cloud snapshot.
pub fn build_snapshot_from_program(
    program: &Program,
    source_label: &str,
    twin_id: Option<&str>,
    tenant_id: &str,
) -> TwinCloudSnapshot {
    let report = evaluate_mission_twin(program, source_label);
    let twin_id = twin_id
        .map(str::to_string)
        .unwrap_or_else(|| default_twin_id(source_label));
    TwinCloudSnapshot {
        version: TWIN_CLOUD_API_VERSION.into(),
        twin_id,
        tenant_id: tenant_id.into(),
        program: source_label.into(),
        captured_at_ms: unix_ms(),
        mission_twin: report,
    }
}

impl TwinCloudSnapshot {
    /// Convert snapshot metadata to a list summary row.
    pub fn summary(&self) -> TwinCloudSummary {
        TwinCloudSummary {
            twin_id: self.twin_id.clone(),
            tenant_id: self.tenant_id.clone(),
            program: self.program.clone(),
            captured_at_ms: self.captured_at_ms,
            mission_ready: self.mission_twin.risks.mission_ready,
            readiness_score: self.mission_twin.risks.readiness_score,
            history_count: 0,
        }
    }
}

fn unix_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}
