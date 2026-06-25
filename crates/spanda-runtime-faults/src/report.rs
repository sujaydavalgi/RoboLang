//! Fault report formatting (text, JSON, HTML).

use crate::types::{FaultScanReport, RuntimeHealthStatus};
use spanda_readiness::types::ReportFormat;

/// Format a fault scan report in the requested format.
pub fn format_fault_report(report: &FaultScanReport, format: ReportFormat) -> String {
    // Format a fault scan report as text, JSON, or HTML.
    //
    // Parameters:
    // - `report` — fault scan report to format
    // - `format` — output format (Text, Json, Markdown, Html)
    //
    // Returns:
    // Formatted report string.
    //
    // Options:
    // None.
    //
    // Example:
    // let output = format_fault_report(&report, ReportFormat::Json);

    match format {
        ReportFormat::Json => serde_json::to_string_pretty(report).unwrap_or_default(),
        ReportFormat::Html => format_html(report),
        ReportFormat::Markdown => format_markdown(report),
        ReportFormat::Text => format_text(report),
    }
}

/// Format runtime health summary.
pub fn format_runtime_health(report: &FaultScanReport) -> String {
    // Format a concise runtime health summary.
    //
    // Parameters:
    // - `report` — fault scan report with health data
    //
    // Returns:
    // Human-readable health summary.
    //
    // Options:
    // None.
    //
    // Example:
    // let summary = format_runtime_health(&report);

    let h = &report.health;
    let mut lines = vec![
        format!("Runtime Health: {}", h.overall.as_str()),
        format!("Uptime: {:.0}ms", h.uptime_ms),
        format!("Active faults: {}", h.active_faults.len()),
        format!(
            "Monitors: {} heartbeat, {} memory, {} resource, {} restart",
            report.heartbeats_configured,
            report.memory_watches_configured,
            report.resource_watches_configured,
            report.restart_policies_configured
        ),
    ];
    for fault in &h.active_faults {
        lines.push(format!(
            "  [{}] {} — {}",
            status_icon(fault.status),
            fault.kind.as_str(),
            fault.message
        ));
    }
    lines.join("\n")
}

fn format_text(report: &FaultScanReport) -> String {
    let mut out = format_runtime_health(report);
    out.push_str("\n\nFault Timeline:\n");
    for entry in &report.timeline {
        out.push_str(&format!(
            "  {:.0}ms  {}  {}  {:?}\n",
            entry.timestamp_ms, entry.event, entry.target, entry.status
        ));
    }
    out.push_str(&format!(
        "\nScan result: {}\n",
        if report.passed { "PASSED" } else { "FAILED" }
    ));
    out
}

fn format_markdown(report: &FaultScanReport) -> String {
    let mut out = format!(
        "# Runtime Fault Report\n\n**Source:** {}\n\n",
        report.source
    );
    out.push_str(&format!(
        "**Overall:** {} | **Passed:** {}\n\n",
        report.health.overall.as_str(),
        report.passed
    ));
    out.push_str("## Active Faults\n\n");
    if report.faults.is_empty() {
        out.push_str("_No faults detected._\n");
    } else {
        out.push_str("| Kind | Target | Status | Message |\n");
        out.push_str("|------|--------|--------|----------|\n");
        for f in &report.faults {
            out.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                f.kind.as_str(),
                f.target,
                f.status.as_str(),
                f.message
            ));
        }
    }
    out.push_str("\n## Timeline\n\n");
    for entry in &report.timeline {
        out.push_str(&format!(
            "- {:.0}ms: **{}** on `{}` — {:?}\n",
            entry.timestamp_ms, entry.event, entry.target, entry.status
        ));
    }
    out
}

fn format_html(report: &FaultScanReport) -> String {
    let status_class = if report.passed { "passed" } else { "failed" };
    let mut fault_rows = String::new();
    for f in &report.faults {
        fault_rows.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
            f.kind.as_str(),
            html_escape(&f.target),
            f.status.as_str(),
            html_escape(&f.message)
        ));
    }
    let mut timeline = String::new();
    for entry in &report.timeline {
        timeline.push_str(&format!(
            "<li>{:.0}ms: <strong>{}</strong> on <code>{}</code></li>\n",
            entry.timestamp_ms,
            html_escape(&entry.event),
            html_escape(&entry.target)
        ));
    }
    format!(
        r#"<!DOCTYPE html>
<html><head><meta charset="utf-8"><title>Runtime Fault Report</title>
<style>
body {{ font-family: system-ui, sans-serif; margin: 2rem; }}
.{status_class} {{ color: {color}; font-weight: bold; }}
table {{ border-collapse: collapse; width: 100%; }}
th, td {{ border: 1px solid #ccc; padding: 0.5rem; text-align: left; }}
th {{ background: #f5f5f5; }}
</style></head><body>
<h1>Runtime Fault Report</h1>
<p>Source: <code>{source}</code></p>
<p>Overall: <span class="{status_class}">{overall}</span></p>
<h2>Active Faults</h2>
<table><tr><th>Kind</th><th>Target</th><th>Status</th><th>Message</th></tr>
{fault_rows}</table>
<h2>Timeline</h2>
<ul>{timeline}</ul>
</body></html>"#,
        status_class = status_class,
        color = if report.passed { "green" } else { "red" },
        source = html_escape(&report.source),
        overall = report.health.overall.as_str(),
        fault_rows = fault_rows,
        timeline = timeline,
    )
}

fn status_icon(status: RuntimeHealthStatus) -> &'static str {
    match status {
        RuntimeHealthStatus::Healthy => "OK",
        RuntimeHealthStatus::Warning => "WARN",
        RuntimeHealthStatus::Degraded => "DEG",
        RuntimeHealthStatus::Critical => "CRIT",
        RuntimeHealthStatus::Crashed => "CRASH",
        RuntimeHealthStatus::Rebooted => "BOOT",
        RuntimeHealthStatus::Unknown => "?",
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
