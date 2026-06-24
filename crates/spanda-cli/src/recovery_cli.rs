//! CLI commands for self-healing and recovery framework.

use spanda_assurance::{
    analyze_failure_with_recovery, diagnose_from_trace, evaluate_recovery, format_recovery,
    load_merged_recovery_knowledge, recovery_from_diagnosis, simulate_failure_recovery,
    RecoveryContext, RecoveryLevel, RecoveryReport,
};
use spanda_lexer::tokenize;
use spanda_parser::parse;
use spanda_readiness::{format_failure_analysis, ReportFormat};
use std::fs;
use std::path::Path;
use std::process;

const MINIMAL_PROGRAM: &str = "robot Placeholder { behavior idle() {} }";

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
    } else if args.iter().any(|a| a == "--markdown") {
        ReportFormat::Markdown
    } else if args.iter().any(|a| a == "--html") {
        ReportFormat::Html
    } else {
        ReportFormat::Text
    }
}

fn file_arg(args: &[String]) -> String {
    args.iter()
        .find(|a| !a.starts_with('-'))
        .cloned()
        .unwrap_or_else(|| {
            eprintln!("Missing file path");
            process::exit(1);
        })
}

fn failure_arg(args: &[String]) -> Option<String> {
    args.iter()
        .position(|a| a == "--inject-failure" || a == "--failure")
        .and_then(|i| args.get(i + 1).cloned())
}

fn build_report(file: &str, args: &[String]) -> RecoveryReport {
    if file.ends_with(".trace") {
        let diagnosis = diagnose_from_trace(Path::new(file)).unwrap_or_else(|e| {
            eprintln!("{e}");
            process::exit(1);
        });
        let sd_path = file.replacen(".trace", ".sd", 1);
        let program = if Path::new(&sd_path).exists() {
            parse_program(&read_file(&sd_path))
        } else {
            parse_program(MINIMAL_PROGRAM)
        };
        recovery_from_diagnosis(&program, &diagnosis)
    } else {
        let source = read_file(file);
        let program = parse_program(&source);
        if let Some(failure) = failure_arg(args) {
            simulate_failure_recovery(&program, &failure)
        } else {
            evaluate_recovery(&program, None)
        }
    }
}

/// `spanda heal <file.sd|mission.trace> [--json|--markdown|--html] [--failure <kind>]`
pub fn cmd_heal(args: &[String]) {
    let format = parse_format(args);
    let file = file_arg(args);
    let report = build_report(&file, args);
    println!("{}", format_recovery(&report, format));
    if !report.passed {
        process::exit(1);
    }
}

/// `spanda recover <file.sd> [--json] [--failure <kind>]`
pub fn cmd_recover(args: &[String]) {
    let format = parse_format(args);
    let file = file_arg(args);
    let source = read_file(&file);
    let program = parse_program(&source);
    let report = if let Some(failure) = failure_arg(args) {
        simulate_failure_recovery(&program, &failure)
    } else {
        let ctx = RecoveryContext {
            issue: "gps.failed".into(),
            diagnosis: Some("Satellite lock lost".into()),
            classification: None,
            level: RecoveryLevel::Level3AutomaticWithValidation,
        };
        evaluate_recovery(&program, Some(&ctx))
    };
    println!("{}", format_recovery(&report, format));
    if !report.passed {
        process::exit(1);
    }
}

/// `spanda recovery-report <file.sd> [--json|--markdown|--html]`
pub fn cmd_recovery_report(args: &[String]) {
    let format = parse_format(args);
    let file = file_arg(args);
    let source = read_file(&file);
    let program = parse_program(&source);
    let report = evaluate_recovery(&program, None);
    println!("{}", format_recovery(&report, format));
    if !report.passed {
        process::exit(1);
    }
}

/// `spanda recovery knowledge <file.sd> [--json]`
pub fn cmd_recovery_knowledge(args: &[String]) {
    let file = file_arg(args);
    let source = read_file(&file);
    let program = parse_program(&source);
    let kb = load_merged_recovery_knowledge(&program);
    if args.iter().any(|a| a == "--json") {
        println!("{}", serde_json::to_string_pretty(&kb).unwrap_or_default());
    } else {
        for entry in &kb.entries {
            println!(
                "{} -> {} ({:.0}% success)\n  {}",
                entry.failure_pattern,
                entry.recovery_pattern,
                entry.success_rate * 100.0,
                entry.recommendation
            );
        }
        if kb.entries.is_empty() {
            println!("No recovery knowledge entries.");
        }
    }
}

/// Dispatch `spanda recovery` subcommands.
pub fn recovery_dispatch(args: &[String]) {
    match args.first().map(String::as_str).unwrap_or("") {
        "plan" | "report" => cmd_recovery_report(&args[1..]),
        "knowledge" => cmd_recovery_knowledge(&args[1..]),
        _ => {
            eprintln!("Usage: spanda recovery plan|report|knowledge <file.sd>");
            process::exit(1);
        }
    }
}

/// Extended failure analysis with recovery planning (`--with-recovery`).
pub fn cmd_analyze_failure_recovery(args: &[String]) {
    let file = file_arg(args);
    let source = read_file(&file);
    let program = parse_program(&source);
    let report = analyze_failure_with_recovery(&program);
    if args.iter().any(|a| a == "--json") {
        println!(
            "{}",
            serde_json::to_string_pretty(&report).unwrap_or_default()
        );
    } else {
        let mut out = format_failure_analysis(&report.failure);
        out.push_str("\nRecovery Plans:\n");
        for plan in &report.recovery_plans {
            out.push_str(&format!(
                "  Failure: {}\n  Impact: see above\n  Fallback: {}\n  Risk: {}\n",
                plan.failure,
                plan.actions
                    .first()
                    .map(|a| a.description.as_str())
                    .unwrap_or("none"),
                plan.risk
            ));
            for action in &plan.actions {
                out.push_str(&format!("    - {}\n", action.description));
            }
        }
        out.push_str(&format!("\nOverall Risk: {}\n", report.risk));
        println!("{out}");
    }
}
