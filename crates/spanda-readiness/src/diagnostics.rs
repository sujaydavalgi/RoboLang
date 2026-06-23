//! Readiness diagnostics for CLI, LSP, and CI integration.

use crate::engine::evaluate_readiness_with_runtime;
use crate::runtime::build_runtime_context;
use crate::types::{ReadinessOptions, ReadinessSeverity};
use spanda_ast::nodes::Program;
use spanda_capability::VerificationDiagnostic;

/// Collect span-aware readiness diagnostics for IDE and `spanda check --readiness-json`.
pub fn collect_readiness_diagnostics(
    program: &Program,
    options: &ReadinessOptions,
) -> Vec<VerificationDiagnostic> {
    let runtime = if options.include_runtime {
        Some(build_runtime_context(program, options.inject_health_faults))
    } else {
        None
    };
    let report = evaluate_readiness_with_runtime(program, options, runtime.as_ref());
    report
        .issues
        .iter()
        .map(|issue| VerificationDiagnostic {
            message: issue.message.clone(),
            line: 1,
            column: 1,
            severity: severity_label(issue.severity).into(),
            category: format!("readiness:{}", issue.factor.to_ascii_lowercase()),
            suggested_fix: issue.suggested_action.clone(),
        })
        .collect()
}

fn severity_label(severity: ReadinessSeverity) -> &'static str {
    match severity {
        ReadinessSeverity::Critical => "error",
        ReadinessSeverity::High => "error",
        ReadinessSeverity::Medium => "warning",
        ReadinessSeverity::Low => "info",
        ReadinessSeverity::Info => "info",
    }
}
