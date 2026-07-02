//! Recovery Orchestrator types — escalation levels, strategies, policies, and reports.
//!
use serde::{Deserialize, Serialize};
use spanda_config::entity::{EntityKind, EntityRecord};
use spanda_runtime::recovery_types::{
    FailureClassification, RecoveryLevel, RecoveryPlan, RecoveryReport, RecoveryStatus,
    RecoveryStrategy,
};

/// Standardized recovery escalation levels (0–8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryEscalationLevel {
    /// Level 0 — retry transient failures.
    Level0Retry = 0,
    /// Level 1 — restart software component or service.
    Level1RestartComponent = 1,
    /// Level 2 — restart package or provider.
    Level2RestartPackage = 2,
    /// Level 3 — recover hardware or device.
    Level3RecoverDevice = 3,
    /// Level 4 — recover robot or vehicle.
    Level4RecoverRobot = 4,
    /// Level 5 — mission reassignment (delegate / takeover).
    Level5MissionReassign = 5,
    /// Level 6 — fleet or swarm redistribution.
    Level6FleetRedistribute = 6,
    /// Level 7 — human intervention required.
    Level7HumanIntervention = 7,
    /// Level 8 — emergency shutdown.
    Level8EmergencyShutdown = 8,
}

impl RecoveryEscalationLevel {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Level0Retry => "retry",
            Self::Level1RestartComponent => "restart_component",
            Self::Level2RestartPackage => "restart_package",
            Self::Level3RecoverDevice => "recover_device",
            Self::Level4RecoverRobot => "recover_robot",
            Self::Level5MissionReassign => "mission_reassign",
            Self::Level6FleetRedistribute => "fleet_redistribute",
            Self::Level7HumanIntervention => "human_intervention",
            Self::Level8EmergencyShutdown => "emergency_shutdown",
        }
    }

    pub fn from_u8(n: u8) -> Option<Self> {
        match n {
            0 => Some(Self::Level0Retry),
            1 => Some(Self::Level1RestartComponent),
            2 => Some(Self::Level2RestartPackage),
            3 => Some(Self::Level3RecoverDevice),
            4 => Some(Self::Level4RecoverRobot),
            5 => Some(Self::Level5MissionReassign),
            6 => Some(Self::Level6FleetRedistribute),
            7 => Some(Self::Level7HumanIntervention),
            8 => Some(Self::Level8EmergencyShutdown),
            _ => None,
        }
    }
}

/// Extensible orchestrator recovery strategy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrchestratorStrategy {
    Retry,
    RestartComponent,
    RestartService,
    RestartPackage,
    RestartProvider,
    RestartDevice,
    RestartRobot,
    RestartFleet,
    RestartGateway,
    Reconnect,
    Reinitialize,
    ReloadConfiguration,
    Rollback,
    SwitchProvider,
    SwitchPackage,
    SwitchHardware,
    SwitchSensor,
    SwitchNetwork,
    SwitchGateway,
    SwitchFleet,
    TransferMission,
    DelegateMission,
    TakeoverMission,
    GracefulDegradation,
    SafeShutdown,
    EmergencyShutdown,
    HumanEscalation,
    Custom(String),
}

impl OrchestratorStrategy {
    /// Map orchestrator strategy to legacy [`RecoveryStrategy`] when possible.
    pub fn to_legacy(&self) -> RecoveryStrategy {
        match self {
            Self::Retry => RecoveryStrategy::Custom("retry".into()),
            Self::Reconnect | Self::RestartGateway | Self::SwitchNetwork | Self::SwitchGateway => {
                RecoveryStrategy::ConnectivityRestart
            }
            Self::RestartProvider | Self::SwitchProvider => RecoveryStrategy::ProviderReconnect,
            Self::RestartPackage | Self::SwitchPackage | Self::ReloadConfiguration => {
                RecoveryStrategy::PackageReload
            }
            Self::RestartDevice | Self::Reinitialize | Self::SwitchSensor => {
                RecoveryStrategy::DeviceReinitialize
            }
            Self::SwitchHardware => RecoveryStrategy::RedundantHardwareSwitch,
            Self::GracefulDegradation => RecoveryStrategy::DegradedMode,
            Self::TransferMission | Self::DelegateMission | Self::TakeoverMission => {
                RecoveryStrategy::MissionPause
            }
            Self::RestartFleet | Self::SwitchFleet => RecoveryStrategy::FleetReassign,
            Self::HumanEscalation => RecoveryStrategy::OperatorAlert,
            Self::SafeShutdown | Self::EmergencyShutdown => RecoveryStrategy::DegradedMode,
            Self::RestartComponent | Self::RestartService | Self::RestartRobot => {
                RecoveryStrategy::Custom(self.label().to_string())
            }
            Self::Rollback => RecoveryStrategy::Custom("rollback".into()),
            Self::Custom(s) => RecoveryStrategy::Custom(s.clone()),
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Retry => "retry",
            Self::RestartComponent => "restart_component",
            Self::RestartService => "restart_service",
            Self::RestartPackage => "restart_package",
            Self::RestartProvider => "restart_provider",
            Self::RestartDevice => "restart_device",
            Self::RestartRobot => "restart_robot",
            Self::RestartFleet => "restart_fleet",
            Self::RestartGateway => "restart_gateway",
            Self::Reconnect => "reconnect",
            Self::Reinitialize => "reinitialize",
            Self::ReloadConfiguration => "reload_configuration",
            Self::Rollback => "rollback",
            Self::SwitchProvider => "switch_provider",
            Self::SwitchPackage => "switch_package",
            Self::SwitchHardware => "switch_hardware",
            Self::SwitchSensor => "switch_sensor",
            Self::SwitchNetwork => "switch_network",
            Self::SwitchGateway => "switch_gateway",
            Self::SwitchFleet => "switch_fleet",
            Self::TransferMission => "transfer_mission",
            Self::DelegateMission => "delegate_mission",
            Self::TakeoverMission => "takeover_mission",
            Self::GracefulDegradation => "graceful_degradation",
            Self::SafeShutdown => "safe_shutdown",
            Self::EmergencyShutdown => "emergency_shutdown",
            Self::HumanEscalation => "human_escalation",
            Self::Custom(s) => s.as_str(),
        }
    }

    pub fn default_escalation_level(&self) -> RecoveryEscalationLevel {
        match self {
            Self::Retry => RecoveryEscalationLevel::Level0Retry,
            Self::RestartComponent | Self::RestartService => {
                RecoveryEscalationLevel::Level1RestartComponent
            }
            Self::RestartPackage
            | Self::RestartProvider
            | Self::ReloadConfiguration
            | Self::SwitchProvider
            | Self::SwitchPackage
            | Self::Rollback => RecoveryEscalationLevel::Level2RestartPackage,
            Self::RestartDevice
            | Self::Reinitialize
            | Self::SwitchHardware
            | Self::SwitchSensor
            | Self::Reconnect
            | Self::RestartGateway
            | Self::SwitchNetwork
            | Self::SwitchGateway => RecoveryEscalationLevel::Level3RecoverDevice,
            Self::RestartRobot => RecoveryEscalationLevel::Level4RecoverRobot,
            Self::TransferMission | Self::DelegateMission | Self::TakeoverMission => {
                RecoveryEscalationLevel::Level5MissionReassign
            }
            Self::RestartFleet | Self::SwitchFleet => {
                RecoveryEscalationLevel::Level6FleetRedistribute
            }
            Self::HumanEscalation => RecoveryEscalationLevel::Level7HumanIntervention,
            Self::SafeShutdown | Self::EmergencyShutdown => {
                RecoveryEscalationLevel::Level8EmergencyShutdown
            }
            Self::GracefulDegradation => RecoveryEscalationLevel::Level1RestartComponent,
            Self::Custom(_) => RecoveryEscalationLevel::Level1RestartComponent,
        }
    }
}

/// Per-entity recovery policy from TOML or program declarations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityRecoveryPolicy {
    pub entity_id: String,
    pub entity_kind: Option<EntityKind>,
    pub priority: u32,
    pub timeout_secs: u64,
    pub retry_limit: u32,
    pub max_escalation_level: RecoveryEscalationLevel,
    pub escalation_rules: Vec<EscalationRule>,
    pub dependencies: Vec<String>,
    pub validation_rules: Vec<String>,
    pub requires_approval: bool,
    pub safety_constraints: Vec<String>,
    pub trust_requirements: Vec<String>,
    pub readiness_requirements: Vec<String>,
}

impl Default for EntityRecoveryPolicy {
    fn default() -> Self {
        Self {
            entity_id: String::new(),
            entity_kind: None,
            priority: 50,
            timeout_secs: 300,
            retry_limit: 3,
            max_escalation_level: RecoveryEscalationLevel::Level4RecoverRobot,
            escalation_rules: Vec::new(),
            dependencies: Vec::new(),
            validation_rules: vec!["health".into(), "readiness".into(), "trust".into()],
            requires_approval: false,
            safety_constraints: Vec::new(),
            trust_requirements: Vec::new(),
            readiness_requirements: Vec::new(),
        }
    }
}

/// Escalation rule — when retries exhausted, escalate to next level.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EscalationRule {
    pub from_level: RecoveryEscalationLevel,
    pub to_level: RecoveryEscalationLevel,
    pub after_retries: u32,
    pub strategy: OrchestratorStrategy,
}

/// Recovery playbook — versioned, reusable multi-step recovery workflow.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecoveryPlaybook {
    pub name: String,
    pub version: String,
    pub description: String,
    pub trigger: String,
    pub steps: Vec<PlaybookStep>,
    pub entity_kinds: Vec<EntityKind>,
}

/// Single step in a recovery playbook.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlaybookStep {
    pub order: u32,
    pub description: String,
    pub strategy: OrchestratorStrategy,
    pub escalation_level: RecoveryEscalationLevel,
    pub timeout_secs: u64,
    pub requires_validation: bool,
}

/// Recovery decision from the decision engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecoveryDecision {
    pub can_recover: bool,
    pub should_recover: bool,
    pub is_safe: bool,
    pub is_authorized: bool,
    pub automatic: bool,
    pub recommended_strategy: OrchestratorStrategy,
    pub recommended_level: RecoveryEscalationLevel,
    pub lowest_risk_strategy: OrchestratorStrategy,
    pub mission_disruption_score: u32,
    pub estimated_downtime_secs: u64,
    pub backup_entity_id: Option<String>,
    pub explanations: Vec<String>,
}

/// Orchestrated recovery plan for a specific entity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrchestratedRecoveryPlan {
    pub plan_id: String,
    pub entity_id: String,
    pub entity_kind: EntityKind,
    pub failure: String,
    pub classification: FailureClassification,
    pub diagnosis: String,
    pub strategies: Vec<OrchestratorStrategy>,
    pub escalation_level: RecoveryEscalationLevel,
    pub decision: RecoveryDecision,
    pub legacy_plan: Option<RecoveryPlan>,
    pub playbook: Option<String>,
    pub upstream_impact: Vec<String>,
    pub downstream_impact: Vec<String>,
    pub estimated_duration_secs: u64,
    pub risk: String,
}

/// Simulation mode for recovery operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoverySimulationMode {
    Plan,
    Simulate,
    DryRun,
    Validate,
}

/// Input for orchestrator recovery operations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecoveryOrchestratorRequest {
    pub entity_id: Option<String>,
    pub failure: Option<String>,
    pub mode: RecoverySimulationMode,
    pub playbook: Option<String>,
    pub max_escalation_level: Option<RecoveryEscalationLevel>,
    pub force_execute: bool,
}

impl Default for RecoveryOrchestratorRequest {
    fn default() -> Self {
        Self {
            entity_id: None,
            failure: None,
            mode: RecoverySimulationMode::Plan,
            playbook: None,
            max_escalation_level: None,
            force_execute: false,
        }
    }
}

/// Immutable recovery evidence record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrchestratorRecoveryEvidence {
    pub evidence_id: String,
    pub root_cause: String,
    pub strategy: OrchestratorStrategy,
    pub timeline: Vec<RecoveryTimelineEvent>,
    pub entities_involved: Vec<String>,
    pub safety_validation: String,
    pub readiness_result: String,
    pub trust_result: String,
    pub operator_actions: Vec<String>,
    pub automatic_decisions: Vec<String>,
    pub mission_impact: String,
    pub duration_secs: u64,
    pub status: RecoveryStatus,
    pub timestamp: String,
}

/// Event in a recovery timeline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecoveryTimelineEvent {
    pub phase: String,
    pub description: String,
    pub timestamp: String,
    pub duration_ms: u64,
}

/// Recovery metrics aggregated across history.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct RecoveryMetrics {
    pub total_recoveries: u64,
    pub successful_recoveries: u64,
    pub failed_recoveries: u64,
    pub success_rate: f64,
    pub average_duration_secs: f64,
    pub most_effective_strategies: Vec<StrategyEffectiveness>,
    pub repeated_failures: Vec<RepeatedFailure>,
    pub recovery_confidence: f64,
}

/// Strategy effectiveness from historical data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrategyEffectiveness {
    pub strategy: OrchestratorStrategy,
    pub success_rate: f64,
    pub average_duration_secs: f64,
    pub usage_count: u64,
}

/// Repeated failure pattern.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepeatedFailure {
    pub failure_pattern: String,
    pub entity_id: String,
    pub occurrence_count: u64,
    pub last_seen: String,
}

/// Predictive recovery indicator from telemetry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PredictiveIndicator {
    pub indicator: String,
    pub entity_id: String,
    pub severity: String,
    pub confidence: f64,
    pub recommended_action: OrchestratorStrategy,
    pub preventative: bool,
}

/// Full orchestrator report wrapping legacy assurance output.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrchestratorRecoveryReport {
    pub plans: Vec<OrchestratedRecoveryPlan>,
    pub evidence: Vec<OrchestratorRecoveryEvidence>,
    pub metrics: RecoveryMetrics,
    pub predictive_indicators: Vec<PredictiveIndicator>,
    pub legacy_report: Option<RecoveryReport>,
    pub simulation_mode: RecoverySimulationMode,
    pub passed: bool,
}

/// Entity target for universal recovery APIs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecoveryEntityTarget {
    pub id: String,
    pub kind: EntityKind,
    pub display_name: String,
}

impl From<&EntityRecord> for RecoveryEntityTarget {
    fn from(entity: &EntityRecord) -> Self {
        Self {
            id: entity.id.clone(),
            kind: entity.entity_type.clone(),
            display_name: entity
                .display_name
                .clone()
                .or_else(|| entity.name.clone())
                .unwrap_or_else(|| entity.id.clone()),
        }
    }
}

/// Plugin-contributed recovery extension.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginRecoveryExtension {
    pub plugin_id: String,
    pub extension_kind: String,
    pub name: String,
    pub description: String,
}

/// Validation result for orchestrated recovery.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrchestratorValidationResult {
    pub passed: bool,
    pub health_verification: bool,
    pub capability_verification: bool,
    pub hardware_verification: bool,
    pub readiness_verification: bool,
    pub trust_verification: bool,
    pub security_verification: bool,
    pub mission_validation: bool,
    pub messages: Vec<String>,
}

/// Context passed through orchestrator operations.
#[derive(Debug, Clone)]
pub struct OrchestratorContext {
    pub autonomy_level: RecoveryLevel,
    pub dry_run: bool,
    pub skip_execution: bool,
}

impl Default for OrchestratorContext {
    fn default() -> Self {
        Self {
            autonomy_level: RecoveryLevel::Level3AutomaticWithValidation,
            dry_run: false,
            skip_execution: false,
        }
    }
}
