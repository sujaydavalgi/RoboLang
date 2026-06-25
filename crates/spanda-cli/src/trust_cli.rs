//! CLI for composite program trust and registry package trust.
//!
use spanda_lexer::tokenize;
use spanda_parser::parse;
use spanda_trust::{
    evaluate_composite_trust, format_composite_trust, CompositeTrustFormat, CompositeTrustOptions,
};
use std::fs;
use std::path::{Path, PathBuf};
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

fn parse_format(args: &[String]) -> CompositeTrustFormat {
    if args.iter().any(|arg| arg == "--json") {
        return CompositeTrustFormat::Json;
    }
    for (index, arg) in args.iter().enumerate() {
        if arg == "--format" {
            if let Some(value) = args.get(index + 1) {
                return match value.as_str() {
                    "markdown" | "md" => CompositeTrustFormat::Markdown,
                    "json" => CompositeTrustFormat::Json,
                    _ => CompositeTrustFormat::Text,
                };
            }
        }
    }
    CompositeTrustFormat::Text
}

fn cmd_program_trust(args: &[String]) {
    // Evaluate composite trust for a mission program file.
    //
    // Parameters:
    // - `args` — CLI arguments after `trust`
    //
    // Returns:
    // None (prints report or exits on error).
    //
    // Options:
    // `--project`, `--json`, `--format`.
    //
    // Example:
    // cmd_program_trust(&["rover.sd".into(), "--json".into()]);

    let mut program_path: Option<PathBuf> = None;
    let mut project: Option<PathBuf> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--project" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("--project requires a directory path");
                    process::exit(1);
                }
                project = Some(PathBuf::from(&args[i]));
            }
            "--json" | "--format" => {
                if args[i] == "--format" {
                    i += 1;
                }
            }
            other if !other.starts_with('-') && program_path.is_none() => {
                program_path = Some(PathBuf::from(other));
            }
            other => {
                eprintln!("Unknown argument: {other}");
                process::exit(1);
            }
        }
        i += 1;
    }
    let path = program_path.unwrap_or_else(|| {
        eprintln!("Usage: spanda trust <file.sd> [--project <dir>] [--json] [--format text|json|markdown]");
        process::exit(1);
    });
    let format = parse_format(args);
    let (program, source) = parse_program(&path);
    let label = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("program.sd");
    let root = project.or_else(|| {
        path.parent()
            .filter(|parent| parent.as_os_str() != std::ffi::OsStr::new(""))
            .map(Path::to_path_buf)
    });
    let options = CompositeTrustOptions {
        project_root: root,
    };
    let report = evaluate_composite_trust(&program, &source, label, &options);
    println!("{}", format_composite_trust(&report, format));
    if !report.passed {
        process::exit(1);
    }
}

/// `spanda trust <package>` or `spanda trust <file.sd>`
pub fn cmd_trust(args: &[String]) {
    // Dispatch to package trust or composite program trust based on target shape.
    //
    // Parameters:
    // - `args` — CLI arguments after `trust`
    //
    // Returns:
    // None (prints report or exits on error).
    //
    // Options:
    // Package: `--version`, `--project`, `--json`.
    // Program: `--project`, `--json`, `--format`.
    //
    // Example:
    // cmd_trust(&["rover.sd".into()]);

    let target = args
        .iter()
        .find(|arg| !arg.starts_with('-') && arg.as_str() != "json" && arg.as_str() != "markdown");
    if let Some(path) = target {
        if path.ends_with(".sd") {
            cmd_program_trust(args);
            return;
        }
    }
    crate::package::cmd_trust(args);
}
