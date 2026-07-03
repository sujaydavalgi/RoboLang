//! CLI for Twin Cloud SaaS — push, pull, and list mission twin snapshots.

use spanda_twin_cloud::{build_snapshot_from_program, TwinCloudClient, TwinCloudConfig};
use std::fs;
use std::process;

fn parse_program(source: &str) -> spanda_ast::nodes::Program {
    let tokens = spanda_lexer::tokenize(source).unwrap_or_else(|error| {
        eprintln!("{error}");
        process::exit(1);
    });
    spanda_parser::parse(tokens).unwrap_or_else(|error| {
        eprintln!("{error}");
        process::exit(1);
    })
}

fn cloud_config(url: Option<&str>) -> TwinCloudConfig {
    if let Some(base) = url {
        return TwinCloudConfig {
            base_url: base.trim_end_matches('/').to_string(),
            api_key: std::env::var("SPANDA_TWIN_CLOUD_API_KEY")
                .ok()
                .or_else(|| std::env::var("SPANDA_API_KEY").ok()),
            tenant_id: std::env::var("SPANDA_TWIN_CLOUD_TENANT")
                .ok()
                .or_else(|| std::env::var("SPANDA_TENANT_ID").ok())
                .unwrap_or_else(|| "default".into()),
        };
    }
    TwinCloudConfig::from_env().unwrap_or_else(|| {
        eprintln!("Set SPANDA_TWIN_CLOUD_URL or pass --url <base>");
        process::exit(1);
    })
}

/// `spanda twin cloud push|pull|list`
pub fn twin_cloud_dispatch(args: &[String]) {
    match args.first().map(String::as_str) {
        Some("push") => cmd_twin_cloud_push(&args[1..]),
        Some("pull") => cmd_twin_cloud_pull(&args[1..]),
        Some("list") => cmd_twin_cloud_list(&args[1..]),
        Some("sync") => cmd_twin_cloud_sync(&args[1..]),
        Some("import-replay") => cmd_twin_cloud_import_replay(&args[1..]),
        _ => {
            eprintln!(
                "Usage: spanda twin cloud push <file.sd> [--url <base>] [--twin-id <id>] [--json]\n       \
                 spanda twin cloud pull <twin-id> [--url <base>] [--out <file>] [--json]\n       \
                 spanda twin cloud list [--url <base>] [--json]\n       \
                 spanda twin cloud sync [--url <base>] [--twin-id <id>] [--json]\n       \
                 spanda twin cloud import-replay <replay.json> [--program <file.sd>] [--json]"
            );
            process::exit(1);
        }
    }
}

fn cmd_twin_cloud_push(args: &[String]) {
    let mut file: Option<String> = None;
    let mut url: Option<String> = None;
    let mut twin_id: Option<String> = None;
    let mut json = false;
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--json" => json = true,
            "--url" => {
                index += 1;
                if index >= args.len() {
                    eprintln!("--url requires a value");
                    process::exit(1);
                }
                url = Some(args[index].clone());
            }
            "--twin-id" => {
                index += 1;
                if index >= args.len() {
                    eprintln!("--twin-id requires a value");
                    process::exit(1);
                }
                twin_id = Some(args[index].clone());
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
        eprintln!("Missing file path");
        process::exit(1);
    });
    let source = fs::read_to_string(&file).unwrap_or_else(|error| {
        eprintln!("Failed to read {file}: {error}");
        process::exit(1);
    });
    let program = parse_program(&source);
    let config = cloud_config(url.as_deref());
    let snapshot = build_snapshot_from_program(
        &program,
        &file,
        twin_id.as_deref(),
        config.tenant_id.as_str(),
    );
    let client = TwinCloudClient::new(config);
    let response = client.push_snapshot(&snapshot).unwrap_or_else(|error| {
        eprintln!("Twin cloud push failed: {error}");
        process::exit(1);
    });
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&response).unwrap_or_default()
        );
    } else {
        println!(
            "Twin cloud push OK: {} (readiness {}/{} mission_ready={})",
            response.twin_id,
            response.snapshot.mission_twin.readiness.score.total,
            response.snapshot.mission_twin.readiness.score.maximum,
            response.snapshot.mission_twin.risks.mission_ready
        );
    }
}

fn cmd_twin_cloud_pull(args: &[String]) {
    let mut twin_id: Option<String> = None;
    let mut url: Option<String> = None;
    let mut out_path: Option<String> = None;
    let mut json = false;
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--json" => json = true,
            "--url" | "--out" => {
                let flag = args[index].as_str();
                index += 1;
                if index >= args.len() {
                    eprintln!("{flag} requires a value");
                    process::exit(1);
                }
                if flag == "--url" {
                    url = Some(args[index].clone());
                } else {
                    out_path = Some(args[index].clone());
                }
            }
            other if !other.starts_with('-') && twin_id.is_none() => {
                twin_id = Some(other.to_string())
            }
            other => {
                eprintln!("Unknown argument: {other}");
                process::exit(1);
            }
        }
        index += 1;
    }
    let twin_id = twin_id.unwrap_or_else(|| {
        eprintln!("Missing twin id");
        process::exit(1);
    });
    let client = TwinCloudClient::new(cloud_config(url.as_deref()));
    let snapshot = client.latest_snapshot(&twin_id).unwrap_or_else(|error| {
        eprintln!("Twin cloud pull failed: {error}");
        process::exit(1);
    });
    let payload = serde_json::to_string_pretty(&snapshot).unwrap_or_default();
    if let Some(path) = out_path {
        fs::write(&path, payload).unwrap_or_else(|error| {
            eprintln!("Failed to write {path}: {error}");
            process::exit(1);
        });
        if !json {
            println!("Wrote twin snapshot to {path}");
        }
    } else {
        println!("{payload}");
    }
}

fn cmd_twin_cloud_list(args: &[String]) {
    let mut url: Option<String> = None;
    let mut json = false;
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--json" => json = true,
            "--url" => {
                index += 1;
                if index >= args.len() {
                    eprintln!("--url requires a value");
                    process::exit(1);
                }
                url = Some(args[index].clone());
            }
            other => {
                eprintln!("Unknown argument: {other}");
                process::exit(1);
            }
        }
        index += 1;
    }
    let client = TwinCloudClient::new(cloud_config(url.as_deref()));
    let response = client.list_twins().unwrap_or_else(|error| {
        eprintln!("Twin cloud list failed: {error}");
        process::exit(1);
    });
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&response).unwrap_or_default()
        );
        return;
    }
    if response.twins.is_empty() {
        println!("No twin snapshots registered");
        return;
    }
    for twin in response.twins {
        println!(
            "{} program={} readiness={} mission_ready={} history={}",
            twin.twin_id, twin.program, twin.readiness_score, twin.mission_ready, twin.history_count
        );
    }
}

fn cmd_twin_cloud_sync(args: &[String]) {
    let mut url: Option<String> = None;
    let mut twin_id: Option<String> = None;
    let mut json = false;
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--json" => json = true,
            "--url" | "--twin-id" => {
                let flag = args[index].as_str();
                index += 1;
                if index >= args.len() {
                    eprintln!("{flag} requires a value");
                    process::exit(1);
                }
                if flag == "--url" {
                    url = Some(args[index].clone());
                } else {
                    twin_id = Some(args[index].clone());
                }
            }
            other => {
                eprintln!("Unknown argument: {other}");
                process::exit(1);
            }
        }
        index += 1;
    }
    let client = TwinCloudClient::new(cloud_config(url.as_deref()));
    let response = client
        .sync_program_snapshot(twin_id.as_deref())
        .unwrap_or_else(|error| {
            eprintln!("Twin cloud sync failed: {error}");
            process::exit(1);
        });
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&response).unwrap_or_default()
        );
    } else {
        println!("Twin cloud sync OK: {}", response.twin_id);
    }
}

fn cmd_twin_cloud_import_replay(args: &[String]) {
    let mut replay_path: Option<String> = None;
    let mut program: Option<String> = None;
    let mut url: Option<String> = None;
    let mut twin_id: Option<String> = None;
    let mut json = false;
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--json" => json = true,
            "--url" | "--program" | "--twin-id" => {
                let flag = args[index].as_str();
                index += 1;
                if index >= args.len() {
                    eprintln!("{flag} requires a value");
                    process::exit(1);
                }
                match flag {
                    "--url" => url = Some(args[index].clone()),
                    "--program" => program = Some(args[index].clone()),
                    _ => twin_id = Some(args[index].clone()),
                }
            }
            other if !other.starts_with('-') && replay_path.is_none() => {
                replay_path = Some(other.to_string());
            }
            other => {
                eprintln!("Unknown argument: {other}");
                process::exit(1);
            }
        }
        index += 1;
    }
    let replay_path = replay_path.unwrap_or_else(|| {
        eprintln!("Missing replay JSON path");
        process::exit(1);
    });
    let program = program.unwrap_or(replay_path.clone());
    let client = TwinCloudClient::new(cloud_config(url.as_deref()));
    let response = client
        .import_replay(&program, twin_id.as_deref())
        .unwrap_or_else(|error| {
            eprintln!("Twin cloud import-replay failed: {error}");
            process::exit(1);
        });
    if json {
        println!("{}", serde_json::to_string_pretty(&response).unwrap_or_default());
    } else {
        println!(
            "Twin cloud import-replay OK: {}",
            response["twin_id"].as_str().unwrap_or("unknown")
        );
    }
}
