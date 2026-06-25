//! Verify-time tamper and integrity analysis for Spanda programs.
//!
pub mod detect;
pub mod integrity;

pub use detect::{
    format_tamper_report, generate_tamper_check, TamperFinding, TamperFormat, TamperReport,
    TamperSeverity, TamperStatus,
};
pub use integrity::{
    apply_agent_integrity, compare_agent_integrity, format_integrity_report,
    generate_integrity_report, AgentIntegrityActual, AgentIntegrityExpected,
    ArtifactIntegrityStatus, IntegrityArtifact, IntegrityFormat, IntegrityReport,
};
