//! RBAC-gated configuration snapshot approval queue.
//!
use crate::config_snapshots::{default_snapshots_dir, load_config_snapshot};
use crate::error::{ConfigError, ConfigResult};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Approval lifecycle for a configuration snapshot publish request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigApprovalStatus {
    Pending,
    Approved,
    Rejected,
}

/// One operator vote on a pending configuration approval request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigApprovalVote {
    pub approver: String,
    pub approved_at_ms: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// One approval request tied to a saved configuration snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigApprovalRequest {
    pub id: String,
    pub snapshot_id: String,
    pub requester: String,
    pub status: ConfigApprovalStatus,
    pub created_at_ms: f64,
    #[serde(default = "default_required_approvals")]
    pub required_approvals: u32,
    #[serde(default)]
    pub approvals: Vec<ConfigApprovalVote>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_at_ms: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolver: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

fn default_required_approvals() -> u32 {
    1
}

/// Resolve how many distinct approvers are required for publish-on-approve.
pub fn approval_policy_required_count(explicit: Option<u32>) -> u32 {
    explicit
        .filter(|count| *count > 0)
        .or_else(|| {
            std::env::var("SPANDA_CONFIG_APPROVALS_REQUIRED")
                .ok()
                .and_then(|value| value.parse::<u32>().ok())
        })
        .filter(|count| *count > 0)
        .unwrap_or(1)
}

/// Whether a request has collected enough distinct approver votes.
pub fn approval_quorum_met(request: &ConfigApprovalRequest) -> bool {
    request.approvals.len() >= request.required_approvals as usize
}

/// Persisted approval queue for Control Center configuration management.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfigApprovalQueue {
    pub requests: Vec<ConfigApprovalRequest>,
}

/// Default path for the approval queue file.
pub fn default_approvals_path() -> PathBuf {
    PathBuf::from(".spanda/config-approvals.json")
}

/// Load the approval queue from disk.
pub fn load_approval_queue(path: &Path) -> ConfigResult<ConfigApprovalQueue> {
    if !path.exists() {
        return Ok(ConfigApprovalQueue::default());
    }
    let text = fs::read_to_string(path).map_err(|source| ConfigError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    serde_json::from_str(&text).map_err(|source| ConfigError::JsonParse {
        path: path.to_path_buf(),
        source,
    })
}

/// Persist the approval queue to disk.
pub fn save_approval_queue(path: &Path, queue: &ConfigApprovalQueue) -> ConfigResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| ConfigError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    let text = serde_json::to_string_pretty(queue).map_err(|source| ConfigError::JsonParse {
        path: path.to_path_buf(),
        source,
    })?;
    fs::write(path, text).map_err(|source| ConfigError::Io {
        path: path.to_path_buf(),
        source,
    })
}

/// Submit a new approval request for an existing snapshot.
pub fn submit_config_approval(
    queue: &mut ConfigApprovalQueue,
    snapshot_id: &str,
    requester: &str,
    note: Option<String>,
    required_approvals: Option<u32>,
) -> ConfigResult<ConfigApprovalRequest> {
    let snapshots_dir = default_snapshots_dir();
    let _snapshot = load_config_snapshot(&snapshots_dir, snapshot_id)?;
    let required = approval_policy_required_count(required_approvals);
    let request = ConfigApprovalRequest {
        id: format!("approval-{}", now_ms().to_string().replace('.', "")),
        snapshot_id: snapshot_id.to_string(),
        requester: requester.to_string(),
        status: ConfigApprovalStatus::Pending,
        created_at_ms: now_ms(),
        required_approvals: required,
        approvals: Vec::new(),
        resolved_at_ms: None,
        resolver: None,
        note,
    };
    queue.requests.push(request.clone());
    Ok(request)
}

/// Record an approver vote; finalizes when quorum is met.
pub fn approve_config_request(
    queue: &mut ConfigApprovalQueue,
    request_id: &str,
    resolver: &str,
    note: Option<String>,
) -> ConfigResult<ConfigApprovalRequest> {
    let request = queue
        .requests
        .iter_mut()
        .find(|entry| entry.id == request_id)
        .ok_or_else(|| ConfigError::Approval {
            detail: format!("approval request '{request_id}' not found"),
        })?;
    if request.status != ConfigApprovalStatus::Pending {
        return Err(ConfigError::Approval {
            detail: format!("approval request '{request_id}' is not pending"),
        });
    }
    if request
        .approvals
        .iter()
        .any(|vote| vote.approver == resolver)
    {
        return Err(ConfigError::Approval {
            detail: format!("approver '{resolver}' already voted on '{request_id}'"),
        });
    }
    if request.required_approvals > 1 && request.requester == resolver {
        return Err(ConfigError::Approval {
            detail: "requester cannot approve their own multi-approver request".into(),
        });
    }
    request.approvals.push(ConfigApprovalVote {
        approver: resolver.to_string(),
        approved_at_ms: now_ms(),
        note,
    });
    if approval_quorum_met(request) {
        request.status = ConfigApprovalStatus::Approved;
        request.resolved_at_ms = Some(now_ms());
        request.resolver = Some(resolver.to_string());
    }
    Ok(request.clone())
}

/// Reject a pending configuration publish request.
pub fn reject_config_request(
    queue: &mut ConfigApprovalQueue,
    request_id: &str,
    resolver: &str,
    note: Option<String>,
) -> ConfigResult<ConfigApprovalRequest> {
    let request = queue
        .requests
        .iter_mut()
        .find(|entry| entry.id == request_id)
        .ok_or_else(|| ConfigError::Approval {
            detail: format!("approval request '{request_id}' not found"),
        })?;
    if request.status != ConfigApprovalStatus::Pending {
        return Err(ConfigError::Approval {
            detail: format!("approval request '{request_id}' is not pending"),
        });
    }
    request.status = ConfigApprovalStatus::Rejected;
    request.resolved_at_ms = Some(now_ms());
    request.resolver = Some(resolver.to_string());
    if note.is_some() {
        request.note = note;
    }
    Ok(request.clone())
}

/// Append-only evidence log path for compliance exports.
pub fn default_evidence_log_path() -> PathBuf {
    PathBuf::from(".spanda/evidence-append.jsonl")
}

/// Append one immutable evidence record as a JSON line.
pub fn append_evidence_record(path: &Path, record: &serde_json::Value) -> ConfigResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| ConfigError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|source| ConfigError::Io {
            path: path.to_path_buf(),
            source,
        })?;
    let line = serde_json::to_string(record).map_err(|source| ConfigError::JsonParse {
        path: path.to_path_buf(),
        source,
    })?;
    writeln!(file, "{line}").map_err(|source| ConfigError::Io {
        path: path.to_path_buf(),
        source,
    })
}

/// Read all evidence records from the append-only log.
pub fn list_evidence_records(path: &Path) -> ConfigResult<Vec<serde_json::Value>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(path).map_err(|source| ConfigError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let mut records = Vec::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        if let Ok(value) = serde_json::from_str(line) {
            records.push(value);
        }
    }
    Ok(records)
}

fn now_ms() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs_f64() * 1000.0)
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approval_resolve_pending_request() {
        let mut queue = ConfigApprovalQueue::default();
        queue.requests.push(ConfigApprovalRequest {
            id: "approval-1".into(),
            snapshot_id: "cfg-1".into(),
            requester: "operator".into(),
            status: ConfigApprovalStatus::Pending,
            created_at_ms: 1.0,
            required_approvals: 1,
            approvals: Vec::new(),
            resolved_at_ms: None,
            resolver: None,
            note: None,
        });
        let approved = approve_config_request(&mut queue, "approval-1", "officer", None).unwrap();
        assert_eq!(approved.status, ConfigApprovalStatus::Approved);
        assert_eq!(approved.approvals.len(), 1);
    }

    #[test]
    fn two_of_two_requires_distinct_approvers() {
        let mut queue = ConfigApprovalQueue::default();
        queue.requests.push(ConfigApprovalRequest {
            id: "approval-2".into(),
            snapshot_id: "cfg-1".into(),
            requester: "operator".into(),
            status: ConfigApprovalStatus::Pending,
            created_at_ms: 1.0,
            required_approvals: 2,
            approvals: Vec::new(),
            resolved_at_ms: None,
            resolver: None,
            note: None,
        });
        let first = approve_config_request(&mut queue, "approval-2", "officer-a", None).unwrap();
        assert_eq!(first.status, ConfigApprovalStatus::Pending);
        assert_eq!(first.approvals.len(), 1);
        let second = approve_config_request(&mut queue, "approval-2", "officer-b", None).unwrap();
        assert_eq!(second.status, ConfigApprovalStatus::Approved);
        assert_eq!(second.approvals.len(), 2);
    }

    #[test]
    fn requester_cannot_self_approve_multi_approver_request() {
        let mut queue = ConfigApprovalQueue::default();
        queue.requests.push(ConfigApprovalRequest {
            id: "approval-3".into(),
            snapshot_id: "cfg-1".into(),
            requester: "operator".into(),
            status: ConfigApprovalStatus::Pending,
            created_at_ms: 1.0,
            required_approvals: 2,
            approvals: Vec::new(),
            resolved_at_ms: None,
            resolver: None,
            note: None,
        });
        let error = approve_config_request(&mut queue, "approval-3", "operator", None)
            .expect_err("self approve");
        assert!(error.to_string().contains("requester cannot approve"));
    }

    #[test]
    fn evidence_append_only_log() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("evidence.jsonl");
        append_evidence_record(&path, &serde_json::json!({"id": "e1"})).unwrap();
        append_evidence_record(&path, &serde_json::json!({"id": "e2"})).unwrap();
        let records = list_evidence_records(&path).unwrap();
        assert_eq!(records.len(), 2);
    }
}
