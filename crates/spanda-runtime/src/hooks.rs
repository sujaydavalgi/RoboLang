//! Optional runtime hooks injected by the driver or API layer.
//!
use spanda_ast::nodes::Program;
use std::sync::Arc;

/// Extension points for platform services wired outside the interpreter core.
pub trait RuntimeHooks: Send + Sync {
    /// Enforce deploy certification before mission execution when requested.
    fn enforce_certification(&self, program: &Program, enforce: bool) -> Result<(), String> {
        let _ = (program, enforce);
        Ok(())
    }
}

/// Default no-op hooks for direct interpreter use without a driver.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopRuntimeHooks;

impl RuntimeHooks for NoopRuntimeHooks {}

/// Shared hook handle passed through run options at the driver boundary.
pub type SharedRuntimeHooks = Arc<dyn RuntimeHooks>;
