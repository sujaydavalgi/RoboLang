//! CLI for mission deployment risk scoring.
//!
use spanda_lexer::tokenize;
use spanda_parser::parse;
use spanda_risk::{evaluate_mission_risk, format_mission_risk, MissionRiskFormat};
use std::fs;
use std::path::Path;
use std::process;

fn parse_program(path: &Path) -> (spanda_ast::nodes::Program, String) {
    let source = fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("Failed to read {}: {e}", path.display());
        process::exit(1);
    });
    let tokens = tokenize(&source).unwrap_or_else(|e| {
        eprintln!("{e}");
        process::exit(1);
    });
    let program = parse(tokens).unwrap_or_else(|e| {
        eprintln!("{e}");
        process::exit(1);
    });
    (program, source)
}

fn file_arg(args: &[String]) -> String {
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--json" => index += 1,
            other if !other.starts_with('-') => return other.to_string(),
            _ => index += 1,
        }
    }
    eprintln!("Usage: spanda risk <file.sd> [--json]");
    process::exit(1);
}

/// `spanda risk <file.sd> [--json]`
pub fn risk_dispatch(args: &[String]) {
    let file = file_arg(args);
    let path = Path::new(&file);
    let (program, source) = parse_program(path);
    let report = evaluate_mission_risk(&program, &source, &file);
    let format = if args.iter().any(|a| a == "--json") {
        MissionRiskFormat::Json
    } else {
        MissionRiskFormat::Text
    };
    println!("{}", format_mission_risk(&report, format));
    if report.score.tier == "critical" {
        process::exit(1);
    }
}
