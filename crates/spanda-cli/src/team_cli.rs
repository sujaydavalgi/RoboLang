//! CLI for human/robot teaming verification.

use spanda_lexer::tokenize;
use spanda_parser::parse;
use spanda_readiness::{evaluate_human_teaming, format_human_teaming};
use std::fs;
use std::process;

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

/// `spanda team verify <file.sd> [--json]`
pub fn team_dispatch(args: &[String]) {
    if args.first().map(String::as_str) != Some("verify") {
        eprintln!("Usage: spanda team verify <file.sd> [--json]");
        process::exit(1);
    }
    let mut json = false;
    let mut file: Option<String> = None;
    for arg in args.iter().skip(1) {
        if arg == "--json" {
            json = true;
        } else if !arg.starts_with('-') && file.is_none() {
            file = Some(arg.clone());
        }
    }
    let file = file.unwrap_or_else(|| {
        eprintln!("Missing file path");
        process::exit(1);
    });
    let source = fs::read_to_string(&file).unwrap_or_else(|e| {
        eprintln!("Failed to read {}: {e}", file);
        process::exit(1);
    });
    let program = parse_program(&source);
    let report = evaluate_human_teaming(&program, &file);
    println!("{}", format_human_teaming(&report, json));
    if !report.passed {
        process::exit(1);
    }
}
