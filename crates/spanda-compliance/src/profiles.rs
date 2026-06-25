//! Built-in industry compliance profile templates.

use serde::{Deserialize, Serialize};

const ACCREDITATION_TEMPLATE_NOTICE: &str =
    "Template profile only — not legal accreditation or certification.";

/// Template requirements for an industry compliance profile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComplianceProfile {
    pub name: String,
    pub description: String,
    pub requires_kill_switch: bool,
    pub min_readiness_score: u32,
    pub required_capabilities: Vec<String>,
    pub min_health_checks: usize,
    pub requires_assurance_case: bool,
    pub max_speed_mps: Option<f64>,
    pub operation_hours: Option<String>,
    pub requires_secure_comm: bool,
    pub requires_tamper_policy: bool,
    pub warn_only: bool,
    #[serde(skip)]
    pub template_notice: &'static str,
}

/// List built-in compliance profile names.
pub fn list_builtin_profiles() -> Vec<&'static str> {
    vec![
        "industrial",
        "warehouse",
        "medical",
        "agriculture",
        "defense",
        "research",
    ]
}

/// Resolve a built-in compliance profile by name.
pub fn builtin_profile(name: &str) -> Option<ComplianceProfile> {
    match name.trim().to_ascii_lowercase().as_str() {
        "industrial" => Some(industrial_profile()),
        "warehouse" => Some(warehouse_profile()),
        "medical" => Some(medical_profile()),
        "agriculture" => Some(agriculture_profile()),
        "defense" => Some(defense_profile()),
        "research" => Some(research_profile()),
        _ => None,
    }
}

fn industrial_profile() -> ComplianceProfile {
    ComplianceProfile {
        name: "industrial".into(),
        description: "Factory AMRs with fixed safety zones and baseline readiness".into(),
        requires_kill_switch: true,
        min_readiness_score: 75,
        required_capabilities: vec!["obstacle_avoidance".into()],
        min_health_checks: 1,
        requires_assurance_case: false,
        max_speed_mps: Some(1.5),
        operation_hours: None,
        requires_secure_comm: false,
        requires_tamper_policy: false,
        warn_only: false,
        template_notice: ACCREDITATION_TEMPLATE_NOTICE,
    }
}

fn warehouse_profile() -> ComplianceProfile {
    ComplianceProfile {
        name: "warehouse".into(),
        description: "Warehouse AMRs with speed caps and shift-hour discipline".into(),
        requires_kill_switch: true,
        min_readiness_score: 70,
        required_capabilities: vec![
            "gps_navigation".into(),
            "obstacle_avoidance".into(),
        ],
        min_health_checks: 1,
        requires_assurance_case: false,
        max_speed_mps: Some(2.0),
        operation_hours: Some("06:00-22:00".into()),
        requires_secure_comm: false,
        requires_tamper_policy: false,
        warn_only: false,
        template_notice: ACCREDITATION_TEMPLATE_NOTICE,
    }
}

fn medical_profile() -> ComplianceProfile {
    ComplianceProfile {
        name: "medical".into(),
        description: "Medical robotics with stricter health evidence and assurance cases".into(),
        requires_kill_switch: true,
        min_readiness_score: 85,
        required_capabilities: vec![],
        min_health_checks: 2,
        requires_assurance_case: true,
        max_speed_mps: Some(1.0),
        operation_hours: None,
        requires_secure_comm: false,
        requires_tamper_policy: true,
        warn_only: false,
        template_notice: ACCREDITATION_TEMPLATE_NOTICE,
    }
}

fn agriculture_profile() -> ComplianceProfile {
    ComplianceProfile {
        name: "agriculture".into(),
        description: "Outdoor agriculture with GPS reliance and connectivity tolerance".into(),
        requires_kill_switch: true,
        min_readiness_score: 60,
        required_capabilities: vec!["gps_navigation".into()],
        min_health_checks: 1,
        requires_assurance_case: false,
        max_speed_mps: Some(2.5),
        operation_hours: None,
        requires_secure_comm: false,
        requires_tamper_policy: false,
        warn_only: false,
        template_notice: ACCREDITATION_TEMPLATE_NOTICE,
    }
}

fn defense_profile() -> ComplianceProfile {
    ComplianceProfile {
        name: "defense".into(),
        description: "Defense robotics with secure comm and capability minimization".into(),
        requires_kill_switch: true,
        min_readiness_score: 85,
        required_capabilities: vec![],
        min_health_checks: 1,
        requires_assurance_case: true,
        max_speed_mps: Some(1.2),
        operation_hours: None,
        requires_secure_comm: true,
        requires_tamper_policy: true,
        warn_only: false,
        template_notice: ACCREDITATION_TEMPLATE_NOTICE,
    }
}

fn research_profile() -> ComplianceProfile {
    ComplianceProfile {
        name: "research".into(),
        description: "Research deployments with relaxed gates and explicit warnings".into(),
        requires_kill_switch: false,
        min_readiness_score: 50,
        required_capabilities: vec![],
        min_health_checks: 0,
        requires_assurance_case: false,
        max_speed_mps: None,
        operation_hours: None,
        requires_secure_comm: false,
        requires_tamper_policy: false,
        warn_only: true,
        template_notice: ACCREDITATION_TEMPLATE_NOTICE,
    }
}
