//! Recovery policy parsing and evaluation from TOML configuration.
//!
use crate::types::{
    EntityRecoveryPolicy, EscalationRule, OrchestratorStrategy, RecoveryEscalationLevel,
};
use spanda_config::entity::{EntityKind, EntityRegistry};
use spanda_config::resolved::ResolvedSystemConfig;
use std::collections::HashMap;

/// Load entity recovery policies from resolved system config.
pub fn load_recovery_policies(
    cfg: &ResolvedSystemConfig,
    registry: &EntityRegistry,
) -> Vec<EntityRecoveryPolicy> {
    // Load entity recovery policies from resolved system config.
    //
    // Parameters:
    // - `cfg` — resolved system configuration
    // - `registry` — entity registry for default policies
    //
    // Returns:
    // Parsed recovery policies per entity.
    //
    // Options:
    // None.
    //
    // Example:
    // let policies = load_recovery_policies(&resolved, &registry);

    let mut policies = Vec::new();
    let mut by_id: HashMap<String, EntityRecoveryPolicy> = HashMap::new();

    if let Some(section) = cfg.recovery_config() {
        if let Some(table) = section.get("policies").and_then(|v| v.as_table()) {
            for (name, value) in table {
                if let Some(policy) = parse_policy_table(name, value) {
                    by_id.insert(policy.entity_id.clone(), policy);
                }
            }
        }
        if let Some(defaults) = section.get("default").and_then(|v| v.as_table()) {
            let default_policy =
                parse_policy_table("default", &toml::Value::Table(defaults.clone()))
                    .unwrap_or_default();
            for entity in registry.list() {
                if !by_id.contains_key(&entity.id) && is_policy_eligible(&entity.entity_type) {
                    let mut p = default_policy.clone();
                    p.entity_id = entity.id.clone();
                    p.entity_kind = Some(entity.entity_type.clone());
                    by_id.insert(entity.id.clone(), p);
                }
            }
        }
    }

    // Ensure every recoverable entity has at least a default policy.
    for entity in registry.list() {
        if is_policy_eligible(&entity.entity_type) && !by_id.contains_key(&entity.id) {
            let mut p = EntityRecoveryPolicy::default();
            p.entity_id = entity.id.clone();
            p.entity_kind = Some(entity.entity_type.clone());
            by_id.insert(entity.id.clone(), p);
        }
    }

    policies.extend(by_id.into_values());
    policies.sort_by(|a, b| a.entity_id.cmp(&b.entity_id));
    policies
}

/// Evaluate whether recovery is allowed under entity policy constraints.
pub fn evaluate_policy(
    policy: &EntityRecoveryPolicy,
    strategy: &OrchestratorStrategy,
    escalation_level: RecoveryEscalationLevel,
) -> (bool, Vec<String>) {
    // Evaluate whether recovery is allowed under entity policy constraints.
    //
    // Parameters:
    // - `policy` — entity recovery policy
    // - `strategy` — proposed strategy
    // - `escalation_level` — proposed escalation level
    //
    // Returns:
    // Tuple of (allowed, explanation messages).
    //
    // Options:
    // None.
    //
    // Example:
    // let (ok, msgs) = evaluate_policy(&policy, &strategy, level);

    let mut messages = Vec::new();
    let strategy_level = strategy.default_escalation_level();

    if escalation_level > policy.max_escalation_level {
        messages.push(format!(
            "Escalation level {:?} exceeds policy maximum {:?}",
            escalation_level, policy.max_escalation_level
        ));
        return (false, messages);
    }

    if strategy_level > policy.max_escalation_level {
        messages.push(format!(
            "Strategy '{}' requires level {:?} which exceeds policy maximum {:?}",
            strategy.label(),
            strategy_level,
            policy.max_escalation_level
        ));
        return (false, messages);
    }

    // Check safety constraints against strategy label.
    for constraint in &policy.safety_constraints {
        if strategy.label().contains(&constraint.to_lowercase()) {
            messages.push(format!(
                "Strategy '{}' blocked by safety constraint '{}'",
                strategy.label(),
                constraint
            ));
            return (false, messages);
        }
    }

    messages.push(format!(
        "Policy allows strategy '{}' at level {:?}",
        strategy.label(),
        escalation_level
    ));
    (true, messages)
}

/// Find policy for a specific entity.
pub fn policy_for_entity<'a>(
    policies: &'a [EntityRecoveryPolicy],
    entity_id: &str,
) -> Option<&'a EntityRecoveryPolicy> {
    policies.iter().find(|p| p.entity_id == entity_id)
}

/// Parse a single policy TOML table.
fn parse_policy_table(name: &str, table: &toml::Value) -> Option<EntityRecoveryPolicy> {
    let t = table.as_table()?;
    let mut policy = EntityRecoveryPolicy::default();
    policy.entity_id = t
        .get("entity_id")
        .and_then(|v| v.as_str())
        .unwrap_or(name)
        .to_string();
    if let Some(kind) = t.get("entity_kind").and_then(|v| v.as_str()) {
        policy.entity_kind = Some(EntityKind::parse(kind));
    }
    if let Some(p) = t.get("priority").and_then(|v| v.as_integer()) {
        policy.priority = p as u32;
    }
    if let Some(t) = t.get("timeout_secs").and_then(|v| v.as_integer()) {
        policy.timeout_secs = t as u64;
    }
    if let Some(r) = t.get("retry_limit").and_then(|v| v.as_integer()) {
        policy.retry_limit = r as u32;
    }
    if let Some(m) = t.get("max_escalation_level").and_then(|v| v.as_integer()) {
        if let Some(level) = RecoveryEscalationLevel::from_u8(m as u8) {
            policy.max_escalation_level = level;
        }
    }
    if let Some(a) = t.get("requires_approval").and_then(|v| v.as_bool()) {
        policy.requires_approval = a;
    }
    policy.dependencies = string_array(t, "dependencies");
    policy.validation_rules = string_array(t, "validation_rules");
    policy.safety_constraints = string_array(t, "safety_constraints");
    policy.trust_requirements = string_array(t, "trust_requirements");
    policy.readiness_requirements = string_array(t, "readiness_requirements");
    policy.escalation_rules = parse_escalation_rules(t);
    Some(policy)
}

fn parse_escalation_rules(table: &toml::map::Map<String, toml::Value>) -> Vec<EscalationRule> {
    let Some(arr) = table.get("escalation_rules").and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    arr.iter()
        .filter_map(|item| {
            let t = item.as_table()?;
            let from = t
                .get("from_level")
                .and_then(|v| v.as_integer())
                .and_then(|n| RecoveryEscalationLevel::from_u8(n as u8))?;
            let to = t
                .get("to_level")
                .and_then(|v| v.as_integer())
                .and_then(|n| RecoveryEscalationLevel::from_u8(n as u8))?;
            let strategy = t
                .get("strategy")
                .and_then(|v| v.as_str())
                .map(parse_strategy)
                .unwrap_or(OrchestratorStrategy::Retry);
            let after_retries = t
                .get("after_retries")
                .and_then(|v| v.as_integer())
                .unwrap_or(1) as u32;
            Some(EscalationRule {
                from_level: from,
                to_level: to,
                after_retries,
                strategy,
            })
        })
        .collect()
}

/// Parse strategy name from TOML or CLI input.
pub fn parse_strategy_for_playbook(s: &str) -> OrchestratorStrategy {
    parse_strategy(s)
}

fn parse_strategy(s: &str) -> OrchestratorStrategy {
    match s.to_ascii_lowercase().as_str() {
        "retry" => OrchestratorStrategy::Retry,
        "restart_component" => OrchestratorStrategy::RestartComponent,
        "restart_service" => OrchestratorStrategy::RestartService,
        "restart_package" => OrchestratorStrategy::RestartPackage,
        "restart_provider" => OrchestratorStrategy::RestartProvider,
        "restart_device" => OrchestratorStrategy::RestartDevice,
        "restart_robot" => OrchestratorStrategy::RestartRobot,
        "restart_fleet" => OrchestratorStrategy::RestartFleet,
        "restart_gateway" => OrchestratorStrategy::RestartGateway,
        "reconnect" => OrchestratorStrategy::Reconnect,
        "reinitialize" => OrchestratorStrategy::Reinitialize,
        "reload_configuration" => OrchestratorStrategy::ReloadConfiguration,
        "rollback" => OrchestratorStrategy::Rollback,
        "switch_provider" => OrchestratorStrategy::SwitchProvider,
        "switch_package" => OrchestratorStrategy::SwitchPackage,
        "switch_hardware" => OrchestratorStrategy::SwitchHardware,
        "switch_sensor" => OrchestratorStrategy::SwitchSensor,
        "switch_network" => OrchestratorStrategy::SwitchNetwork,
        "switch_gateway" => OrchestratorStrategy::SwitchGateway,
        "switch_fleet" => OrchestratorStrategy::SwitchFleet,
        "transfer_mission" => OrchestratorStrategy::TransferMission,
        "delegate_mission" => OrchestratorStrategy::DelegateMission,
        "takeover_mission" => OrchestratorStrategy::TakeoverMission,
        "graceful_degradation" => OrchestratorStrategy::GracefulDegradation,
        "safe_shutdown" => OrchestratorStrategy::SafeShutdown,
        "emergency_shutdown" => OrchestratorStrategy::EmergencyShutdown,
        "human_escalation" => OrchestratorStrategy::HumanEscalation,
        other => OrchestratorStrategy::Custom(other.to_string()),
    }
}

fn string_array(table: &toml::map::Map<String, toml::Value>, key: &str) -> Vec<String> {
    table
        .get(key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

fn is_policy_eligible(kind: &EntityKind) -> bool {
    !matches!(kind, EntityKind::Incident | EntityKind::Hazard)
}
