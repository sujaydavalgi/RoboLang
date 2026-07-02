//! Mission time travel — inspect historical mission state at a trace timestamp.

use crate::error::RuntimeError;
use crate::replay::{parse_replay_offset, MissionTrace, ReplayStateSnapshot, TraceFrame};
use serde::{Deserialize, Serialize};

/// Inspection facet for time-travel queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeTravelInspect {
    Decisions,
    Health,
    Readiness,
    Safety,
    All,
}

/// Snapshot of mission state at a point in trace time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoricalMissionState {
    pub at_ms: f64,
    pub frame_count: usize,
    pub latest_event: Option<String>,
    pub state: Option<ReplayStateSnapshot>,
    pub decisions: Vec<serde_json::Value>,
    pub health_events: Vec<String>,
    pub readiness_events: Vec<String>,
    pub safety_events: Vec<String>,
}

/// Timeline summary for a trace up to a timestamp.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimelineExplorer {
    pub source: String,
    pub at_ms: f64,
    pub total_frames: usize,
    pub inspected_frames: usize,
    pub state: HistoricalMissionState,
}

/// Parse `--at` values: milliseconds, `T+mm:ss`, `HH:MM:SS`, or ISO-8601 matched to frame clock.
pub fn parse_time_travel_at(raw: &str, trace: &MissionTrace) -> Result<f64, RuntimeError> {
    if let Ok(ms) = parse_replay_offset(raw) {
        return Ok(ms);
    }
    if let Some(ms) = parse_clock_time_ms(raw) {
        return Ok(ms);
    }
    if let Some(ms) = parse_iso_against_trace(raw, trace) {
        return Ok(ms);
    }
    Err(RuntimeError::new(
        format!(
            "Invalid --at value '{raw}'; expected T+mm:ss, HH:MM:SS, milliseconds, or ISO-8601"
        ),
        0,
    ))
}

/// Inspect mission trace state at a timestamp for selected facets.
pub fn inspect_mission_at(
    trace: &MissionTrace,
    at_ms: f64,
    inspect: &[TimeTravelInspect],
) -> TimelineExplorer {
    let inspect_all = inspect.is_empty() || inspect.contains(&TimeTravelInspect::All);
    let frames = frames_until(trace, at_ms);
    let frame_count = frames.len();
    let latest = frames.last().copied();
    let mut decisions = Vec::new();
    let mut health_events = Vec::new();
    let mut readiness_events = Vec::new();
    let mut safety_events = Vec::new();

    for frame in &frames {
        if inspect_all || inspect.contains(&TimeTravelInspect::Decisions) {
            collect_decisions(frame, &mut decisions);
        }
        if inspect_all || inspect.contains(&TimeTravelInspect::Health) {
            collect_labeled(frame, &["health", "fault", "degraded"], &mut health_events);
        }
        if inspect_all || inspect.contains(&TimeTravelInspect::Readiness) {
            collect_labeled(
                frame,
                &["readiness", "mission_ready"],
                &mut readiness_events,
            );
        }
        if inspect_all || inspect.contains(&TimeTravelInspect::Safety) {
            collect_labeled(
                frame,
                &["safety", "emergency", "kill_switch", "stop"],
                &mut safety_events,
            );
        }
    }

    let state = HistoricalMissionState {
        at_ms,
        frame_count,
        latest_event: latest.map(|frame| frame.event.clone()),
        state: latest.and_then(|frame| frame.state.clone()),
        decisions,
        health_events,
        readiness_events,
        safety_events,
    };
    TimelineExplorer {
        source: trace.source.clone(),
        at_ms,
        total_frames: trace.frames.len(),
        inspected_frames: frames.len(),
        state,
    }
}

fn frames_until(trace: &MissionTrace, at_ms: f64) -> Vec<&TraceFrame> {
    trace
        .frames
        .iter()
        .filter(|frame| frame.sim_time_ms <= at_ms)
        .collect()
}

fn collect_decisions(frame: &TraceFrame, out: &mut Vec<serde_json::Value>) {
    if frame.event.contains("decision") || frame.payload.get("version").is_some() {
        out.push(serde_json::json!({
            "sim_time_ms": frame.sim_time_ms,
            "event": frame.event,
            "payload": frame.payload,
        }));
        return;
    }
    if let Some(decision) = frame.payload.get("decision") {
        out.push(serde_json::json!({
            "sim_time_ms": frame.sim_time_ms,
            "event": frame.event,
            "decision": decision,
        }));
    }
}

fn collect_labeled(frame: &TraceFrame, labels: &[&str], out: &mut Vec<String>) {
    let haystack = format!("{} {}", frame.event, frame.payload).to_ascii_lowercase();
    if labels.iter().any(|label| haystack.contains(label)) {
        out.push(format!(
            "t={:.1}ms {} {:?}",
            frame.sim_time_ms, frame.event, frame.payload
        ));
    }
}

fn parse_clock_time_ms(raw: &str) -> Option<f64> {
    let parts: Vec<&str> = raw.split(':').collect();
    let total_secs = match parts.as_slice() {
        [hours, mins, secs] if raw.contains(':') && !raw.starts_with("T+") => {
            hours.parse::<f64>().ok()? * 3600.0
                + mins.parse::<f64>().ok()? * 60.0
                + secs.parse::<f64>().ok()?
        }
        [mins, secs] if raw.contains(':') && !raw.starts_with("T+") => {
            mins.parse::<f64>().ok()? * 60.0 + secs.parse::<f64>().ok()?
        }
        _ => return None,
    };
    Some(total_secs * 1000.0)
}

fn parse_iso_against_trace(raw: &str, trace: &MissionTrace) -> Option<f64> {
    let target = chrono::DateTime::parse_from_rfc3339(raw)
        .ok()?
        .with_timezone(&chrono::Utc);
    for frame in &trace.frames {
        if let Some(recorded) = frame.payload.get("recorded_at").and_then(|v| v.as_str()) {
            if let Ok(at) = chrono::DateTime::parse_from_rfc3339(recorded) {
                if at.with_timezone(&chrono::Utc) >= target {
                    return Some(frame.sim_time_ms);
                }
            }
        }
    }
    None
}

/// Format timeline explorer for CLI output.
pub fn format_timeline_explorer(explorer: &TimelineExplorer, json: bool) -> String {
    if json {
        return serde_json::to_string_pretty(explorer).unwrap_or_else(|e| e.to_string());
    }
    let state = &explorer.state;
    let mut lines = vec![
        format!("Mission time travel: {}", explorer.source),
        format!(
            "At {:.1}ms — {} / {} frames",
            explorer.at_ms, explorer.inspected_frames, explorer.total_frames
        ),
    ];
    if let Some(event) = &state.latest_event {
        lines.push(format!("Latest event: {event}"));
    }
    if !state.decisions.is_empty() {
        lines.push(format!("Decisions: {}", state.decisions.len()));
    }
    if !state.health_events.is_empty() {
        lines.push(format!("Health signals: {}", state.health_events.len()));
    }
    if !state.readiness_events.is_empty() {
        lines.push(format!(
            "Readiness signals: {}",
            state.readiness_events.len()
        ));
    }
    if !state.safety_events.is_empty() {
        lines.push(format!("Safety signals: {}", state.safety_events.len()));
    }
    lines.join("\n")
}

/// Parse comma-separated inspect facets from CLI.
pub fn parse_inspect_facets(raw: &str) -> Vec<TimeTravelInspect> {
    raw.split(',')
        .filter_map(|part| match part.trim().to_ascii_lowercase().as_str() {
            "decisions" | "decision" => Some(TimeTravelInspect::Decisions),
            "health" => Some(TimeTravelInspect::Health),
            "readiness" => Some(TimeTravelInspect::Readiness),
            "safety" => Some(TimeTravelInspect::Safety),
            "all" => Some(TimeTravelInspect::All),
            _ => None,
        })
        .collect()
}
