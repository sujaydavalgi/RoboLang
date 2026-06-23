//! One-command showcase demos for evaluators (`spanda demo <name>`).
//!
use spanda_driver::{check, run, verify_compatibility_with_registry, RunOptions};
use spanda_hardware::VerifyOptions;
use std::env;
use std::path::{Path, PathBuf};
use std::process::{self, Command};

fn repo_root() -> PathBuf {
    // Resolve the Spanda repository root for bundled showcase examples.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Path to repo root, or current working directory if not found.
    //
    // Options:
    // Honors `SPANDA_ROOT` when set.
    //
    // Example:
    // let root = repo_root();

    if let Ok(root) = env::var("SPANDA_ROOT") {
        let path = PathBuf::from(root);
        if path.join("examples/showcase/README.md").is_file() {
            return path;
        }
    }

    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(cwd) = env::current_dir() {
        candidates.push(cwd);
    }
    if let Ok(exe) = env::current_exe() {
        if let Some(parent) = exe.parent() {
            candidates.push(parent.to_path_buf());
            if let Some(grand) = parent.parent() {
                candidates.push(grand.to_path_buf());
            }
        }
    }

    for start in candidates {
        let mut dir = start;
        for _ in 0..8 {
            if dir.join("examples/showcase/README.md").is_file() {
                return dir;
            }
            if !dir.pop() {
                break;
            }
        }
    }

    env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn showcase(root: &Path, parts: &[&str]) -> PathBuf {
    let mut path = root.join("examples/showcase");
    for part in parts {
        path.push(part);
    }
    path
}

fn require_file(path: &Path) -> &Path {
    if !path.is_file() {
        eprintln!(
            "Demo file not found: {}\n\
             Clone the Spanda repository or set SPANDA_ROOT to the repo root.",
            path.display()
        );
        process::exit(1);
    }
    path
}

fn read_source(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("Error reading {}: {e}", path.display());
        process::exit(1);
    })
}

fn spanda_bin() -> String {
    env::var("SPANDA_BIN").unwrap_or_else(|_| "spanda".into())
}

fn run_spanda(subcommand: &str, file: &Path, extra: &[&str]) {
    let spanda = spanda_bin();
    let mut cmd = Command::new(&spanda);
    cmd.arg(subcommand).arg(file);
    for flag in extra {
        cmd.arg(flag);
    }
    let status = cmd.status().unwrap_or_else(|e| {
        eprintln!("Failed to run {spanda}: {e}");
        process::exit(1);
    });
    if !status.success() {
        process::exit(status.code().unwrap_or(1));
    }
}

fn run_spanda_args(args: &[&str]) {
    let spanda = spanda_bin();
    let status = Command::new(&spanda)
        .args(args)
        .status()
        .unwrap_or_else(|e| {
            eprintln!("Failed to run {spanda}: {e}");
            process::exit(1);
        });
    if !status.success() {
        process::exit(status.code().unwrap_or(1));
    }
}

fn expect_check_fail(file: &Path) {
    let source = read_source(file);
    if check(&source).is_ok() {
        eprintln!(
            "Expected compile error for {} (unsafe path should be rejected)",
            file.display()
        );
        process::exit(1);
    }
    println!(
        "✓ {} — compile-time safety gate rejected unsafe code",
        file.display()
    );
}

fn demo_rover(root: &Path) {
    let rover_dir = root.join("examples/showcase/autonomous_rover");
    let rover_path = rover_dir.join("src/rover.sd");
    let rover_sd = require_file(&rover_path);
    let trace = rover_dir.join("src/rover.trace");

    println!("== Autonomous Rover — flagship platform demo ==\n");

    if rover_dir.join("spanda.toml").is_file() {
        println!("→ Installing packages…");
        let spanda = env::var("SPANDA_BIN").unwrap_or_else(|_| "spanda".into());
        let status = Command::new(&spanda)
            .arg("install")
            .current_dir(&rover_dir)
            .status()
            .unwrap_or_else(|e| {
                eprintln!("Failed to run {spanda} install: {e}");
                process::exit(1);
            });
        if !status.success() {
            process::exit(status.code().unwrap_or(1));
        }
    }

    println!("→ Verify hardware fit");
    run_spanda("verify", rover_sd, &["--json", "--target", "RoverV1"]);

    println!("→ Simulate patrol");
    run_spanda("sim", rover_sd, &[]);

    if trace.is_file() {
        println!("→ Replay recorded mission");
        run_spanda("replay", &trace, &[]);
    }

    println!("\nDemo complete. See examples/showcase/autonomous_rover/README.md");
}

fn demo_safety(root: &Path) {
    let unsafe_path = showcase(root, &["unsafe_ai", "unsafe.sd"]);
    let safe_path = showcase(root, &["unsafe_ai", "safe.sd"]);
    let unsafe_sd = require_file(&unsafe_path);
    let safe_sd = require_file(&safe_path);

    println!("== Safety — ActionProposal vs SafeAction ==\n");
    println!("→ Unsafe: ActionProposal sent directly to actuator (must fail)");
    expect_check_fail(unsafe_sd);

    println!("→ Safe: safety.validate() gate (must pass)");
    run_spanda("check", safe_sd, &[]);

    println!("\nDemo complete. See examples/showcase/unsafe_ai/README.md");
}

fn demo_verify(root: &Path) {
    let fail_path = showcase(root, &["hardware_verification", "mission_missing_lidar.sd"]);
    let pass_path = showcase(root, &["hardware_verification", "mission_with_lidar.sd"]);
    let fail_sd = require_file(&fail_path);
    let pass_sd = require_file(&pass_path);

    println!("== Hardware verification — obstacle avoidance needs Lidar ==\n");

    println!("→ Mission on robot without Lidar (must fail)");
    let source = read_source(fail_sd);
    let options = VerifyOptions::default();
    let report = verify_compatibility_with_registry(&source, &options, None).unwrap_or_else(|e| {
        eprintln!("Verify failed: {e}");
        process::exit(1);
    });
    if report.compatible {
        eprintln!("Expected verification failure for {}", fail_sd.display());
        process::exit(1);
    }
    println!("✓ Verification failed as expected — missing Lidar sensor");

    println!("→ Same mission on robot with Lidar (must pass)");
    run_spanda("verify", pass_sd, &["--json", "--target", "RoverV1"]);

    println!("\nDemo complete. See examples/showcase/hardware_verification/README.md");
}

fn demo_fleet(root: &Path) {
    let fleet_path = showcase(root, &["fleet_management", "fleet.sd"]);
    let fleet_sd = require_file(&fleet_path);

    println!("== Fleet — multi-robot coordination ==\n");
    run_spanda("check", fleet_sd, &[]);
    run_spanda_args(&["fleet", "run", fleet_sd.to_str().unwrap()]);

    println!("\nDemo complete. See examples/showcase/fleet_management/README.md");
}

fn demo_health(root: &Path) {
    let health_path = showcase(root, &["health_monitoring", "rover.sd"]);
    let health_sd = require_file(&health_path);

    println!("== Health monitoring — fault injection ==\n");
    run_spanda("check", health_sd, &[]);
    run_spanda_args(&["health", "robot", health_sd.to_str().unwrap(), "--json"]);

    println!("→ Simulate with injected health faults");
    let source = read_source(health_sd);
    let opts = RunOptions {
        inject_health_faults: true,
        ..Default::default()
    };
    run(&source, opts).unwrap_or_else(|e| {
        eprintln!("Simulation failed: {e}");
        process::exit(1);
    });
    println!("✓ Simulation completed with --inject-health-faults");

    println!("\nDemo complete. See examples/showcase/health_monitoring/README.md");
}

pub fn demo_dispatch(args: &[String]) {
    // Run a bundled showcase demo by name.
    //
    // Parameters:
    // - `args` — demo name plus optional flags (reserved)
    //
    // Returns:
    // Nothing; exits the process on failure.
    //
    // Options:
    // Set `SPANDA_ROOT` or run from a cloned repository.
    //
    // Example:
    // demo_dispatch(&["rover".into()]);

    let root = repo_root();
    let name = args.first().map(String::as_str).unwrap_or("");

    match name {
        "rover" => demo_rover(&root),
        "safety" => demo_safety(&root),
        "verify" => demo_verify(&root),
        "fleet" => demo_fleet(&root),
        "health" => demo_health(&root),
        "" | "list" | "--help" | "-h" => {
            eprintln!(
                "Spanda showcase demos\n\n\
                 Usage:\n\
                   spanda demo <name>\n\n\
                 Demos:\n\
                   rover   — GPS, packages, verify, sim, replay (flagship)\n\
                   safety  — ActionProposal blocked; SafeAction passes\n\
                   verify  — hardware fit: missing Lidar fails, added Lidar passes\n\
                   fleet   — multi-robot fleet simulation\n\
                   health  — health checks with fault injection\n\n\
                 Set SPANDA_ROOT to the repository root if examples are not found.\n\
                 See examples/showcase/README.md"
            );
            if name.is_empty() {
                process::exit(1);
            }
        }
        other => {
            eprintln!("Unknown demo '{other}'. Run: spanda demo --help");
            process::exit(1);
        }
    }
}
