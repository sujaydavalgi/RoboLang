//! Verify-time tamper and integrity analysis for Spanda programs.
//!
pub mod detect;
pub mod diagnosis;
pub mod fleet;
pub mod integrity;
pub mod runtime;

pub use detect::{
    format_tamper_report, generate_tamper_check, TamperFinding, TamperFormat, TamperReport,
    TamperSeverity, TamperStatus,
};
pub use diagnosis::{
    diagnose_tamper_trace, format_tamper_diagnosis, TamperDiagnosisFormat, TamperDiagnosisReport,
    TamperTimelineEvent,
};
pub use fleet::{
    correlate_fleet_tamper, format_fleet_tamper_report, load_fleet_tamper_manifest,
    FleetTamperCorrelation, FleetTamperManifest, FleetTamperMember, FleetTamperReport,
    MemberTamperDiagnosis,
};
pub use integrity::{
    apply_agent_integrity, compare_agent_integrity, format_integrity_report,
    generate_integrity_report, AgentIntegrityActual, AgentIntegrityExpected,
    ArtifactIntegrityStatus, IntegrityArtifact, IntegrityFormat, IntegrityReport,
};
pub use runtime::{generate_runtime_tamper_check, MissionTrace, TraceFrame};
