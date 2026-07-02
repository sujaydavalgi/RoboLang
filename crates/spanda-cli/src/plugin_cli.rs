//! Plugin CLI (`spanda plugin search|install|uninstall|inspect|trust|enable|disable`).

use spanda_package::manifest::find_project_root;
use spanda_plugin::registry::{lookup_plugin_entry, search_plugins, PluginTrustTier};
use spanda_plugin::runtime::{PluginManager, PluginState};
use std::env;
use std::path::PathBuf;
use std::process;

const HOST_VERSION: &str = env!("CARGO_PKG_VERSION");

fn project_root_or_cwd() -> PathBuf {
    env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn resolve_project_root() -> PathBuf {
    find_project_root(&project_root_or_cwd()).unwrap_or_else(|| project_root_or_cwd())
}

fn open_manager() -> PluginManager {
    PluginManager::open(&resolve_project_root(), HOST_VERSION).unwrap_or_else(|e| {
        eprintln!("Error opening plugin store: {e}");
        process::exit(1);
    })
}

pub fn plugin_dispatch(args: &[String]) {
    let Some(sub) = args.first().map(String::as_str) else {
        print_usage();
        process::exit(1);
    };
    match sub {
        "search" => cmd_search(&args[1..]),
        "install" => cmd_install(&args[1..]),
        "uninstall" => cmd_uninstall(&args[1..]),
        "inspect" => cmd_inspect(&args[1..]),
        "trust" => cmd_trust(&args[1..]),
        "enable" => cmd_enable(&args[1..]),
        "disable" => cmd_disable(&args[1..]),
        "list" => cmd_list(&args[1..]),
        other => {
            eprintln!("Unknown plugin subcommand: {other}");
            print_usage();
            process::exit(1);
        }
    }
}

fn print_usage() {
    eprintln!(
        "Usage:\n\
         spanda plugin search <query>\n\
         spanda plugin install <name|path> [--path <dir>] [--approve-dangerous]\n\
         spanda plugin uninstall <name>\n\
         spanda plugin inspect <name> [--json]\n\
         spanda plugin trust <name> <official|verified|community|experimental|deprecated|blocked>\n\
         spanda plugin enable <name>\n\
         spanda plugin disable <name>\n\
         spanda plugin list [--json]"
    );
}

fn cmd_search(args: &[String]) {
    let query = args.first().map(String::as_str).unwrap_or("");
    let results = search_plugins(query);
    if results.is_empty() {
        println!("No plugins matched '{query}'");
        return;
    }
    for entry in results {
        println!(
            "{} ({}) — {} [{}]",
            entry.name,
            entry.latest_version().unwrap_or("?"),
            entry.description.as_deref().unwrap_or(""),
            entry.trust_tier().as_str()
        );
    }
}

fn cmd_install(args: &[String]) {
    let mut path: Option<PathBuf> = None;
    let mut name: Option<String> = None;
    let mut approve_dangerous = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--path" if i + 1 < args.len() => {
                path = Some(PathBuf::from(&args[i + 1]));
                i += 1;
            }
            "--approve-dangerous" => approve_dangerous = true,
            other if !other.starts_with('-') && name.is_none() => name = Some(other.to_string()),
            other => {
                eprintln!("Unknown argument: {other}");
                process::exit(1);
            }
        }
        i += 1;
    }
    let source = if let Some(p) = path {
        p
    } else if let Some(name) = &name {
        if std::path::Path::new(name).is_dir() {
            PathBuf::from(name)
        } else if let Some(entry) = lookup_plugin_entry(name) {
            let examples = resolve_project_root().ancestors().find_map(|root| {
                let slug = name
                    .strip_prefix("spanda-plugin-")
                    .unwrap_or(name.as_str())
                    .replace('-', "_");
                let candidate = root.join("examples/plugins").join(slug);
                if candidate.is_dir() {
                    Some(candidate)
                } else {
                    None
                }
            });
            examples.unwrap_or_else(|| {
                eprintln!(
                    "Plugin '{name}' found in registry ({}). Provide --path for local install.",
                    entry.latest_version().unwrap_or("?")
                );
                process::exit(1);
            })
        } else {
            eprintln!("Plugin not found in registry: {name}. Use --path <dir>.");
            process::exit(1);
        }
    } else {
        eprintln!("Missing plugin name or --path <dir>");
        process::exit(1);
    };

    let mut manager = open_manager();
    let host_version = manager.host_version().to_string();
    match manager
        .store_mut()
        .install_from_dir(&source, &host_version, approve_dangerous)
    {
        Ok(record) => {
            println!(
                "✓ Installed plugin {}@{} ({})",
                record.name, record.version, record.plugin_type
            );
        }
        Err(e) => {
            eprintln!("Install failed: {e}");
            process::exit(1);
        }
    }
}

fn cmd_uninstall(args: &[String]) {
    let name = args.first().cloned().unwrap_or_else(|| {
        eprintln!("Missing plugin name");
        process::exit(1);
    });
    let mut manager = open_manager();
    manager.store_mut().uninstall(&name).unwrap_or_else(|e| {
        eprintln!("Uninstall failed: {e}");
        process::exit(1);
    });
    println!("✓ Uninstalled plugin {name}");
}

fn cmd_inspect(args: &[String]) {
    let mut json = false;
    let mut name: Option<String> = None;
    for arg in args {
        if arg == "--json" {
            json = true;
        } else if name.is_none() {
            name = Some(arg.clone());
        }
    }
    let name = name.unwrap_or_else(|| {
        eprintln!("Missing plugin name");
        process::exit(1);
    });
    let manager = open_manager();
    let report = manager.store().inspect(&name).unwrap_or_else(|e| {
        eprintln!("Inspect failed: {e}");
        process::exit(1);
    });
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report).unwrap_or_else(|e| {
                eprintln!("JSON error: {e}");
                process::exit(1);
            })
        );
    } else {
        println!(
            "Plugin: {}@{}",
            report.installed.name, report.installed.version
        );
        println!("  Type: {}", report.installed.plugin_type);
        println!("  State: {:?}", report.installed.state);
        println!("  Trust: {}", report.installed.trust_tier);
        println!(
            "  Capabilities: {}",
            report.manifest.capabilities.requires.join(", ")
        );
        println!(
            "  Sandbox: {} network={} filesystem={}",
            report.manifest.security.sandbox,
            report.manifest.security.network,
            report.manifest.security.filesystem
        );
    }
}

fn cmd_trust(args: &[String]) {
    let name = args.first().cloned().unwrap_or_else(|| {
        eprintln!("Missing plugin name");
        process::exit(1);
    });
    let tier_str = args.get(1).map(String::as_str).unwrap_or_else(|| {
        eprintln!("Missing trust tier");
        process::exit(1);
    });
    let tier = PluginTrustTier::parse_str(tier_str).unwrap_or_else(|| {
        eprintln!("Unknown trust tier: {tier_str}");
        process::exit(1);
    });
    let mut manager = open_manager();
    manager
        .store_mut()
        .set_trust(&name, tier)
        .unwrap_or_else(|e| {
            eprintln!("Trust update failed: {e}");
            process::exit(1);
        });
    println!("✓ Set trust tier for {name} to {tier}");
}

fn cmd_enable(args: &[String]) {
    let name = args.first().cloned().unwrap_or_else(|| {
        eprintln!("Missing plugin name");
        process::exit(1);
    });
    let mut manager = open_manager();
    manager.store_mut().enable(&name).unwrap_or_else(|e| {
        eprintln!("Enable failed: {e}");
        process::exit(1);
    });
    println!("✓ Enabled plugin {name}");
}

fn cmd_disable(args: &[String]) {
    let name = args.first().cloned().unwrap_or_else(|| {
        eprintln!("Missing plugin name");
        process::exit(1);
    });
    let mut manager = open_manager();
    manager.store_mut().disable(&name).unwrap_or_else(|e| {
        eprintln!("Disable failed: {e}");
        process::exit(1);
    });
    println!("✓ Disabled plugin {name}");
}

fn cmd_list(args: &[String]) {
    let json = args.iter().any(|a| a == "--json");
    let manager = open_manager();
    let plugins = manager.store().list();
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&plugins).unwrap_or_else(|e| {
                eprintln!("JSON error: {e}");
                process::exit(1);
            })
        );
    } else if plugins.is_empty() {
        println!("No plugins installed");
    } else {
        for plugin in plugins {
            let state = match plugin.state {
                PluginState::Installed => "installed",
                PluginState::Enabled => "enabled",
                PluginState::Disabled => "disabled",
            };
            println!(
                "{}@{} — {} [{state}] trust={}",
                plugin.name, plugin.version, plugin.plugin_type, plugin.trust_tier
            );
        }
    }
}
