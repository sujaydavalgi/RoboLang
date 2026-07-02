//! Simulation scenarios for distributed decisions.

use crate::engine::{evaluate_distributed_decisions, DecisionContext, DistributedDecisionReport};
use crate::types::DecisionLayer;
use serde::{Deserialize, Serialize};
use spanda_ast::nodes::Program;
use std::collections::HashMap;

/// Simulation scenario flags.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SimulationOptions {
    pub offline: bool,
    pub network_partition: bool,
    pub fleet_coordinator_failure: bool,
    pub entity_id: String,
    pub mission: Option<String>,
    pub signals: HashMap<String, bool>,
}

/// Result of a distributed decision simulation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimulationReport {
    pub scenario: String,
    pub options: SimulationOptions,
    pub report: DistributedDecisionReport,
    pub notes: Vec<String>,
}

/// Simulate distributed decisions under various failure scenarios.
pub fn simulate_distributed_decisions(
    program: &Program,
    options: SimulationOptions,
) -> SimulationReport {
    // Description:
    //     Run decision evaluation with scenario-specific context overrides.
    //
    // Parameters:
    // - `program` — parsed Spanda program
    // - `options` — simulation flags and signals
    //
    // Returns:
    // Simulation report with scenario notes.
    //
    // Options:
    // None.
    //
    // Example:
    // let sim = simulate_distributed_decisions(&program, SimulationOptions::default());

    let mut notes = Vec::new();
    let mut layer = DecisionLayer::LocalEntity;
    let mut offline_minutes = 0u32;
    let mut signals = options.signals.clone();

    if options.offline {
        offline_minutes = 15;
        notes.push("Simulating offline operation (15 min since last sync)".into());
    }
    if options.network_partition {
        layer = DecisionLayer::LocalEntity;
        signals.insert("network.partitioned".into(), true);
        notes.push("Simulating network partition — central unreachable".into());
    }
    if options.fleet_coordinator_failure {
        signals.insert("fleet.coordinator.failed".into(), true);
        notes.push("Simulating fleet coordinator failure — backup promotion expected".into());
    }

    let scenario = if options.offline {
        "offline"
    } else if options.network_partition {
        "network_partition"
    } else if options.fleet_coordinator_failure {
        "fleet_coordinator_failure"
    } else {
        "nominal"
    };

    let context = DecisionContext {
        entity_id: options.entity_id.clone(),
        mission: options.mission.clone(),
        layer,
        action: "continue_mission".into(),
        signals,
        offline_minutes,
        policy_version: "1.0.0".into(),
    };

    let report = evaluate_distributed_decisions(program, &context);
    if !report.passed {
        notes.push("Simulation detected policy or authority violations".into());
    }

    SimulationReport {
        scenario: scenario.into(),
        options,
        report,
        notes,
    }
}

/// Format simulation report for CLI.
pub fn format_simulation_report(sim: &SimulationReport, json: bool) -> String {
    if json {
        return serde_json::to_string_pretty(sim).unwrap_or_default();
    }
    let mut out = format!("Decision simulation: {}\n", sim.scenario);
    for n in &sim.notes {
        out.push_str(&format!("  NOTE: {n}\n"));
    }
    out.push_str(&crate::engine::format_distributed_report(
        &sim.report,
        false,
    ));
    out
}
