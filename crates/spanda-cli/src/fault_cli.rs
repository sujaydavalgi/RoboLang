//! CLI commands for runtime fault detection and health reporting.

use spanda_lexer::tokenize;
use spanda_parser::parse;
use spanda_readiness::types::ReportFormat;
use spanda_readiness::{evaluate_readiness, ReadinessOptions};
use spanda_runtime::replay::MissionTrace;
use spanda_runtime_faults::{
    apply_fault_readiness_impact, diagnose_fault_report, format_fault_report,
    format_runtime_health, format_trace_faults, scan_program_faults, FaultScanOptions,
};
use std::fs;
use std::process;

fn read_file(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("Failed to read {path}: {e}");
        process::exit(1);
    })
}

fn parse_program(source: &str) -> spanda_ast::nodes::Program {
    let tokens = tokenize(source).unwrap_or_else(|e| {
        eprintln!("{e}");
        process::exit(1);
    });
    parse(tokens).unwrap_or_else(|e| {
        eprintln!("{e}");
        process::exit(1);
    })
}

fn parse_format(args: &[String]) -> ReportFormat {
    if args.iter().any(|a| a == "--json") {
        ReportFormat::Json
    } else if args.iter().any(|a| a == "--html") {
        ReportFormat::Html
    } else if args.iter().any(|a| a == "--markdown") {
        ReportFormat::Markdown
    } else {
        ReportFormat::Text
    }
}

fn scan_options_from_args(args: &[String]) -> FaultScanOptions {
    FaultScanOptions {
        inject_crash: args.iter().any(|a| a == "--inject-crash"),
        inject_memory_leak: args.iter().any(|a| a == "--inject-memory-leak"),
        inject_reboot: args.iter().any(|a| a == "--inject-reboot"),
        inject_heartbeat_loss: args.iter().any(|a| a == "--inject-heartbeat-loss"),
        inject_resource_pressure: args.iter().any(|a| a == "--inject-resource-pressure"),
        sim_time_ms: 0.0,
    }
}

/// `spanda fault scan <file.sd>`
pub fn cmd_fault_scan(args: &[String]) {
    let file = args
        .iter()
        .find(|a| !a.starts_with('-'))
        .unwrap_or_else(|| {
            eprintln!("Usage: spanda fault scan <file.sd> [--json] [--inject-crash]");
            process::exit(1);
        });
    let source = read_file(file);
    let program = parse_program(&source);
    let options = scan_options_from_args(args);
    let report = scan_program_faults(&program, file, &options);
    let format = parse_format(args);
    println!("{}", format_fault_report(&report, format));
    if !report.passed {
        process::exit(1);
    }
}

/// `spanda runtime health <file.sd>`
pub fn cmd_runtime_health(args: &[String]) {
    let file = args
        .iter()
        .find(|a| !a.starts_with('-'))
        .unwrap_or_else(|| {
            eprintln!("Usage: spanda runtime health <file.sd> [--json]");
            process::exit(1);
        });
    let source = read_file(file);
    let program = parse_program(&source);
    let options = scan_options_from_args(args);
    let report = scan_program_faults(&program, file, &options);
    if args.iter().any(|a| a == "--json") {
        println!(
            "{}",
            serde_json::to_string_pretty(&report.health).unwrap_or_default()
        );
    } else {
        println!("{}", format_runtime_health(&report));
    }
}

/// `spanda runtime diagnose <trace>`
pub fn cmd_runtime_diagnose(args: &[String]) {
    let trace_file = args
        .iter()
        .find(|a| !a.starts_with('-'))
        .unwrap_or_else(|| {
            eprintln!("Usage: spanda runtime diagnose <mission.trace> [--json]");
            process::exit(1);
        });
    let trace = MissionTrace::load(trace_file).unwrap_or_else(|e| {
        eprintln!("{e}");
        process::exit(1);
    });
    let timeline = spanda_runtime_faults::faults_from_trace(&trace);
    if args.iter().any(|a| a == "--json") {
        println!(
            "{}",
            serde_json::to_string_pretty(&timeline).unwrap_or_default()
        );
    } else {
        println!("{}", format_trace_faults(&trace));
    }
}

/// `spanda fault report <file.sd>`
pub fn cmd_fault_report(args: &[String]) {
    let file = args
        .iter()
        .find(|a| !a.starts_with('-'))
        .unwrap_or_else(|| {
            eprintln!("Usage: spanda fault report <file.sd> [--json|--html|--markdown]");
            process::exit(1);
        });
    let source = read_file(file);
    let program = parse_program(&source);
    let options = scan_options_from_args(args);
    let fault_report = scan_program_faults(&program, file, &options);
    let mut readiness = evaluate_readiness(&program, &ReadinessOptions::default());
    apply_fault_readiness_impact(&mut readiness, &fault_report);
    let diagnoses = diagnose_fault_report(&fault_report);
    let format = parse_format(args);
    if format == ReportFormat::Json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "faults": fault_report,
                "readiness": readiness,
                "diagnoses": diagnoses,
            }))
            .unwrap_or_default()
        );
    } else {
        println!("{}", format_fault_report(&fault_report, format));
        println!("\nReadiness impact: {:?}", readiness.status);
        for d in &diagnoses {
            println!("  Diagnosis: {} — {}", d.what, d.likely_cause);
        }
    }
}

/// Dispatch `spanda fault` subcommands.
pub fn fault_dispatch(args: &[String]) {
    let sub = args.first().map(String::as_str).unwrap_or("");
    match sub {
        "scan" => cmd_fault_scan(&args[1..]),
        "report" => cmd_fault_report(&args[1..]),
        _ => {
            eprintln!("Usage: spanda fault scan|report <file.sd> [options]");
            process::exit(1);
        }
    }
}

/// Dispatch `spanda runtime` subcommands.
pub fn runtime_dispatch(args: &[String]) {
    let sub = args.first().map(String::as_str).unwrap_or("");
    match sub {
        "health" => cmd_runtime_health(&args[1..]),
        "diagnose" => cmd_runtime_diagnose(&args[1..]),
        _ => {
            eprintln!("Usage: spanda runtime health|diagnose <file> [options]");
            process::exit(1);
        }
    }
}
