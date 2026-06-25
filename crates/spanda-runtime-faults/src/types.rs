//! Core runtime fault types and health status values.

use serde::{Deserialize, Serialize};

/// Overall runtime health status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum RuntimeHealthStatus {
    Healthy,
    Warning,
    Degraded,
    Critical,
    Crashed,
    Rebooted,
    Unknown,
}

/// Heartbeat monitoring status for a runtime target.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeartbeatStatus {
    pub target: String,
    pub last_seen_ms: f64,
    pub interval_ms: f64,
    pub timeout_ms: f64,
    pub missed_count: u32,
    pub status: RuntimeHealthStatus,
}

/// Process-level health snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessHealth {
    pub name: String,
    pub pid: Option<u32>,
    pub status: RuntimeHealthStatus,
    pub exit_code: Option<i32>,
    pub restart_count: u32,
    pub uptime_ms: f64,
}

/// Aggregated runtime health for a program.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuntimeHealth {
    pub overall: RuntimeHealthStatus,
    pub heartbeats: Vec<HeartbeatStatus>,
    pub processes: Vec<ProcessHealth>,
    pub active_faults: Vec<RuntimeFault>,
    pub uptime_ms: f64,
    pub boot_id: Option<String>,
}

/// Kind of runtime fault detected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeFaultKind {
    MemoryLeak,
    ProcessCrash,
    RuntimePanic,
    UnexpectedReboot,
    OsReboot,
    WatchdogTimeout,
    OutOfMemory,
    Deadlock,
    TaskStarvation,
    ProviderCrash,
    PackageCrash,
    SensorDriverCrash,
    ActuatorDriverCrash,
    NetworkStackCrash,
    RestartLoop,
    AbnormalShutdown,
    CpuOverload,
    MemoryPressure,
    DiskPressure,
    TelemetryDrop,
    HeartbeatLoss,
}

/// A detected runtime fault with evidence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuntimeFault {
    pub kind: RuntimeFaultKind,
    pub target: String,
    pub status: RuntimeHealthStatus,
    pub message: String,
    pub evidence: FaultEvidence,
    pub detected_at_ms: f64,
}

/// Supporting evidence for a fault diagnosis.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct FaultEvidence {
    pub metric: Option<String>,
    pub value: Option<String>,
    pub threshold: Option<String>,
    pub boot_id: Option<String>,
    pub exit_code: Option<i32>,
    pub stack_trace: Option<String>,
    pub related_events: Vec<String>,
}

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

/// Chronological fault timeline entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FaultTimeline {
    pub timestamp_ms: f64,
    pub event: String,
    pub fault_kind: Option<RuntimeFaultKind>,
    pub target: String,
    pub status: RuntimeHealthStatus,
    pub detail: Option<String>,
}

/// Full fault scan report for a program.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FaultScanReport {
    pub source: String,
    pub health: RuntimeHealth,
    pub faults: Vec<RuntimeFault>,
    pub timeline: Vec<FaultTimeline>,
    pub heartbeats_configured: u32,
    pub memory_watches_configured: u32,
    pub resource_watches_configured: u32,
    pub restart_policies_configured: u32,
    pub passed: bool,
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

impl Default for RuntimeHealthStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

impl RuntimeHealthStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Healthy => "Healthy",
            Self::Warning => "Warning",
            Self::Degraded => "Degraded",
            Self::Critical => "Critical",
            Self::Crashed => "Crashed",
            Self::Rebooted => "Rebooted",
            Self::Unknown => "Unknown",
        }
    }

    pub fn severity_rank(&self) -> u8 {
        match self {
            Self::Healthy => 0,
            Self::Warning => 1,
            Self::Degraded => 2,
            Self::Rebooted => 3,
            Self::Critical => 4,
            Self::Crashed => 5,
            Self::Unknown => 6,
        }
    }
}

impl RuntimeFaultKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MemoryLeak => "memory_leak",
            Self::ProcessCrash => "process_crash",
            Self::RuntimePanic => "runtime_panic",
            Self::UnexpectedReboot => "unexpected_reboot",
            Self::OsReboot => "os_reboot",
            Self::WatchdogTimeout => "watchdog_timeout",
            Self::OutOfMemory => "out_of_memory",
            Self::Deadlock => "deadlock",
            Self::TaskStarvation => "task_starvation",
            Self::ProviderCrash => "provider_crash",
            Self::PackageCrash => "package_crash",
            Self::SensorDriverCrash => "sensor_driver_crash",
            Self::ActuatorDriverCrash => "actuator_driver_crash",
            Self::NetworkStackCrash => "network_stack_crash",
            Self::RestartLoop => "restart_loop",
            Self::AbnormalShutdown => "abnormal_shutdown",
            Self::CpuOverload => "cpu_overload",
            Self::MemoryPressure => "memory_pressure",
            Self::DiskPressure => "disk_pressure",
            Self::TelemetryDrop => "telemetry_drop",
            Self::HeartbeatLoss => "heartbeat_loss",
        }
    }
}
