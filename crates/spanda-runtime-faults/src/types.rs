//! Core runtime fault types and health status values.

pub use spanda_runtime::fault_types::{
    FaultEvidence, FaultScanOptions, FaultScanReport, FaultTimeline, RuntimeFault,
    RuntimeFaultKind, RuntimeHealth, RuntimeHealthStatus, HeartbeatStatus, ProcessHealth,
};

use serde::{Deserialize, Serialize};

/// Crash event record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrashEvent {
    pub process: String,
    pub exit_code: i32,
    pub signal: Option<String>,
    pub panic_message: Option<String>,
    pub timestamp_ms: f64,
    pub abnormal: bool,
}

/// Reboot event record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RebootEvent {
    pub boot_id: String,
    pub previous_boot_id: Option<String>,
    pub uptime_before_ms: f64,
    pub reason: String,
    pub unexpected: bool,
    pub timestamp_ms: f64,
}

/// Memory leak detection event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryLeakEvent {
    pub target: String,
    pub growth_mb: f64,
    pub window_ms: f64,
    pub baseline_mb: f64,
    pub current_mb: f64,
    pub timestamp_ms: f64,
}

/// Out-of-memory event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OomEvent {
    pub target: String,
    pub memory_used_mb: f64,
    pub memory_limit_mb: f64,
    pub timestamp_ms: f64,
}

/// Watchdog timeout event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WatchdogTimeout {
    pub watchdog: String,
    pub target: Option<String>,
    pub timeout_ms: f64,
    pub elapsed_ms: f64,
    pub timestamp_ms: f64,
}

/// Deadlock or starvation event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeadlockEvent {
    pub task: String,
    pub queue: Option<String>,
    pub stalled_ms: f64,
    pub kind: String,
    pub timestamp_ms: f64,
}

/// Restart loop detection record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RestartLoop {
    pub target: String,
    pub restart_count: u32,
    pub window_ms: f64,
    pub timestamps_ms: Vec<f64>,
    pub exceeded: bool,
}

/// Resource pressure snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourcePressure {
    pub resource: String,
    pub value: f64,
    pub threshold: f64,
    pub unit: String,
    pub duration_ms: Option<f64>,
    pub status: RuntimeHealthStatus,
}

/// Runtime reliability evidence for assurance integration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuntimeReliabilityEvidence {
    pub uptime_ms: f64,
    pub crash_free_duration_ms: f64,
    pub reboot_count: u32,
    pub unexpected_reboot_count: u32,
    pub memory_stable: bool,
    pub watchdog_coverage: u32,
    pub restart_policies: u32,
    pub heartbeat_monitors: u32,
}

/// Diagnosis summary for a runtime fault.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FaultDiagnosis {
    pub what: String,
    pub when_ms: f64,
    pub likely_cause: String,
    pub affected: Vec<String>,
    pub recovery_successful: Option<bool>,
}

/// Recovery action recommendation for a fault.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FaultRecoveryAction {
    pub action: String,
    pub target: String,
    pub requires_approval: bool,
    pub safety_validated: bool,
}
