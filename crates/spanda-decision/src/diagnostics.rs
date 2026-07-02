//! Static diagnostics for distributed decision declarations.

use spanda_ast::nodes::{Program, RobotDecl};
use spanda_capability::VerificationDiagnostic;

/// Collect span-aware diagnostics for decision trees, offline policies, and authority.
pub fn collect_decision_diagnostics(program: &Program) -> Vec<VerificationDiagnostic> {
    let Program::Program {
        decision_trees,
        offline_policies,
        robots,
        ..
    } = program;
    let mut diags = Vec::new();

    if decision_trees.is_empty() && offline_policies.is_empty() {
        let has_authority = robots.iter().any(|r| {
            let RobotDecl::RobotDecl {
                local_decision_authority,
                requires_central_approval,
                ..
            } = r;
            !local_decision_authority.is_empty() || !requires_central_approval.is_empty()
        });
        if has_authority {
            diags.push(VerificationDiagnostic {
                message: "Entity declares decision authority but no decision_tree or offline_policy"
                    .into(),
                line: 1,
                column: 1,
                severity: "info".into(),
                category: "decision:authority".into(),
                suggested_fix: Some(
                    "decision_tree LocalRecovery local {\n    when gps.status == Failed { enter degraded_mode; }\n}"
                        .into(),
                ),
            });
        }
    }

    for tree in decision_trees {
        let spanda_ast::assurance_decl::DecisionTreeDecl::DecisionTreeDecl {
            name,
            branches,
            span,
            ..
        } = tree;
        if branches.is_empty() {
            diags.push(VerificationDiagnostic {
                message: format!("decision_tree '{name}' has no when branches"),
                line: span.start.line,
                column: span.start.column,
                severity: "warning".into(),
                category: "decision:tree".into(),
                suggested_fix: Some(format!(
                    "decision_tree {name} local {{\n    when condition {{ action; }}\n}}"
                )),
            });
        }
    }

    for policy in offline_policies {
        let spanda_ast::assurance_decl::OfflinePolicyDecl::OfflinePolicyDecl {
            name,
            max_duration_minutes,
            allowed_actions,
            forbidden_actions,
            span,
        } = policy;
        if *max_duration_minutes == 0 {
            diags.push(VerificationDiagnostic {
                message: format!("offline_policy '{name}' has zero max_duration"),
                line: span.start.line,
                column: span.start.column,
                severity: "error".into(),
                category: "decision:offline".into(),
                suggested_fix: Some(format!(
                    "offline_policy {name} {{\n    max_duration = 30 min;\n}}"
                )),
            });
        }
        if allowed_actions.is_empty() {
            diags.push(VerificationDiagnostic {
                message: format!("offline_policy '{name}' has no allowed_actions"),
                line: span.start.line,
                column: span.start.column,
                severity: "warning".into(),
                category: "decision:offline".into(),
                suggested_fix: None,
            });
        }
        if forbidden_actions.is_empty() {
            diags.push(VerificationDiagnostic {
                message: format!(
                    "offline_policy '{name}' should forbid high-risk actions while offline"
                ),
                line: span.start.line,
                column: span.start.column,
                severity: "info".into(),
                category: "decision:offline".into(),
                suggested_fix: Some(
                    "forbidden_actions [disable_safety, accept_unknown_device, update_firmware]"
                        .into(),
                ),
            });
        }
    }

    for robot in robots {
        let RobotDecl::RobotDecl {
            name,
            local_decision_authority,
            requires_central_approval,
            span,
            ..
        } = robot;
        if local_decision_authority.is_empty() && requires_central_approval.is_empty() {
            continue;
        }
        for action in requires_central_approval {
            if local_decision_authority.iter().any(|a| a == action) {
                diags.push(VerificationDiagnostic {
                    message: format!(
                        "robot '{name}': '{action}' is both local and requires central approval"
                    ),
                    line: span.start.line,
                    column: span.start.column,
                    severity: "error".into(),
                    category: "decision:authority".into(),
                    suggested_fix: None,
                });
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

    fn parse_sd(source: &str) -> Program {
        parse(tokenize(source).unwrap()).unwrap()
    }

    #[test]
    fn warns_empty_decision_tree() {
        let program = parse_sd(
            r#"
            decision_tree Empty local {}
            "#,
        );
        let diags = collect_decision_diagnostics(&program);
        assert!(diags.iter().any(|d| d.category == "decision:tree"));
    }
}
