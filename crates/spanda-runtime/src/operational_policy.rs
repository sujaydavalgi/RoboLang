//! Runtime operational policy enforcement for interpreter motion gates.

use chrono::{Local, NaiveTime};
use serde::{Deserialize, Serialize};
use spanda_ast::nodes::Program;
use spanda_ast::policy_decl::{OperationalPolicyDecl, OperationalPolicyRule};

/// Runtime violation surfaced when a policy rule blocks motion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuntimePolicyViolation {
    pub policy: String,
    pub rule: String,
    pub message: String,
}

/// Compiled runtime policy monitor extracted from a named `policy` block.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuntimePolicyMonitor {
    pub policy_name: String,
    pub max_speed_mps: Option<f64>,
    pub operation_hours: Option<String>,
    #[serde(skip)]
    pub clock_override: Option<NaiveTime>,
}

/// Build a runtime policy monitor from a program declaration.
pub fn build_runtime_policy_monitor(
    program: &Program,
    policy_name: &str,
) -> Result<RuntimePolicyMonitor, String> {
    let Program::Program {
        operational_policies,
        ..
    } = program;
    let policy = operational_policies
        .iter()
        .find(|policy| policy_name_matches(policy, policy_name))
        .ok_or_else(|| format!("Policy '{policy_name}' not found in program"))?;
    let mut max_speed_mps = None;
    let mut operation_hours = None;
    for rule in policy_rules(policy) {
        match rule {
            OperationalPolicyRule::MaxSpeed { limit_mps, .. } => {
                max_speed_mps = Some(*limit_mps);
            }
            OperationalPolicyRule::OperationHours { range, .. } => {
                operation_hours = Some(range.clone());
            }
            _ => {}
        }
    }
    Ok(RuntimePolicyMonitor {
        policy_name: policy_name.into(),
        max_speed_mps,
        operation_hours,
        clock_override: None,
    })
}

/// Check whether motion is allowed under the active runtime policy.
pub fn check_runtime_policy_motion(
    monitor: &RuntimePolicyMonitor,
    linear_mps: f64,
) -> Result<(), RuntimePolicyViolation> {
    if let Some(limit) = monitor.max_speed_mps {
        if linear_mps > limit + f64::EPSILON {
            return Err(RuntimePolicyViolation {
                policy: monitor.policy_name.clone(),
                rule: "max_speed".into(),
                message: format!(
                    "requested speed {linear_mps:.2} m/s exceeds policy limit {limit:.2} m/s"
                ),
            });
        }
    }
    if let Some(range) = &monitor.operation_hours {
        if !operation_hours_allow(range, monitor.clock_override)? {
            return Err(RuntimePolicyViolation {
                policy: monitor.policy_name.clone(),
                rule: "operation_hours".into(),
                message: format!("motion outside allowed operation hours `{range}`"),
            });
        }
    }
    Ok(())
}

fn policy_name_matches(policy: &OperationalPolicyDecl, policy_name: &str) -> bool {
    let OperationalPolicyDecl::OperationalPolicyDecl { name, .. } = policy;
    name == policy_name
}

fn policy_rules(policy: &OperationalPolicyDecl) -> &[OperationalPolicyRule] {
    let OperationalPolicyDecl::OperationalPolicyDecl { rules, .. } = policy;
    rules
}

fn operation_hours_allow(
    range: &str,
    clock_override: Option<NaiveTime>,
) -> Result<bool, RuntimePolicyViolation> {
    let Some((start, end)) = parse_operation_hours(range) else {
        return Err(RuntimePolicyViolation {
            policy: "runtime".into(),
            rule: "operation_hours".into(),
            message: format!("invalid operation_hours range `{range}`"),
        });
    };
    let now = clock_override.unwrap_or_else(|| Local::now().time());
    Ok(time_in_range(now, start, end))
}

fn parse_operation_hours(range: &str) -> Option<(NaiveTime, NaiveTime)> {
    let (start_raw, end_raw) = range.split_once('-')?;
    let start = NaiveTime::parse_from_str(start_raw.trim(), "%H:%M").ok()?;
    let end = NaiveTime::parse_from_str(end_raw.trim(), "%H:%M").ok()?;
    Some((start, end))
}

fn time_in_range(now: NaiveTime, start: NaiveTime, end: NaiveTime) -> bool {
    if start <= end {
        now >= start && now <= end
    } else {
        now >= start || now <= end
    }
}
