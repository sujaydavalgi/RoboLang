//! CLI for dependency graph visualization.
//!
use crate::config_load::load_system_config;
use spanda_graph::{build_dependency_graph, format_dependency_graph, GraphFormat};
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

fn parse_format(args: &[String]) -> GraphFormat {
    if args.iter().any(|a| a == "--json") {
        return GraphFormat::Json;
    }
    for (i, arg) in args.iter().enumerate() {
        if arg == "--format" {
            if let Some(value) = args.get(i + 1) {
                return GraphFormat::parse(value);
            }
        }
    }
    GraphFormat::Text
}

fn file_arg(args: &[String]) -> String {
    args.iter()
        .find(|a| !a.starts_with('-'))
        .cloned()
        .unwrap_or_else(|| {
            eprintln!("Usage: spanda graph <file.sd> [--format json|mermaid|dot|text] [--json] [--config <spanda.toml>]");
            process::exit(1);
        })
}

/// `spanda graph <file.sd> [--format json|mermaid|dot|text] [--json] [--config <spanda.toml>]`
pub fn graph_dispatch(args: &[String]) {
    let file = file_arg(args);
    let path = Path::new(&file);
    let program = parse_program(path);
    let config_flag = spanda_config::config_flag_from_args(args);
    let system_config = load_system_config(path, config_flag.as_deref().map(Path::new));
    let graph = build_dependency_graph(
        &program,
        &file,
        system_config.as_deref(),
    );
    let format = parse_format(args);
    println!("{}", format_dependency_graph(&graph, format));
}
