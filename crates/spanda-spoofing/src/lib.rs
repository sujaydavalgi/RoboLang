//! GPS and sensor spoofing detection for Spanda programs and mission traces.
//!
pub mod confidence;
pub mod coverage;
pub mod detect;
pub mod ml;
pub mod trace;
pub mod testing;

pub use confidence::{
    apply_spoofing_confidence_filter, requires_operator_confirmation, spoofing_min_confidence,
};
pub use detect::{
    analyze_path, format_spoofing_report, generate_program_spoof_check,
    generate_trace_spoof_check, SpoofingFormat, SpoofingReport, SpoofingSourceKind,
};
pub use trace::{analyze_trace_spoofing, MissionTrace, SpoofingAlert, SpoofingSeverity, TraceFrame};
