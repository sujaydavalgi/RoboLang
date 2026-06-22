//! Multi-file module export registry for type checking linked projects.
//!
use spanda_ast::foundations::{ModuleFnDecl, Visibility};
use spanda_ast::nodes::Program;
use std::collections::HashMap;

/// Exported symbols from a single module.
#[derive(Debug, Clone, Default)]
pub struct ModuleExports {
    pub functions: HashMap<String, ModuleFnDecl>,
}

/// Registry of parsed modules keyed by fully-qualified module name.
#[derive(Debug, Clone, Default)]
pub struct ModuleRegistry {
    modules: HashMap<String, ModuleExports>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, module_name: &str, program: &Program) {
        let Program::Program { functions, .. } = program;
        let mut exports = ModuleExports::default();
        for func in functions {
            let ModuleFnDecl {
                name, visibility, ..
            } = func;
            if matches!(visibility, Visibility::Public | Visibility::Export) {
                exports.functions.insert(name.clone(), func.clone());
            }
        }
        self.modules.insert(module_name.to_string(), exports);
    }

    pub fn from_programs(entries: &[(String, Program)]) -> Self {
        let mut registry = Self::new();
        for (name, program) in entries {
            registry.register(name, program);
        }
        registry
    }

    pub fn exports_for(&self, import_path: &str) -> Option<&ModuleExports> {
        self.modules.get(import_path)
    }

    pub fn function(&self, import_path: &str, name: &str) -> Option<&ModuleFnDecl> {
        self.exports_for(import_path)
            .and_then(|e| e.functions.get(name))
    }

    pub fn module_count(&self) -> usize {
        self.modules.len()
    }
}
