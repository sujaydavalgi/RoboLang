//! Runtime execution errors surfaced by the Spanda interpreter.
//!

/// Interpreter failure with source line attribution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeError {
    pub message: String,
    pub line: u32,
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (line {})", self.message, self.line)
    }
}

impl RuntimeError {
    pub fn new(message: impl Into<String>, line: u32) -> Self {
        // Build a runtime error tagged with a source line.
        //
        // Parameters:
        // - `message` — human-readable failure description
        // - `line` — 1-based source line where execution failed
        //
        // Returns:
        // Tagged runtime error value.
        //
        // Options:
        // None.
        //
        // Example:
        // let err = RuntimeError::new("division by zero", 42);

        Self {
            message: message.into(),
            line,
        }
    }
}
