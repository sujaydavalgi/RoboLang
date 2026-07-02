//! Tests for mission time travel inspection.

use spanda_runtime::{
    inspect_mission_at, parse_time_travel_at, MissionTrace, TimeTravelInspect, TraceFrame,
};

#[test]
fn inspects_decisions_at_offset() {
    let mut trace = MissionTrace::new("rover.sd");
    trace.record(0.0, "tick", serde_json::json!({"health": "ok"}));
    trace.record(
        1000.0,
        "decision_trace",
        serde_json::json!({"version": 3, "decision": "pause_mission"}),
    );
    let at_ms = parse_time_travel_at("T+00:01", &trace).expect("parse");
    let explorer = inspect_mission_at(&trace, at_ms, &[TimeTravelInspect::Decisions]);
    assert_eq!(explorer.inspected_frames, 2);
    assert_eq!(explorer.state.decisions.len(), 1);
}
