//! Spanda compile driver — wires lexer, parser, and type checker.
//!
mod compile;

pub use compile::{
    check, check_with_registry, compile, compile_with_registry, CompileResult,
};
