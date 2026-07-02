//! Format orchestrator reports for CLI and API output.
//!
use crate::graph::RecoveryGraph;
use crate::types::{
    OrchestratedRecoveryPlan, OrchestratorRecoveryEvidence, OrchestratorRecoveryReport,
    RecoveryDecision, RecoveryMetrics, RecoveryPlaybook,
};
use spanda_readiness::ReportFormat;

/// Format orchestrator recovery report.
pub fn format_orchestrator_report(
    report: &OrchestratorRecoveryReport,
    format: ReportFormat,
) -> String {
    match format {
        ReportFormat::Json => serde_json::to_string_pretty(report).unwrap_or_default(),
        ReportFormat::Markdown => format_markdown(report),
        ReportFormat::Html => format_html(report),
        ReportFormat::Text => format_text(report),
    }
}

/// Format recovery decision explanations.
pub fn format_decision(decision: &RecoveryDecision, format: ReportFormat) -> String {
    match format {
        ReportFormat::Json => serde_json::to_string_pretty(decision).unwrap_or_default(),
        _ => {
            let mut out = String::from("Recovery Decision\n=================\n\n");
            out.push_str(&format!("Can recover: {}\n", decision.can_recover));
            out.push_str(&format!("Should recover: {}\n", decision.should_recover));
            out.push_str(&format!("Is safe: {}\n", decision.is_safe));
            out.push_str(&format!("Is authorized: {}\n", decision.is_authorized));
            out.push_str(&format!("Automatic: {}\n", decision.automatic));
            out.push_str(&format!(
                "Recommended: {} (level {:?})\n",
                decision.recommended_strategy.label(),
                decision.recommended_level
            ));
            out.push_str(&format!(
                "Lowest risk: {}\n",
                decision.lowest_risk_strategy.label()
            ));
            out.push_str(&format!(
                "Mission disruption: {}\n",
                decision.mission_disruption_score
            ));
            out.push_str(&format!(
                "Estimated downtime: {}s\n",
                decision.estimated_downtime_secs
            ));
            if let Some(ref backup) = decision.backup_entity_id {
                out.push_str(&format!("Backup entity: {backup}\n"));
            }
            out.push_str("\nExplanations:\n");
            for exp in &decision.explanations {
                out.push_str(&format!("  - {exp}\n"));
            }
            out
        }
    }
}

/// Format recovery metrics.
pub fn format_metrics(metrics: &RecoveryMetrics, format: ReportFormat) -> String {
    match format {
        ReportFormat::Json => serde_json::to_string_pretty(metrics).unwrap_or_default(),
        _ => {
            let mut out = String::from("Recovery Metrics\n================\n\n");
            out.push_str(&format!("Total recoveries: {}\n", metrics.total_recoveries));
            out.push_str(&format!("Successful: {}\n", metrics.successful_recoveries));
            out.push_str(&format!("Failed: {}\n", metrics.failed_recoveries));
            out.push_str(&format!(
                "Success rate: {:.1}%\n",
                metrics.success_rate * 100.0
            ));
            out.push_str(&format!(
                "Average duration: {:.1}s\n",
                metrics.average_duration_secs
            ));
            out.push_str(&format!(
                "Recovery confidence: {:.1}%\n",
                metrics.recovery_confidence * 100.0
            ));
            if !metrics.most_effective_strategies.is_empty() {
                out.push_str("\nMost effective strategies:\n");
                for s in &metrics.most_effective_strategies {
                    out.push_str(&format!(
                        "  {} — {:.0}% success ({} uses)\n",
                        s.strategy.label(),
                        s.success_rate * 100.0,
                        s.usage_count
                    ));
                }
            }
            out
        }
    }
}

/// Format playbook list.
pub fn format_playbooks(playbooks: &[RecoveryPlaybook], format: ReportFormat) -> String {
    match format {
        ReportFormat::Json => serde_json::to_string_pretty(playbooks).unwrap_or_default(),
        _ => {
            let mut out = String::from("Recovery Playbooks\n==================\n\n");
            for pb in playbooks {
                out.push_str(&format!("{} (v{})\n", pb.name, pb.version));
                out.push_str(&format!("  Trigger: {}\n", pb.trigger));
                out.push_str(&format!("  {}\n", pb.description));
                out.push_str(&format!("  Steps: {}\n", pb.steps.len()));
                for step in &pb.steps {
                    out.push_str(&format!(
                        "    {}. {} [{}]\n",
                        step.order,
                        step.description,
                        step.strategy.label()
                    ));
                }
                out.push('\n');
            }
            out
        }
    }
}

/// Format recovery history.
pub fn format_history(evidence: &[OrchestratorRecoveryEvidence], format: ReportFormat) -> String {
    match format {
        ReportFormat::Json => serde_json::to_string_pretty(evidence).unwrap_or_default(),
        _ => {
            let mut out = String::from("Recovery History\n================\n\n");
            for ev in evidence {
                out.push_str(&format!(
                    "[{}] {} — {} ({:?})\n",
                    ev.timestamp,
                    ev.evidence_id,
                    ev.strategy.label(),
                    ev.status
                ));
                out.push_str(&format!("  Root cause: {}\n", ev.root_cause));
                out.push_str(&format!("  Duration: {}s\n", ev.duration_secs));
                out.push_str(&format!("  Mission impact: {}\n", ev.mission_impact));
            }
            if evidence.is_empty() {
                out.push_str("No recovery history recorded.\n");
            }
            out
        }
    }
}

fn format_text(report: &OrchestratorRecoveryReport) -> String {
    let mut out = String::from("Recovery Orchestrator Report\n");
    out.push_str("============================\n\n");
    out.push_str(&format!("Mode: {:?}\n", report.simulation_mode));
    out.push_str(&format!("Passed: {}\n\n", report.passed));
    out.push_str(&format!("Plans ({}):\n", report.plans.len()));
    for plan in &report.plans {
        out.push_str(&format_plan_text(plan));
    }
    if !report.predictive_indicators.is_empty() {
        out.push_str(&format!(
            "\nPredictive indicators ({}):\n",
            report.predictive_indicators.len()
        ));
        for ind in &report.predictive_indicators {
            out.push_str(&format!(
                "  {} on {} — {} (confidence {:.0}%)\n",
                ind.indicator,
                ind.entity_id,
                ind.severity,
                ind.confidence * 100.0
            ));
        }
    }
    out.push_str(&format_metrics(&report.metrics, ReportFormat::Text));
    out
}

fn format_plan_text(plan: &OrchestratedRecoveryPlan) -> String {
    let mut out = format!("\n  Plan: {}\n", plan.plan_id);
    out.push_str(&format!(
        "  Entity: {} ({:?})\n",
        plan.entity_id, plan.entity_kind
    ));
    out.push_str(&format!("  Failure: {}\n", plan.failure));
    out.push_str(&format!("  Risk: {}\n", plan.risk));
    if let Some(ref pb) = plan.playbook {
        out.push_str(&format!("  Playbook: {pb}\n"));
    }
    out.push_str("  Strategies:\n");
    for s in &plan.strategies {
        out.push_str(&format!("    - {}\n", s.label()));
    }
    out
}

fn format_markdown(report: &OrchestratorRecoveryReport) -> String {
    let mut out = String::from("# Recovery Orchestrator Report\n\n");
    out.push_str(&format!("**Mode:** `{:?}`  \n", report.simulation_mode));
    out.push_str(&format!("**Passed:** {}  \n\n", report.passed));
    out.push_str("## Plans\n\n");
    for plan in &report.plans {
        out.push_str(&format!("### {}\n", plan.plan_id));
        out.push_str(&format!("- Entity: `{}`\n", plan.entity_id));
        out.push_str(&format!("- Failure: {}\n", plan.failure));
        out.push_str(&format!("- Risk: {}\n", plan.risk));
    }
    out
}

fn format_html(report: &OrchestratorRecoveryReport) -> String {
    format!(
        "<html><body><h1>Recovery Orchestrator</h1><p>Mode: {:?}</p><p>Passed: {}</p><p>Plans: {}</p></body></html>",
        report.simulation_mode,
        report.passed,
        report.plans.len()
    )
}

/// Format recovery graph for CLI.
pub fn format_graph(graph: &RecoveryGraph, format: ReportFormat) -> String {
    match format {
        ReportFormat::Json => serde_json::to_string_pretty(graph).unwrap_or_default(),
        _ => crate::graph::format_recovery_graph_text(graph),
    }
}
