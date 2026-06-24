//! Span-aware continuity policy diagnostics for IDE and `spanda check --readiness-json`.

use spanda_ast::assurance_decl::ContinuityPolicyDecl;
use spanda_ast::nodes::{Program, RobotDecl, TopicDecl};
use spanda_ast::robotics_decl::FleetDecl;
use spanda_capability::VerificationDiagnostic;

fn normalize_action(action: &str) -> String {
    // Description:
    //     Normalize action text for continuity policy comparisons.
    //
    // Inputs:
    //     action: &str
    //         Raw continuity policy action text.
    //
    // Outputs:
    //     result: String
    //         Lowercased action with whitespace removed.
    //
    // Example:
    //     let key = normalize_action("hot takeover");

    action
        .to_ascii_lowercase()
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect()
}

fn continuity_action_is_high_risk(action: &str) -> bool {
    // Description:
    //     Decide whether a continuity action needs operator approval.
    //
    // Inputs:
    //     action: &str
    //         Continuity policy action text.
    //
    // Outputs:
    //     result: bool
    //         True when the action implies hot, cold, or human takeover.
    //
    // Example:
    //     let risky = continuity_action_is_high_risk("hot takeover");

    let lower = normalize_action(action);
    lower.contains("hottakeover")
        || lower.contains("coldtakeover")
        || lower.contains("humantakeover")
        || lower.contains("operatortakeover")
}

fn robot_has_approval_topic(robot: &RobotDecl) -> bool {
    // Description:
    //     Check whether a robot exposes an Approval topic.
    //
    // Inputs:
    //     robot: &RobotDecl
    //         Robot declaration from the program AST.
    //
    // Outputs:
    //     result: bool
    //         True when an Approval topic is declared.
    //
    // Example:
    //     let approved = robot_has_approval_topic(robot);

    let RobotDecl::RobotDecl { topics, .. } = robot;
    topics.iter().any(|topic| {
        let TopicDecl::TopicDecl { message_type, .. } = topic;
        message_type == "Approval"
    })
}

fn program_has_approval_path(program: &Program) -> bool {
    // Description:
    //     Check whether any robot exposes an operator approval topic.
    //
    // Inputs:
    //     program: &Program
    //         Parsed Spanda program.
    //
    // Outputs:
    //     result: bool
    //         True when an Approval topic exists on any robot.
    //
    // Example:
    //     let approved = program_has_approval_path(program);

    let Program::Program { robots, .. } = program;
    robots.iter().any(robot_has_approval_topic)
}

fn recovery_has_handoff_action(program: &Program) -> bool {
    // Description:
    //     Detect recovery policies that reassign or promote fleet work.
    //
    // Inputs:
    //     program: &Program
    //         Parsed Spanda program.
    //
    // Outputs:
    //     result: bool
    //         True when a recovery branch performs mission handoff.
    //
    // Example:
    //     let handoff = recovery_has_handoff_action(program);

    let Program::Program {
        recovery_policies, ..
    } = program;
    recovery_policies.iter().any(|policy| {
        let spanda_ast::assurance_decl::RecoveryPolicyDecl::RecoveryPolicyDecl {
            branches, ..
        } = policy;
        branches.iter().any(|branch| {
            branch.actions.iter().any(|action| {
                let lower = normalize_action(action);
                lower.contains("reassign")
                    || lower.contains("promote")
                    || lower.contains("replace")
                    || lower.contains("redistribute")
            })
        })
    })
}

fn fleet_member_count(program: &Program) -> usize {
    // Description:
    //     Count fleet members declared across all fleets.
    //
    // Inputs:
    //     program: &Program
    //         Parsed Spanda program.
    //
    // Outputs:
    //     result: usize
    //         Total number of fleet member references.
    //
    // Example:
    //     let members = fleet_member_count(program);

    let Program::Program { fleets, .. } = program;
    fleets
        .iter()
        .map(|fleet| {
            let FleetDecl::FleetDecl { members, .. } = fleet;
            members.len()
        })
        .sum()
}

/// Collect continuity-policy diagnostics for static analysis and IDE hints.
pub fn collect_continuity_diagnostics(program: &Program) -> Vec<VerificationDiagnostic> {
    // Description:
    //     Collect continuity diagnostics for check JSON and IDE hints.
    //
    // Inputs:
    //     program: &Program
    //         Parsed Spanda program.
    //
    // Outputs:
    //     result: Vec<VerificationDiagnostic>
    //         Span-aware continuity policy diagnostics.
    //
    // Example:
    //     let diags = collect_continuity_diagnostics(program);

    let Program::Program {
        continuity_policies,
        recovery_policies,
        fleets,
        ..
    } = program;

    let mut diags = Vec::new();
    let approval_path = program_has_approval_path(program);
    let has_continuity = !continuity_policies.is_empty();
    let multi_member_fleet = fleet_member_count(program) >= 2;

    if multi_member_fleet && !has_continuity {
        let line = fleets
            .first()
            .map(|fleet| {
                let FleetDecl::FleetDecl { span, .. } = fleet;
                span.start.line
            })
            .unwrap_or(1);
        let column = fleets
            .first()
            .map(|fleet| {
                let FleetDecl::FleetDecl { span, .. } = fleet;
                span.start.column
            })
            .unwrap_or(1);
        diags.push(VerificationDiagnostic {
            message: "Fleet declared without continuity_policy for takeover and succession"
                .into(),
            line,
            column,
            severity: "warning".into(),
            category: "continuity:policy".into(),
            suggested_fix: Some(
                "Add continuity_policy with on robot.failed { resume from checkpoint; reassign mission; }"
                    .into(),
            ),
        });
    }

    if recovery_has_handoff_action(program) && multi_member_fleet && !has_continuity {
        let line = recovery_policies
            .first()
            .map(|policy| {
                let spanda_ast::assurance_decl::RecoveryPolicyDecl::RecoveryPolicyDecl {
                    span, ..
                } = policy;
                span.start.line
            })
            .unwrap_or(1);
        let column = recovery_policies
            .first()
            .map(|policy| {
                let spanda_ast::assurance_decl::RecoveryPolicyDecl::RecoveryPolicyDecl {
                    span, ..
                } = policy;
                span.start.column
            })
            .unwrap_or(1);
        diags.push(VerificationDiagnostic {
            message: "Recovery reassigns mission but no continuity_policy defines takeover mode"
                .into(),
            line,
            column,
            severity: "info".into(),
            category: "continuity:handoff".into(),
            suggested_fix: Some(
                "Pair recovery reassign with continuity_policy for checkpoint resume and successor selection"
                    .into(),
            ),
        });
    }

    for policy in continuity_policies {
        let ContinuityPolicyDecl::ContinuityPolicyDecl {
            name,
            branches,
            span,
        } = policy;
        if branches.is_empty() {
            diags.push(VerificationDiagnostic {
                message: format!("continuity_policy '{name}' has no on branches"),
                line: span.start.line,
                column: span.start.column,
                severity: "warning".into(),
                category: "continuity:policy".into(),
                suggested_fix: Some("Add on robot.failed { resume from checkpoint; } branches".into()),
            });
            continue;
        }
        for branch in branches {
            let trigger_lower = branch.condition.to_ascii_lowercase();
            if (trigger_lower.contains("fleet") || trigger_lower.contains("swarm"))
                && fleets.is_empty()
            {
                diags.push(VerificationDiagnostic {
                    message: format!(
                        "continuity_policy '{name}' references fleet failures but no fleet is declared"
                    ),
                    line: branch.span.start.line,
                    column: branch.span.start.column,
                    severity: "error".into(),
                    category: "continuity:fleet".into(),
                    suggested_fix: Some("Declare fleet <Name> { members; } or adjust trigger".into()),
                });
            }
            for action in &branch.actions {
                if continuity_action_is_high_risk(action) && !approval_path {
                    diags.push(VerificationDiagnostic {
                        message: format!(
                            "High-risk continuity action '{action}' should have an Approval topic or operator path"
                        ),
                        line: branch.span.start.line,
                        column: branch.span.start.column,
                        severity: "warning".into(),
                        category: "continuity:approval".into(),
                        suggested_fix: Some(
                            "Add topic approval: Approval subscribe on \"/ops/approval\"; or mission requires approval Operator"
                                .into(),
                        ),
                    });
                }
            }
        }
    }

    diags
}

#[cfg(test)]
mod tests {
    use super::*;
    use spanda_lexer::tokenize;
    use spanda_parser::parse;

    #[test]
    fn warns_when_fleet_lacks_continuity_policy() {
        let program = parse(tokenize(
            r#"
fleet Patrol { RoverA; RoverB; }
robot RoverA { sensor gps: GPS; actuator w: DifferentialDrive; safety { max_speed = 1 m/s; } behavior b() {} }
robot RoverB { sensor gps: GPS; actuator w: DifferentialDrive; safety { max_speed = 1 m/s; } behavior b() {} }
"#,
        ).unwrap()).unwrap();
        let diags = collect_continuity_diagnostics(&program);
        assert!(diags.iter().any(|d| d.category == "continuity:policy"));
    }

    #[test]
    fn warns_when_hot_takeover_lacks_approval_topic() {
        let program = parse(tokenize(
            r#"
continuity_policy Risky {
    on robot.failed { hot takeover; }
}
robot R { sensor gps: GPS; actuator w: DifferentialDrive; safety { max_speed = 1 m/s; } behavior b() {} }
"#,
        ).unwrap()).unwrap();
        let diags = collect_continuity_diagnostics(&program);
        assert!(diags.iter().any(|d| d.category == "continuity:approval"));
    }
}
