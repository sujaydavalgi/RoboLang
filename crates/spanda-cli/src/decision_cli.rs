//! CLI commands for distributed decision architecture and audit trails.

use spanda_decision::{
    audit_decisions_from_trace, evaluate_distributed_decisions, extract_decision_authorities,
    extract_decision_trees, extract_offline_policies, format_decision_audit,
    format_decision_explanations, format_distributed_report, format_simulation_report,
    load_persisted_policy_cache, save_persisted_policy_cache, security_audit, sign_offline_policy,
    simulate_distributed_decisions, threat_model_summary, AttackScenario, DecisionContext,
    DecisionLayer, OfflinePolicySpec, PersistedPolicyCache, SimulationOptions,
};
use spanda_lexer::tokenize;
use spanda_parser::parse;
use std::collections::HashMap;
use std::fs;
use std::process;

fn file_arg(args: &[String]) -> String {
    args.iter()
        .find(|a| !a.starts_with('-'))
        .cloned()
        .unwrap_or_else(|| {
            eprintln!("Missing file path");
            process::exit(1);
        })
}

fn read_and_parse(path: &str) -> spanda_ast::nodes::Program {
    let source = fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("Failed to read {path}: {e}");
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

fn flag_value(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1).cloned())
        .or_else(|| {
            args.iter()
                .find_map(|a| a.strip_prefix(&format!("{flag}=")).map(|v| v.to_string()))
        })
}

fn json_output(args: &[String]) -> bool {
    args.iter().any(|a| a == "--json")
}

/// `spanda decision list <file.sd> [--json]`
pub fn cmd_decision_list(args: &[String]) {
    let path = file_arg(args);
    let program = read_and_parse(&path);
    let authorities = extract_decision_authorities(&program);
    let trees = extract_decision_trees(&program);
    let offline = extract_offline_policies(&program);
    if json_output(args) {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "authorities": authorities,
                "decision_trees": trees,
                "offline_policies": offline,
            }))
            .unwrap_or_default()
        );
    } else {
        println!("Decision architecture in {path}");
        println!("  Authorities: {}", authorities.len());
        for a in &authorities {
            println!(
                "    {} — local: [{}], central: [{}]",
                a.entity_id,
                a.local_actions.join(", "),
                a.requires_central_approval.join(", ")
            );
        }
        println!("  Decision trees: {}", trees.len());
        for t in &trees {
            println!(
                "    {} ({:?}, {} branches)",
                t.name,
                t.layer,
                t.branches.len()
            );
        }
        println!("  Offline policies: {}", offline.len());
        for o in &offline {
            println!("    {} (max {} min)", o.name, o.max_duration_minutes);
        }
    }
}

/// `spanda decision inspect <file.sd> [--entity <id>] [--json]`
pub fn cmd_decision_inspect(args: &[String]) {
    let path = file_arg(args);
    let program = read_and_parse(&path);
    let entity_id = flag_value(args, "--entity").unwrap_or_else(|| "Rover".into());
    let signals: HashMap<String, bool> = flag_value(args, "--signal")
        .map(|s| {
            let mut m = HashMap::new();
            for part in s.split(',') {
                let (k, v) = part.split_once('=').unwrap_or((part, "true"));
                m.insert(k.trim().into(), v.trim() != "false");
            }
            m
        })
        .unwrap_or_default();
    let ctx = DecisionContext {
        entity_id,
        mission: flag_value(args, "--mission"),
        layer: DecisionLayer::LocalEntity,
        action: flag_value(args, "--action").unwrap_or_else(|| "continue_mission".into()),
        signals,
        offline_minutes: flag_value(args, "--offline-minutes")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0),
        policy_version: flag_value(args, "--policy-version").unwrap_or_else(|| "1.0.0".into()),
    };
    let report = evaluate_distributed_decisions(&program, &ctx);
    println!("{}", format_distributed_report(&report, json_output(args)));
    if !report.passed {
        process::exit(1);
    }
}

/// `spanda decision simulate <file.sd> [--offline] [--network-partition] [--fleet-coordinator-failure] [--json]`
pub fn cmd_decision_simulate(args: &[String]) {
    let path = file_arg(args);
    let program = read_and_parse(&path);
    let options = SimulationOptions {
        offline: args.iter().any(|a| a == "--offline"),
        network_partition: args.iter().any(|a| a == "--network-partition"),
        fleet_coordinator_failure: args.iter().any(|a| a == "--fleet-coordinator-failure"),
        entity_id: flag_value(args, "--entity").unwrap_or_else(|| "Rover".into()),
        mission: flag_value(args, "--mission"),
        signals: HashMap::new(),
    };
    let sim = simulate_distributed_decisions(&program, options);
    println!("{}", format_simulation_report(&sim, json_output(args)));
}

/// `spanda decision trace <mission.trace> [--json]`
pub fn cmd_decision_trace(args: &[String]) {
    let file = file_arg(args);
    match audit_decisions_from_trace(&file) {
        Ok(report) => println!("{}", format_decision_audit(&report, json_output(args))),
        Err(error) => {
            eprintln!("{error}");
            process::exit(1);
        }
    }
}

/// `spanda decision explain <mission.trace>`
pub fn cmd_decision_explain(args: &[String]) {
    let file = file_arg(args);
    match audit_decisions_from_trace(&file) {
        Ok(report) => println!("{}", format_decision_explanations(&report)),
        Err(error) => {
            eprintln!("{error}");
            process::exit(1);
        }
    }
}

/// `spanda decision policy <file.sd> [--json]`
pub fn cmd_decision_policy(args: &[String]) {
    let path = file_arg(args);
    let program = read_and_parse(&path);
    let offline = extract_offline_policies(&program);
    let trees = extract_decision_trees(&program);
    if json_output(args) {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "offline_policies": offline,
                "decision_trees": trees,
            }))
            .unwrap_or_default()
        );
    } else {
        println!("Decision policies in {path}");
        for o in &offline {
            println!(
                "\n  offline_policy {} (max {} min)",
                o.name, o.max_duration_minutes
            );
            println!("    allowed: [{}]", o.allowed_actions.join(", "));
            println!("    forbidden: [{}]", o.forbidden_actions.join(", "));
        }
        for t in &trees {
            println!("\n  decision_tree {} v{}", t.name, t.version);
            for b in &t.branches {
                println!("    when {} → [{}]", b.condition, b.actions.join(", "));
            }
        }
    }
}

/// `spanda decision sign-policy <file.sd> [--policy <name>] [--key <material>] [--write-cache] [--json]`
pub fn cmd_decision_sign_policy(args: &[String]) {
    let path = file_arg(args);
    let program = read_and_parse(&path);
    let policies = extract_offline_policies(&program);
    if policies.is_empty() {
        eprintln!("No offline_policy declarations in {path}");
        process::exit(1);
    }
    let filter = flag_value(args, "--policy");
    let signing_key = flag_value(args, "--key")
        .or_else(|| std::env::var("SPANDA_DECISION_POLICY_SIGNING_KEY").ok())
        .unwrap_or_else(|| {
            eprintln!(
                "Missing signing key: pass --key <material> or set SPANDA_DECISION_POLICY_SIGNING_KEY"
            );
            process::exit(1);
        });
    let write_cache = args.iter().any(|a| a == "--write-cache");
    let mut cache = if write_cache {
        load_persisted_policy_cache(None)
    } else {
        PersistedPolicyCache::new()
    };
    let targets: Vec<OfflinePolicySpec> = policies
        .into_iter()
        .filter(|p| filter.as_ref().map(|n| n == &p.name).unwrap_or(true))
        .collect();
    if targets.is_empty() {
        eprintln!("No offline policy matching --policy filter");
        process::exit(1);
    }
    let mut signed = Vec::new();
    for spec in targets {
        let signature = sign_offline_policy(&spec, &signing_key);
        let mut signed_spec = spec.clone();
        signed_spec.signature = Some(signature.clone());
        if write_cache {
            cache.upsert_offline_policy(signed_spec.clone());
        }
        signed.push(serde_json::json!({
            "name": signed_spec.name,
            "policy_version": signed_spec.policy_version,
            "signature": signature,
            "snippet": format!(
                "    signature = \"{signature}\";",
            ),
        }));
        if !json_output(args) {
            println!("offline_policy {}:", signed_spec.name);
            println!("  policy_version = \"{}\";", signed_spec.policy_version);
            println!("  signature = \"{signature}\";");
        }
    }
    if write_cache {
        if let Err(err) = save_persisted_policy_cache(&mut cache, None) {
            eprintln!("{err}");
            process::exit(1);
        }
        if !json_output(args) {
            println!("\nWrote signed policies to {}", cache_path_display());
        }
    }
    if json_output(args) {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "policies": signed,
                "cache_path": if write_cache { Some(cache_path_display()) } else { None },
            }))
            .unwrap_or_default()
        );
    }
}

fn cache_path_display() -> String {
    spanda_decision::default_policy_cache_path()
        .to_string_lossy()
        .into_owned()
}

fn cache_path_from_flag(args: &[String]) -> Option<std::path::PathBuf> {
    flag_value(args, "--cache").map(std::path::PathBuf::from)
}

/// `spanda decision cache show [--cache <path>] [--json]`
pub fn cmd_decision_cache_show(args: &[String]) {
    let cache = load_persisted_policy_cache(cache_path_from_flag(args).as_deref());
    if json_output(args) {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "cache_path": cache_path_from_flag(args)
                    .map(|p| p.to_string_lossy().into_owned())
                    .unwrap_or_else(cache_path_display),
                "policy_count": cache.policies.len(),
                "updated_at_ms": cache.updated_at_ms,
                "policies": cache.policies,
            }))
            .unwrap_or_default()
        );
        return;
    }
    println!(
        "Decision policy cache ({}) — {} policies",
        cache_path_from_flag(args)
            .map(|p| p.display().to_string())
            .unwrap_or_else(cache_path_display),
        cache.policies.len()
    );
    for (name, policy) in &cache.policies {
        let signed = policy
            .signature
            .as_ref()
            .map(|s| if s.is_empty() { "unsigned" } else { "signed" })
            .unwrap_or("unsigned");
        println!(
            "  {name} — v{} max {}m [{signed}]",
            policy.policy_version, policy.max_duration_minutes
        );
    }
}

/// `spanda decision cache sync <file.sd> [--sign] [--key <material>] [--cache <path>] [--json]`
pub fn cmd_decision_cache_sync(args: &[String]) {
    let path = file_arg(args);
    let program = read_and_parse(&path);
    let policies = extract_offline_policies(&program);
    if policies.is_empty() {
        eprintln!("No offline_policy declarations in {path}");
        process::exit(1);
    }
    let cache_path = cache_path_from_flag(args);
    let mut cache = load_persisted_policy_cache(cache_path.as_deref());
    let should_sign = args.iter().any(|a| a == "--sign");
    let signing_key = if should_sign {
        Some(
            flag_value(args, "--key")
                .or_else(|| std::env::var("SPANDA_DECISION_POLICY_SIGNING_KEY").ok())
                .unwrap_or_else(|| {
                    eprintln!(
                        "Missing signing key for --sign: pass --key or set SPANDA_DECISION_POLICY_SIGNING_KEY"
                    );
                    process::exit(1);
                }),
        )
    } else {
        None
    };
    let mut synced = Vec::new();
    for mut spec in policies {
        if let Some(key) = &signing_key {
            spec.signature = Some(sign_offline_policy(&spec, key));
        }
        cache.upsert_offline_policy(spec.clone());
        synced.push(spec);
    }
    if let Err(err) = save_persisted_policy_cache(&mut cache, cache_path.as_deref()) {
        eprintln!("{err}");
        process::exit(1);
    }
    if json_output(args) {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "synced": synced,
                "cache_path": cache_path
                    .map(|p| p.to_string_lossy().into_owned())
                    .unwrap_or_else(cache_path_display),
            }))
            .unwrap_or_default()
        );
    } else {
        println!(
            "Synced {} offline policies to {}",
            synced.len(),
            cache_path
                .map(|p| p.display().to_string())
                .unwrap_or_else(cache_path_display)
        );
    }
}

/// `spanda decision cache clear [--policy <name>] [--cache <path>]`
pub fn cmd_decision_cache_clear(args: &[String]) {
    let cache_path = cache_path_from_flag(args).unwrap_or_else(default_policy_cache_path);
    if let Some(name) = flag_value(args, "--policy") {
        let mut cache = load_persisted_policy_cache(Some(&cache_path));
        if cache.policies.remove(&name).is_none() {
            eprintln!("Policy '{name}' not in cache");
            process::exit(1);
        }
        if let Err(err) = save_persisted_policy_cache(&mut cache, Some(&cache_path)) {
            eprintln!("{err}");
            process::exit(1);
        }
        println!("Removed '{name}' from {}", cache_path.display());
        return;
    }
    if cache_path.exists() {
        std::fs::remove_file(&cache_path).unwrap_or_else(|e| {
            eprintln!("Failed to remove {}: {e}", cache_path.display());
            process::exit(1);
        });
    }
    println!("Cleared decision policy cache at {}", cache_path.display());
}

fn default_policy_cache_path() -> std::path::PathBuf {
    spanda_decision::default_policy_cache_path()
}

/// Dispatch `spanda decision cache` subcommands.
pub fn decision_cache_dispatch(args: &[String]) {
    let sub = args.first().map(String::as_str).unwrap_or("");
    match sub {
        "show" => cmd_decision_cache_show(&args[1..]),
        "sync" => cmd_decision_cache_sync(&args[1..]),
        "clear" => cmd_decision_cache_clear(&args[1..]),
        _ => {
            eprintln!(
                "Usage:\n  \
                 spanda decision cache show [--cache <path>] [--json]\n  \
                 spanda decision cache sync <file.sd> [--sign] [--key <material>] [--cache <path>]\n  \
                 spanda decision cache clear [--policy <name>] [--cache <path>]"
            );
            process::exit(1);
        }
    }
}

/// `spanda decision security-audit [--json]`
pub fn cmd_decision_security_audit(args: &[String]) {
    let findings = security_audit();
    if json_output(args) {
        println!(
            "{}",
            serde_json::to_string_pretty(&findings).unwrap_or_default()
        );
    } else {
        for f in &findings {
            println!(
                "- {:?}: severity={}, detected={}",
                f.scenario, f.severity, f.detected
            );
            println!("  mitigation: {}", f.mitigation);
        }
    }
}

/// `spanda decision threat-model`
pub fn cmd_decision_threat_model(_args: &[String]) {
    println!("{}", threat_model_summary());
}

/// `spanda decision simulate-attack <scenario> [--json]`
pub fn cmd_decision_simulate_attack(args: &[String]) {
    let scenario_str = file_arg(args);
    let scenario = match scenario_str.as_str() {
        "policy_tampering" => AttackScenario::PolicyTampering,
        "fake_coordinator" => AttackScenario::FakeCoordinator,
        "replayed_decision" => AttackScenario::ReplayedDecision,
        "compromised_robot" => AttackScenario::CompromisedRobot,
        "poisoned_telemetry" => AttackScenario::PoisonedTelemetry,
        "offline_abuse" => AttackScenario::OfflineAbuse,
        "split_brain_coordinator" => AttackScenario::SplitBrainCoordinator,
        other => {
            eprintln!("Unknown attack scenario: {other}");
            process::exit(1);
        }
    };
    let finding = spanda_decision::simulate_attack(scenario);
    if json_output(args) {
        println!(
            "{}",
            serde_json::to_string_pretty(&finding).unwrap_or_default()
        );
    } else {
        println!("Attack simulation: {:?}", finding.scenario);
        println!("  detected: {}", finding.detected);
        println!("  severity: {}", finding.severity);
        println!("  mitigation: {}", finding.mitigation);
    }
}

/// `spanda audit decisions <mission.trace> [--json] [--explain]`
pub fn cmd_audit_decisions(args: &[String]) {
    if args.iter().any(|a| a == "--explain") {
        cmd_decision_explain(args);
    } else {
        cmd_decision_trace(args);
    }
}

/// Dispatch `spanda decision` subcommands.
pub fn decision_dispatch(args: &[String]) {
    let sub = args.first().map(String::as_str).unwrap_or("");
    match sub {
        "list" => cmd_decision_list(&args[1..]),
        "inspect" => cmd_decision_inspect(&args[1..]),
        "simulate" => cmd_decision_simulate(&args[1..]),
        "trace" => cmd_decision_trace(&args[1..]),
        "explain" => cmd_decision_explain(&args[1..]),
        "policy" => cmd_decision_policy(&args[1..]),
        "sign-policy" => cmd_decision_sign_policy(&args[1..]),
        "cache" => decision_cache_dispatch(&args[1..]),
        "security-audit" => cmd_decision_security_audit(&args[1..]),
        "threat-model" => cmd_decision_threat_model(&args[1..]),
        "simulate-attack" => cmd_decision_simulate_attack(&args[1..]),
        _ => {
            eprintln!(
                "Usage:\n  \
                 spanda decision list <file.sd> [--json]\n  \
                 spanda decision inspect <file.sd> [--entity <id>] [--json]\n  \
                 spanda decision simulate <file.sd> [--offline] [--network-partition] [--fleet-coordinator-failure]\n  \
                 spanda decision trace <mission.trace> [--json]\n  \
                 spanda decision explain <mission.trace>\n  \
                 spanda decision policy <file.sd> [--json]\n  \
                 spanda decision sign-policy <file.sd> [--policy <name>] [--key <material>] [--write-cache] [--json]\n  \
                 spanda decision cache show|sync|clear ...\n  \
                 spanda decision security-audit [--json]\n  \
                 spanda decision threat-model\n  \
                 spanda decision simulate-attack <scenario>"
            );
            process::exit(1);
        }
    }
}

/// Dispatch `spanda audit decisions` (used from main.rs).
pub fn audit_dispatch(args: &[String]) {
    let sub = args.first().map(String::as_str).unwrap_or("");
    match sub {
        "decisions" => cmd_audit_decisions(&args[1..]),
        _ => {
            eprintln!("Usage:\n  spanda audit decisions <mission.trace> [--json] [--explain]");
            process::exit(1);
        }
    }
}
