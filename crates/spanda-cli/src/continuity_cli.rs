//! CLI commands for mission continuity, takeover, delegation, and succession.

use spanda_assurance::{
    evaluate_continuity, format_continuity, format_delegation, format_succession, format_takeover,
    parse_scope, parse_trigger, plan_delegation, plan_succession, plan_takeover, ContinuityContext,
    SuccessionScope,
};
use spanda_lexer::tokenize;
use spanda_parser::parse;
use spanda_readiness::ReportFormat;
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
        .find(|a| !a.starts_with('-') && !a.contains('='))
        .cloned()
        .unwrap_or_else(|| {
            eprintln!("Missing file path");
            process::exit(1);
        })
}

fn flag_value(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1).cloned())
        .or_else(|| {
            args.iter()
                .find_map(|a| a.strip_prefix(&format!("{flag}=")).map(|v| v.to_string()))
        })
}

fn build_context(args: &[String], program: &spanda_ast::nodes::Program) -> ContinuityContext {
    let failed = flag_value(args, "--failed")
        .or_else(|| flag_value(args, "--failed-robot"))
        .unwrap_or_else(|| "Rover".into());
    let mission = flag_value(args, "--mission").unwrap_or_else(|| {
        let spanda_ast::nodes::Program::Program { mission_plans, .. } = program;
        mission_plans
            .first()
            .map(|p| {
                let spanda_ast::assurance_decl::MissionPlanDecl::MissionPlanDecl { name, .. } = p;
                name.clone()
            })
            .unwrap_or_else(|| "default_mission".into())
    });
    let progress: f64 = flag_value(args, "--progress")
        .and_then(|p| p.parse().ok())
        .unwrap_or(0.0);
    let trigger = flag_value(args, "--trigger")
        .map(|t| parse_trigger(&t))
        .unwrap_or(spanda_assurance::ContinuityTrigger::RobotFailed);
    let scope = flag_value(args, "--scope")
        .map(|s| parse_scope(&s))
        .unwrap_or(SuccessionScope::Robot);
    let current_step = flag_value(args, "--step");
    let checkpoints: Vec<String> = flag_value(args, "--checkpoints")
        .map(|c| c.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    ContinuityContext {
        mission,
        failed_entity: failed,
        trigger,
        progress_percent: progress,
        scope,
        current_step,
        checkpoints,
    }
}

/// `spanda continuity <file.sd> [--failed <name>] [--progress <pct>] [--trigger <kind>] [--json]`
pub fn cmd_continuity(args: &[String]) {
    let path = file_arg(args);
    let source = read_file(&path);
    let program = parse_program(&source);
    let context = build_context(args, &program);
    let report = evaluate_continuity(&program, &context);
    let format = parse_format(args);
    print!("{}", format_continuity(&report, format));
    if !report.passed {
        process::exit(1);
    }
}

/// `spanda takeover <file.sd> [--failed <name>] [--successor <name>] [--progress <pct>]`
pub fn cmd_takeover(args: &[String]) {
    let path = file_arg(args);
    let source = read_file(&path);
    let program = parse_program(&source);
    let context = build_context(args, &program);
    let successor = flag_value(args, "--successor");
    let report = plan_takeover(&program, &context, successor.as_deref());
    let format = parse_format(args);
    print!("{}", format_takeover(&report, format));
    if !report.succeeded {
        process::exit(1);
    }
}

/// `spanda delegate <file.sd> [--failed <name>] [--to <name>] [--progress <pct>]`
pub fn cmd_delegate(args: &[String]) {
    let path = file_arg(args);
    let source = read_file(&path);
    let program = parse_program(&source);
    let context = build_context(args, &program);
    let to = flag_value(args, "--to");
    let report = plan_delegation(&program, &context, to.as_deref());
    let format = parse_format(args);
    print!("{}", format_delegation(&report, format));
    if !report.passed {
        process::exit(1);
    }
}

/// `spanda succession <file.sd> [--failed <name>] [--scope fleet|swarm|robot]`
pub fn cmd_succession(args: &[String]) {
    let path = file_arg(args);
    let source = read_file(&path);
    let program = parse_program(&source);
    let context = build_context(args, &program);
    let report = plan_succession(&program, &context);
    let format = parse_format(args);
    print!("{}", format_succession(&report, format));
    if !report.passed {
        process::exit(1);
    }
}
