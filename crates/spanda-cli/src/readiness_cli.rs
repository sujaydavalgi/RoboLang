//! CLI commands for operational readiness and mission assurance.

use spanda_lexer::tokenize;
use spanda_parser::parse;
use spanda_readiness::{
    analyze_failure, audit_program, build_runtime_context,
    evaluate_fleet_readiness, evaluate_readiness_with_runtime, evaluate_twin_readiness,
    format_audit, format_failure_analysis, format_fleet_readiness, format_mission_verification,
    format_readiness, format_safety_report, generate_safety_report,
    readiness_options_from_flags, verify_approvals, verify_fleet, verify_mission, ReadinessOptions,
    ReportFormat,
};
use std::fs;
use std::path::Path;
use std::process;

struct ParsedReadinessCli {
    format: ReportFormat,
    file: String,
    options: ReadinessOptions,
    agent_json: bool,
}

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

fn parse_readiness_cli(args: &[String]) -> ParsedReadinessCli {
    let format = parse_format(args);
    let mut target: Option<String> = None;
    let mut include_runtime = false;
    let mut inject_health_faults = false;
    let mut simulate = false;
    let mut strict = false;
    let mut agent_json = false;
    let mut file: Option<String> = None;
    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--json" | "--markdown" | "--html" | "--agent-json" => {
                if args[i].as_str() == "--agent-json" {
                    agent_json = true;
                }
            }
            "--target" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("--target requires a hardware profile name");
                    process::exit(1);
                }
                target = Some(args[i].clone());
            }
            "--runtime" => include_runtime = true,
            "--inject-health-faults" => {
                include_runtime = true;
                inject_health_faults = true;
            }
            "--simulate" => simulate = true,
            "--strict" => strict = true,
            other if !other.starts_with('-') && file.is_none() => file = Some(other.to_string()),
            other => {
                eprintln!("Unknown argument: {other}");
                process::exit(1);
            }
        }
        i += 1;
    }
    let file = file.unwrap_or_else(|| {
        eprintln!("Missing file path");
        process::exit(1);
    });
    let source = read_file(&file);
    let program = parse_program(&source);
    let options = readiness_options_from_flags(
        &program,
        target,
        include_runtime,
        inject_health_faults,
        simulate,
        strict,
    );
    ParsedReadinessCli {
        format,
        file,
        options,
        agent_json,
    }
}

fn evaluate_with_options(
    program: &spanda_ast::nodes::Program,
    options: &ReadinessOptions,
) -> spanda_readiness::ReadinessReport {
    let runtime = options
        .include_runtime
        .then(|| build_runtime_context(program, options.inject_health_faults));
    evaluate_readiness_with_runtime(program, options, runtime.as_ref())
}

/// `spanda readiness <file.sd> [--target T] [--runtime] [--inject-health-faults] [--json|--agent-json]`
pub fn cmd_readiness(args: &[String]) {
    let parsed = parse_readiness_cli(args);
    let source = read_file(&parsed.file);

    // Emit the same JSON envelope as deploy/fleet `GET /v1/readiness`.
    if parsed.agent_json {
        let body = evaluate_agent_readiness_json(
            &source,
            parsed.options.target.as_deref(),
            parsed.options.include_runtime,
            parsed.options.inject_health_faults,
        )
        .unwrap_or_else(|e| {
            eprintln!("{e}");
            process::exit(1);
        });
        println!("{body}");
        let mission_ready = serde_json::from_str::<serde_json::Value>(&body)
            .ok()
            .and_then(|value| value.get("mission_ready").and_then(|m| m.as_bool()))
            .unwrap_or(false);
        if !mission_ready {
            process::exit(1);
        }
        return;
    }

    let program = parse_program(&source);
    let report = evaluate_with_options(&program, &parsed.options);
    println!("{}", format_readiness(&report, parsed.format));
    if !report.mission_ready {
        process::exit(1);
    }
}

/// `spanda verify mission <file.sd> [--target T] [--json]`
pub fn cmd_verify_mission(args: &[String]) {
    let parsed = parse_readiness_cli(args);
    let source = read_file(&parsed.file);
    let program = parse_program(&source);
    let reports = verify_mission(&program, parsed.options.target.as_deref());
    if parsed.format == ReportFormat::Json {
        println!("{}", serde_json::to_string_pretty(&reports).unwrap());
    } else {
        print!("{}", format_mission_verification(&reports));
    }
    if reports.iter().any(|r| !r.achievable) {
        process::exit(1);
    }
}

/// `spanda analyze-failure <file.sd>`
pub fn cmd_analyze_failure(args: &[String]) {
    let json = args.iter().any(|a| a == "--json");
    let file = args
        .iter()
        .find(|a| !a.starts_with('-'))
        .cloned()
        .unwrap_or_else(|| {
            eprintln!("Missing file path");
            process::exit(1);
        });
    let source = read_file(&file);
    let program = parse_program(&source);
    let report = analyze_failure(&program);
    if json {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
    } else {
        print!("{}", format_failure_analysis(&report));
    }
}

/// `spanda safety-report <file.sd>`
pub fn cmd_safety_report(args: &[String]) {
    let parsed = parse_readiness_cli(args);
    let source = read_file(&parsed.file);
    let program = parse_program(&source);
    let report = generate_safety_report(&program, &parsed.file);
    println!("{}", format_safety_report(&report, parsed.format));
    if !report.deployable {
        process::exit(1);
    }
}

/// `spanda twin readiness <file.sd> [--trace <path>]`
pub fn cmd_twin_readiness(args: &[String]) {
    let json = args.iter().any(|a| a == "--json");
    let file = args
        .iter()
        .find(|a| !a.starts_with('-') && *a != "--trace")
        .cloned()
        .unwrap_or_else(|| {
            eprintln!("Missing file path");
            process::exit(1);
        });
    let trace_path = args
        .windows(2)
        .find(|w| w[0] == "--trace")
        .map(|w| w[1].clone());
    let source = read_file(&file);
    let program = parse_program(&source);
    let status = evaluate_twin_readiness(&program, trace_path.as_deref().map(Path::new));
    if json {
        println!("{}", serde_json::to_string_pretty(&status).unwrap());
    } else {
        print!("{}", spanda_readiness::format_twin_readiness(&status));
    }
}

/// `spanda fleet readiness <file.sd> [--target T] [--runtime]`
pub fn cmd_fleet_readiness(args: &[String]) {
    let parsed = parse_readiness_cli(args);
    let source = read_file(&parsed.file);
    let program = parse_program(&source);
    let report = evaluate_fleet_readiness(&program, &parsed.options);
    if parsed.format == ReportFormat::Json {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
    } else {
        print!("{}", format_fleet_readiness(&report));
    }
}

/// `spanda audit <file.sd>`
pub fn cmd_audit(args: &[String]) {
    let json = args.iter().any(|a| a == "--json");
    let file = args
        .iter()
        .find(|a| !a.starts_with('-'))
        .cloned()
        .unwrap_or_else(|| {
            eprintln!("Missing file path");
            process::exit(1);
        });
    let source = read_file(&file);
    let program = parse_program(&source);
    let report = audit_program(&program, &source);
    if json {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
    } else {
        print!("{}", format_audit(&report));
    }
    if report.critical_count > 0 {
        process::exit(1);
    }
}

/// `spanda verify-fleet <file.sd>`
pub fn cmd_verify_fleet(args: &[String]) {
    let json = args.iter().any(|a| a == "--json");
    let file = args
        .iter()
        .find(|a| !a.starts_with('-'))
        .cloned()
        .unwrap_or_else(|| {
            eprintln!("Missing file path");
            process::exit(1);
        });
    let source = read_file(&file);
    let program = parse_program(&source);
    let report = verify_fleet(&program);
    if json {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
    } else {
        for f in &report.findings {
            println!("[{}] {} — {}", f.severity, f.category, f.message);
        }
    }
    if !report.compatible {
        process::exit(1);
    }
}

/// `spanda verify-approval <file.sd>`
pub fn cmd_verify_approval(args: &[String]) {
    let json = args.iter().any(|a| a == "--json");
    let file = args
        .iter()
        .find(|a| !a.starts_with('-'))
        .cloned()
        .unwrap_or_else(|| {
            eprintln!("Missing file path");
            process::exit(1);
        });
    let source = read_file(&file);
    let program = parse_program(&source);
    let report = verify_approvals(&program);
    if json {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
    } else {
        for row in &report.rows {
            println!(
                "{} / {} — path:{} actor:{} fallback:{} [{}]",
                row.actor,
                row.action,
                row.approval_path_exists,
                row.actor_exists,
                row.fallback_exists,
                row.status
            );
        }
    }
    if !report.compatible {
        process::exit(1);
    }
}

/// Agent-shaped readiness JSON for CLI and service mirrors (`GET /v1/readiness`).
pub fn evaluate_agent_readiness_json(
    source: &str,
    target: Option<&str>,
    include_runtime: bool,
    inject_health_faults: bool,
) -> Result<String, String> {
    // Delegate to the shared readiness crate used by deploy and fleet agents.
    //
    // Parameters:
    // - `source` — deployed `.sd` program text
    // - `target` — optional hardware/deploy profile override
    // - `include_runtime` — fold live runtime health into the score
    // - `inject_health_faults` — simulate degraded sensors (requires runtime)
    //
    // Returns:
    // JSON `{"ok":true,"mission_ready":...,"readiness":...}` or an error string.
    //
    // Options:
    // None.
    //
    // Example:
    // let body = evaluate_agent_readiness_json(program, Some("RoverV1"), true, false)?;

    spanda_readiness::evaluate_agent_readiness_json(
        source,
        target,
        include_runtime,
        inject_health_faults,
    )
}

/// Top-level readiness dispatch for subcommands.
pub fn readiness_dispatch(args: &[String]) {
    if args.is_empty() {
        eprintln!(
            "Usage: spanda readiness <file.sd> [--target <profile>] [--runtime] [--inject-health-faults] [--json|--agent-json|--markdown|--html]"
        );
        process::exit(1);
    }
    cmd_readiness(args);
}
