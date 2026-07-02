//! Decision authority extraction and validation.

use crate::types::{DecisionAuthority, DecisionBoundary, DecisionLayer, DecisionPolicy};
use spanda_ast::nodes::{Program, RobotDecl};

/// Extract decision authorities from robot declarations in a program.
pub fn extract_decision_authorities(program: &Program) -> Vec<DecisionAuthority> {
    // Description:
    //     Build decision authority records from robot local_decision_authority blocks.
    //
    // Parameters:
    // - `program` — parsed Spanda program
    //
    // Returns:
    // Vector of per-entity decision authorities.
    //
    // Options:
    // None.
    //
    // Example:
    // let authorities = extract_decision_authorities(&program);

    let Program::Program { robots, .. } = program;
    let mut out = Vec::new();
    for robot in robots {
        let RobotDecl::RobotDecl {
            name,
            local_decision_authority,
            requires_central_approval,
            ..
        } = robot;
        if local_decision_authority.is_empty() && requires_central_approval.is_empty() {
            continue;
        }
        out.push(DecisionAuthority {
            entity_id: name.clone(),
            local_actions: local_decision_authority.clone(),
            requires_central_approval: requires_central_approval.clone(),
            layer: DecisionLayer::LocalEntity,
        });
    }
    out
}

/// Check whether an entity may decide an action locally.
pub fn entity_may_decide_locally(authority: &DecisionAuthority, action: &str) -> bool {
    // Description:
    //     Return true when the action is in local authority and not central-only.
    //
    // Parameters:
    // - `authority` — entity decision authority
    // - `action` — action identifier
    //
    // Returns:
    // True when local decision is permitted.
    //
    // Options:
    // None.
    //
    // Example:
    // let ok = entity_may_decide_locally(&auth, "emergency_stop");

    if authority
        .requires_central_approval
        .iter()
        .any(|a| a == action)
    {
        return false;
    }
    authority.local_actions.iter().any(|a| a == action)
}

/// Build default safety boundaries that no local layer may bypass.
pub fn default_safety_boundaries() -> Vec<DecisionBoundary> {
    vec![
        DecisionBoundary {
            action: "disable_kill_switch".into(),
            max_layer: DecisionLayer::ControlCenter,
            requires_approval: true,
            reason: "Kill switch disable requires central approval".into(),
        },
        DecisionBoundary {
            action: "override_safety_policy".into(),
            max_layer: DecisionLayer::ControlCenter,
            requires_approval: true,
            reason: "Safety policy override requires governance".into(),
        },
        DecisionBoundary {
            action: "accept_unknown_device".into(),
            max_layer: DecisionLayer::ControlCenter,
            requires_approval: true,
            reason: "Unknown devices require central trust validation".into(),
        },
        DecisionBoundary {
            action: "update_firmware".into(),
            max_layer: DecisionLayer::ControlCenter,
            requires_approval: true,
            reason: "Firmware updates require central approval".into(),
        },
    ]
}

/// Validate an action against policy boundaries.
pub fn validate_against_policy(
    policy: &DecisionPolicy,
    action: &str,
    layer: DecisionLayer,
) -> Result<(), String> {
    // Description:
    //     Ensure action is allowed at the given layer under policy rules.
    //
    // Parameters:
    // - `policy` — decision policy
    // - `action` — proposed action
    // - `layer` — decision layer attempting the action
    //
    // Returns:
    // Ok when permitted, Err with reason when blocked.
    //
    // Options:
    // None.
    //
    // Example:
    // validate_against_policy(&policy, "pause_mission", DecisionLayer::LocalEntity)?;

    if policy.forbidden_actions.iter().any(|a| a == action) {
        return Err(format!(
            "action '{action}' is forbidden by policy '{}'",
            policy.name
        ));
    }
    if layer as u8 > policy.layer as u8 {
        return Err(format!(
            "action '{action}' requires layer <= {:?}, got {:?}",
            policy.layer, layer
        ));
    }
    if !policy.allowed_actions.is_empty() && !policy.allowed_actions.iter().any(|a| a == action) {
        return Err(format!(
            "action '{action}' not in allowed list for policy '{}'",
            policy.name
        ));
    }
    Ok(())
}
