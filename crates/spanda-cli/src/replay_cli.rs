//! Mission trace replay helpers shared by CLI commands.

use spanda_driver::{playback_mission, replay_mission, RunOptions};
use spanda_runtime::replay::{parse_replay_offset, MissionTrace};
use std::fs;
use std::path::Path;
use std::process;

/// Replay or inspect a mission trace with optional deterministic verification.
pub fn human_replay(
    trace_file: &str,
    from: Option<&str>,
    deterministic: bool,
    playback: bool,
    show_faults: bool,
    as_json: bool,
) {
    let trace = MissionTrace::load(trace_file).unwrap_or_else(|error| {
        eprintln!("{error}");
        process::exit(1);
    });

    if show_faults {
        if as_json {
            let faults = spanda_runtime_faults::faults_from_trace(&trace);
            println!(
                "{}",
                serde_json::to_string_pretty(&faults).unwrap_or_default()
            );
        } else {
            println!("{}", spanda_runtime_faults::format_trace_faults(&trace));
        }
        return;
    }
    let offset_ms = if let Some(raw) = from {
        parse_replay_offset(raw).unwrap_or_else(|error| {
            eprintln!("{error}");
            process::exit(1);
        })
    } else {
        0.0
    };
    let frames = trace.frames_from(offset_ms);

    if playback {
        let (report, state) = playback_mission(
            trace_file,
            RunOptions {
                replay_from_ms: Some(offset_ms),
                playback_wall_clock: true,
                ..Default::default()
            },
        )
        .unwrap_or_else(|error| {
            eprintln!("Playback failed: {error}");
            process::exit(1);
        });
        if as_json {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "ok": true,
                    "mode": "playback",
                    "frames_applied": report.frames_applied,
                    "states_applied": report.states_applied,
                    "offset_ms": offset_ms,
                    "state": state,
                }))
                .unwrap()
            );
            return;
        }
        println!(
            "Playback {}: {} frames ({} with state) from {:.0}ms",
            trace_file, report.frames_applied, report.states_applied, offset_ms
        );
        println!(
            "  Final pose: x={:.3} y={:.3} θ={:.3}",
            state.pose.x, state.pose.y, state.pose.theta
        );
        return;
    }

    if deterministic {
        let source_path = resolve_trace_source(trace_file, &trace.source);
        let source = fs::read_to_string(&source_path).unwrap_or_else(|error| {
            eprintln!("Failed to read trace source '{source_path}': {error}");
            process::exit(1);
        });
        let (_, verification) = replay_mission(
            &source,
            trace_file,
            RunOptions {
                max_loop_iterations: 20,
                record_trace: true,
                trace_source: Some(trace.source.clone()),
                replay_from_ms: Some(offset_ms),
                replay_deterministic: true,
                ..Default::default()
            },
        )
        .unwrap_or_else(|error| {
            eprintln!("Replay failed: {error}");
            process::exit(1);
        });
        if as_json {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "ok": verification.ok,
                    "source": trace.source,
                    "deterministic": true,
                    "offset_ms": offset_ms,
                    "matched": verification.matched,
                    "mismatches": verification.mismatches,
                }))
                .unwrap()
            );
        } else if verification.ok {
            println!(
                "✓ Deterministic replay verified for {} ({} frames from {:.0}ms)",
                trace_file, verification.matched, offset_ms
            );
        } else {
            eprintln!("✗ Deterministic replay mismatch for {trace_file}:");
            for mismatch in &verification.mismatches {
                eprintln!("  {mismatch}");
            }
            process::exit(1);
        }
        return;
    }

    if as_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "ok": true,
                "source": trace.source,
                "deterministic": trace.deterministic,
                "offset_ms": offset_ms,
                "frames": frames,
            }))
            .unwrap()
        );
        return;
    }
    println!(
        "Replay {} ({} frames from {:.0}ms)",
        trace_file,
        frames.len(),
        offset_ms
    );
    for frame in frames.iter().take(20) {
        println!(
            "  t={:.1}ms {} {:?}",
            frame.sim_time_ms, frame.event, frame.payload
        );
    }
    if frames.len() > 20 {
        println!("  ... {} more frames", frames.len() - 20);
    }
}

fn resolve_trace_source(trace_file: &str, source: &str) -> String {
    if Path::new(source).is_file() {
        return source.to_string();
    }
    if let Some(parent) = Path::new(trace_file).parent() {
        let candidate = parent.join(source);
        if candidate.is_file() {
            return candidate.to_string_lossy().into_owned();
        }
    }
    source.to_string()
}
