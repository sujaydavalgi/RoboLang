//! CLI for autonomous governance policy evaluation.

use spanda_lexer::tokenize;
use spanda_parser::parse;
use spanda_policy::{evaluate_policy, format_policy_report, list_policies};
use std::fs;
use std::process;

fn parse_program(_path: &str, source: &str) -> spanda_ast::nodes::Program {
    let tokens = tokenize(source).unwrap_or_else(|e| {
        eprintln!("{e}");
        process::exit(1);
    });
    parse(tokens).unwrap_or_else(|e| {
        eprintln!("{e}");
        process::exit(1);
    })
}

/// `spanda governance <file.sd> [--policy <name>] [--json]`
pub fn governance_dispatch(args: &[String]) {
    let mut file: Option<String> = None;
    let mut policy: Option<String> = None;
    let mut json = false;
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--json" => json = true,
            "--policy" => {
                index += 1;
                if index >= args.len() {
                    eprintln!("--policy requires a policy name");
                    process::exit(1);
                }
                policy = Some(args[index].clone());
            }
            other if !other.starts_with('-') && file.is_none() => file = Some(other.to_string()),
            other => {
                eprintln!("Unknown argument: {other}");
                process::exit(1);
            }
        }
        index += 1;
    }
    let file = file.unwrap_or_else(|| {
        eprintln!("Usage: spanda governance <file.sd> [--policy <name>] [--json]");
        process::exit(1);
    });
    let source = fs::read_to_string(&file).unwrap_or_else(|e| {
        eprintln!("Failed to read {}: {e}", file);
        process::exit(1);
    });
    let program = parse_program(&file, &source);
    let policies = list_policies(&program);
    if policies.is_empty() {
        eprintln!("No operational policy blocks declared in {file}");
        process::exit(1);
    }
    let selected = policy.unwrap_or_else(|| policies[0].clone());
    let report = evaluate_policy(&program, &selected, &file).unwrap_or_else(|e| {
        eprintln!("{e}");
        process::exit(1);
    });
    println!("{}", format_policy_report(&report, json));
    if !report.passed {
        process::exit(1);
    }
}
