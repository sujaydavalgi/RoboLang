//! CLI for chaos engineering experiments.
//!
use spanda_chaos::{format_chaos_report, run_chaos_experiment, ChaosExperimentOptions, ChaosFormat};
use spanda_lexer::tokenize;
use spanda_parser::parse;
use std::fs;
use std::path::Path;
use std::process;

fn parse_program(path: &Path) -> spanda_ast::nodes::Program {
    let source = fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("Failed to read {}: {e}", path.display());
        process::exit(1);
    });
    let tokens = tokenize(&source).unwrap_or_else(|e| {
        eprintln!("{e}");
        process::exit(1);
    });
    parse(tokens).unwrap_or_else(|e| {
        eprintln!("{e}");
        process::exit(1);
    })
}

fn file_arg(args: &[String]) -> String {
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--inject" => index += 2,
            "--json" => index += 1,
            other if !other.starts_with('-') => return other.to_string(),
            _ => index += 1,
        }
    }
    eprintln!("Usage: spanda chaos <file.sd> [--inject gps-failure,...] [--json]");
    process::exit(1);
}

fn parse_injections(args: &[String]) -> Vec<String> {
    for (index, arg) in args.iter().enumerate() {
        if arg == "--inject" {
            if let Some(value) = args.get(index + 1) {
                return value
                    .split(',')
                    .map(str::trim)
                    .filter(|part| !part.is_empty())
                    .map(str::to_string)
                    .collect();
            }
        }
    }
    Vec::new()
}

/// `spanda chaos <file.sd> [--inject gps-failure,...] [--json]`
pub fn chaos_dispatch(args: &[String]) {
    let file = file_arg(args);
    let path = Path::new(&file);
    let program = parse_program(path);
    let report = run_chaos_experiment(
        &program,
        &file,
        &ChaosExperimentOptions {
            injections: parse_injections(args),
        },
    );
    let format = if args.iter().any(|a| a == "--json") {
        ChaosFormat::Json
    } else {
        ChaosFormat::Text
    };
    println!("{}", format_chaos_report(&report, format));
    if !report.passed {
        process::exit(1);
    }
}
