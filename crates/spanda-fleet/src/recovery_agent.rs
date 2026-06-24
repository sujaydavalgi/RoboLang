//! Assurance-backed recovery execution on fleet agents with deployed programs.

use crate::agent::{apply_recovery_action, FleetAgentState};
use spanda_assurance::{
    classify_failure, extract_recovery_policies, validate_recovery_plan, RecoveryContext,
    RecoveryLevel, RecoveryPlanner, RecoveryReport,
};
use spanda_lexer::tokenize;
use spanda_parser::parse;

fn normalize_action(action: &str) -> String {
    action
        .to_ascii_lowercase()
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect()
}

fn infer_recovery_issue(action: &str) -> String {
    let lower = action.to_ascii_lowercase();
    if lower.contains("gps") {
        return "gps.failed".into();
    }
    if lower.contains("lidar") {
        return "lidar.failed".into();
    }
    if lower.contains("lte") || lower.contains("wifi") || lower.contains("connect") {
        return "connectivity.failed".into();
    }
    if lower.contains("fleet")
        || lower.contains("reassign")
        || lower.contains("promote")
        || lower.contains("redistribute")
    {
        return "fleet.failed".into();
    }
    if lower.contains("mission") {
        return "mission.failed".into();
    }
    "fleet.failed".into()
}

fn recovery_issue_for_action(
    program: &spanda_ast::nodes::Program,
    action: &str,
) -> String {
    let normalized = normalize_action(action);
    for policy in extract_recovery_policies(program) {
        for (condition, actions) in &policy.triggers {
            for branch_action in actions {
                let branch_norm = normalize_action(branch_action);
                if branch_norm == normalized
                    || normalized.contains(&branch_norm)
                    || branch_norm.contains(&normalized)
                {
                    return condition.clone();
                }
            }
        }
    }
    infer_recovery_issue(action)
}

/// Run assurance recovery planning and validation, then apply gated actions on the agent.
pub fn execute_assurance_recovery_on_agent(
    state: &mut FleetAgentState,
    program_source: &str,
    trigger_action: &str,
) -> Result<RecoveryReport, String> {
    let tokens = tokenize(program_source).map_err(|e| e.to_string())?;
    let program = parse(tokens).map_err(|e| e.to_string())?;
    let issue = recovery_issue_for_action(&program, trigger_action);
    let context = RecoveryContext {
        issue: issue.clone(),
        diagnosis: None,
        classification: Some(classify_failure(&issue)),
        level: RecoveryLevel::Level3AutomaticWithValidation,
    };
    let plan = RecoveryPlanner::plan(&program, &context);
    let safe_actions = validate_recovery_plan(&program, &plan);
    let mut executed_any = false;

    for safe in &safe_actions {
        let gates_ok = safe.safety_validation.passed
            && safe.hardware_verification.passed
            && safe.capability_verification.passed
            && safe.readiness_validation.passed;
        if !gates_ok {
            continue;
        }
        if safe.action.requires_approval && !safe.approved {
            continue;
        }
        apply_recovery_action(state, &safe.action.description);
        executed_any = true;
    }

    if !executed_any {
        apply_recovery_action(state, trigger_action);
    }

    let report = spanda_assurance::evaluate_recovery(&program, None);
    state.recovery_validation = Some(if report.passed {
        "PASS".into()
    } else {
        "FAIL".into()
    });
    state.last_recovery_evidence = serde_json::to_value(&report.results).ok();
    Ok(report)
}

/// Handle an inbound fleet recovery peer command on a deployed agent.
pub fn handle_fleet_recovery_command(state: &mut FleetAgentState, action: &str) {
    state.last_recovery_commands.push(action.to_string());
    if let Some(program) = state.program.clone() {
        if execute_assurance_recovery_on_agent(state, &program, action).is_ok() {
            return;
        }
    }
    apply_recovery_action(state, action);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::FleetAgentState;

    const FLEET_PROGRAM: &str = r#"
recovery_policy FleetRecovery {
    on fleet.failed {
        pause mission;
        reassign mission;
    }
}
robot RoverAlpha {
    sensor gps: GPS;
    actuator wheels: DifferentialDrive;
    safety { max_speed = 1.0 m/s; }
    behavior patrol() {}
}
"#;

    #[test]
    fn assurance_recovery_applies_validated_actions() {
        let mut state = FleetAgentState {
            robot_name: "RoverAlpha".into(),
            program: Some(FLEET_PROGRAM.into()),
            ..FleetAgentState::default()
        };
        handle_fleet_recovery_command(&mut state, "pause mission");
        assert!(state.mission_paused);
        assert_eq!(state.recovery_validation.as_deref(), Some("PASS"));
        assert!(state.recovery_actions_applied.iter().any(|a| a.contains("pause")));
    }
}
