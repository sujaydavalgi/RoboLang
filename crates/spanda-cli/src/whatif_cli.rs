//! CLI for what-if failure scenario analysis.
//!
use spanda_lexer::tokenize;
use spanda_parser::parse;
use spanda_whatif::{format_what_if_report, run_what_if_analysis, WhatIfFormat, WhatIfOptions};
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
            "--scenario" => index += 2,
            "--json" | "--all" => index += 1,
            other if !other.starts_with('-') => return other.to_string(),
            _ => index += 1,
        }
    }
    eprintln!("Usage: spanda what-if <file.sd> [--scenario gps_failure] [--all] [--json]");
    process::exit(1);
}

fn parse_scenario(args: &[String]) -> Option<String> {
    for (index, arg) in args.iter().enumerate() {
        if arg == "--scenario" {
            return args.get(index + 1).cloned();
        }
    }
    None
}

/// `spanda what-if <file.sd> [--scenario gps_failure] [--all] [--json]`
pub fn what_if_dispatch(args: &[String]) {
    let file = file_arg(args);
    let path = Path::new(&file);
    let program = parse_program(path);
    let scenario = parse_scenario(args);
    let report = run_what_if_analysis(
        &program,
        &file,
        &WhatIfOptions {
            scenarios: scenario.into_iter().collect(),
            all: args.iter().any(|a| a == "--all"),
        },
    );
    let format = if args.iter().any(|a| a == "--json") {
        WhatIfFormat::Json
    } else {
        WhatIfFormat::Text
    };
    println!("{}", format_what_if_report(&report, format));
}
