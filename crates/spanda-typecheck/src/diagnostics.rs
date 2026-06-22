//! Compile-time diagnostic records for the Spanda type checker.
//!
use serde::{Deserialize, Serialize};

/// A single type-check or module-link diagnostic with source location.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub message: String,
    pub line: u32,
    pub column: u32,
}
