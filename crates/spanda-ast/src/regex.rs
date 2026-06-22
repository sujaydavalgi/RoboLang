//! Regex pattern literal type and compilation for Spanda AST nodes.
//!
use crate::nodes::Span;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Named capture groups extracted from a regex match.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CaptureResult {
    pub full: String,
    #[serde(default)]
    pub groups: HashMap<String, String>,
}

/// Source-location diagnostic returned when regex compilation fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegexCompileError {
    pub message: String,
    pub line: u32,
    pub column: u32,
}

/// Compiled regex pattern with optional inline flags (`/pattern/i`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegexPattern {
    pub source: String,
    #[serde(default)]
    pub flags: String,
    pub span: Span,
}

impl RegexPattern {
    pub fn compile(&self) -> Result<Regex, RegexCompileError> {
        // Compile the regex pattern into a Rust regex engine instance.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Compiled regex, or a syntax error diagnostic.
        //
        // Options:
        // None.
        //
        // Example:
        // let re = pattern.compile()?;

        // Build the final pattern string with supported inline flags.
        let mut pattern = self.source.clone();
        for flag in self.flags.chars() {
            // Reject unsupported flag letters early with a clear diagnostic.
            if !matches!(flag, 'i' | 'm' | 's') {
                return Err(RegexCompileError {
                    message: format!(
                        "Invalid regex flag '{flag}'; supported flags are i, m, s. Suggestion: remove unsupported flags."
                    ),
                    line: self.span.start.line,
                    column: self.span.start.column,
                });
            }
        }
        if self.flags.contains('i') && !pattern.starts_with("(?i)") {
            pattern = format!("(?i){pattern}");
        }
        if self.flags.contains('m') && !pattern.starts_with("(?m)") {
            pattern = format!("(?m){pattern}");
        }
        if self.flags.contains('s') && !pattern.starts_with("(?s)") {
            pattern = format!("(?s){pattern}");
        }
        Regex::new(&pattern).map_err(|err| RegexCompileError {
            message: format!(
                "Invalid regex syntax: {err}. Suggestion: verify delimiters and escape sequences."
            ),
            line: self.span.start.line,
            column: self.span.start.column,
        })
    }
}
