//! Certification proof and evidence pack CLI (`spanda certify prove|pack`).

use crate::certify_pack::{
    build_certification_pack, format_certification_pack, write_certification_bundle,
};
use spanda_certify::build_certification_proof;
use spanda_driver::compile;
use spanda_hardware::CompatSeverity;
use std::fs;
use std::io::{self, Write};
use std::process;

fn read_source(path: &str) -> String {
    // Description:
    //     Read source.
    //
    // Inputs:
    //     path: &str
    //         Caller-supplied path.
    //
    // Outputs:
    //     result: String
    //         Return value from `read_source`.
    //
    // Example:

    //     let result = spanda_cli::certify_cli::read_source(path);

    fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("Error reading {path}: {e}");
        process::exit(1);
    })
}

pub fn certify_dispatch(args: &[String]) {
    // Description:
    //     Certify dispatch.
    //
    // Inputs:
    //     args: &[String]
    //         Caller-supplied args.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::certify_cli::certify_dispatch(args);

    if args.first().map(String::as_str) == Some("prove") {
        cmd_prove(&args[1..]);
        return;
    }
    if args.first().map(String::as_str) == Some("pack") {
        cmd_pack(&args[1..]);
        return;
    }
    eprintln!(
        "Usage: spanda certify prove [--json] [--strict] [--out <file>] <file.sd>\n\
               spanda certify pack [--json] [--strict] [--bundle <dir|file.json>] <file.sd>"
    );
    process::exit(1);
}

fn cmd_pack(args: &[String]) {
    let mut json = false;
    let mut strict = false;
    let mut bundle: Option<String> = None;
    let mut file: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json = true,
            "--strict" => strict = true,
            "--bundle" if i + 1 < args.len() => {
                bundle = Some(args[i + 1].clone());
                i += 1;
            }
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
    let source = read_source(&file);
    let program = compile(&source)
        .unwrap_or_else(|e| {
            eprintln!("Error compiling {file}: {e}");
            process::exit(1);
        })
        .program;
    let pack = build_certification_pack(&program, &source, &file, strict);
    if let Some(ref path) = bundle {
        write_certification_bundle(&pack, std::path::Path::new(path)).unwrap_or_else(|e| {
            eprintln!("Failed to write bundle: {e}");
            process::exit(1);
        });
        if !json {
            println!("✓ Wrote certification bundle to {path}");
        }
    }
    if json {
        println!("{}", format_certification_pack(&pack, true));
    } else if bundle.is_none() {
        println!("{}", format_certification_pack(&pack, false));
    }
    if !pack.passed {
        process::exit(1);
    }
}

fn cmd_prove(args: &[String]) {
    // Description:
    //     Cmd prove.
    //
    // Inputs:
    //     args: &[String]
    //         Caller-supplied args.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::certify_cli::cmd_prove(args);

    let mut json = false;
    let mut strict = false;
    let mut out_path: Option<String> = None;
    let mut file: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json = true,
            "--strict" => strict = true,
            "--out" if i + 1 < args.len() => {
                out_path = Some(args[i + 1].clone());
                i += 1;
            }
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
    let source = read_source(&file);
    let program = compile(&source)
        .unwrap_or_else(|e| {
            eprintln!("Error compiling {file}: {e}");
            process::exit(1);
        })
        .program;
    let report = build_certification_proof(&program, &file, strict);
    let payload = serde_json::to_string_pretty(&report).unwrap_or_else(|e| {
        eprintln!("Error serializing proof: {e}");
        process::exit(1);
    });
    if let Some(path) = &out_path {
        fs::write(path, &payload).unwrap_or_else(|e| {
            eprintln!("Error writing {path}: {e}");
            process::exit(1);
        });
        if !json {
            println!("✓ Wrote certification proof to {path}");
        }
    }
    if json {
        println!("{payload}");
    } else if out_path.is_none() {
        println!("Certification proof for {file}");
        println!(
            "  Status: {}",
            if report.passed { "PASSED" } else { "FAILED" }
        );
        println!("  {}", report.summary);
        if let Some(hash) = &report.program_hash {
            println!("  program_hash: {hash}");
        }
        for item in &report.checklist {
            let icon = match item.severity {
                CompatSeverity::Pass => "✓",
                CompatSeverity::Warning => "⚠",
                CompatSeverity::Error => "✗",
            };
            println!("  {icon} [{}] {}", item.category, item.message);
        }
    }
    let _ = io::stdout().flush();
    if !report.passed {
        process::exit(1);
    }
}
