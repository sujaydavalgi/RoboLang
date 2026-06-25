//! Composite trust scoring for Spanda mission programs.
//!
pub mod composite;

pub use composite::{
    evaluate_composite_trust, format_composite_trust, CompositeTrustFormat, CompositeTrustOptions,
    CompositeTrustReport, TrustCategory,
};
