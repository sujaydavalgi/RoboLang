//! Variable binding environment for the Spanda interpreter.
//!
use crate::value::{format_runtime_value, RuntimeValue};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Environment {
    bindings: HashMap<String, RuntimeValue>,
}

impl Environment {
    pub fn new() -> Self {
        // Create a new instance.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // A new instance of this type.
        //
        // Options:
        // None.
        //
        // Example:
        // let value = spanda_core::runtime::new();

        // Assemble the struct fields and return it.
        Self {
            bindings: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: impl Into<String>, value: RuntimeValue) {
        // Define.
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — input value
        // - `value` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.define(name, value);

        // Append into self.
        self.bindings.insert(name.into(), value);
    }

    pub fn get(&self, name: &str) -> Option<&RuntimeValue> {
        // Get.
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — input value
        //
        // Returns:
        // Some value on success, otherwise none.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.get(name);

        // Call get on the current instance.
        self.bindings.get(name)
    }

    pub fn remove(&mut self, name: &str) -> Option<RuntimeValue> {
        // Remove a binding from the environment.
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — binding name to drop
        //
        // Returns:
        // Removed value when the binding existed.
        //
        // Options:
        // None.
        //
        // Example:
        // let old = env.remove("counter");

        self.bindings.remove(name)
    }

    pub fn set(&mut self, name: impl Into<String>, value: RuntimeValue) {
        // Set.
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — input value
        // - `value` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.set(name, value);

        // Append into self.
        self.bindings.insert(name.into(), value);
    }

    pub fn clone_bindings(&self) -> Self {
        // Clone bindings.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // A new instance of this type.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.clone_bindings();

        // Assemble the struct fields and return it.
        Self {
            bindings: self.bindings.clone(),
        }
    }

    pub fn snapshot_display(&self) -> std::collections::HashMap<String, String> {
        // Snapshot display.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // std::collections::HashMap<String, String>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.snapshot_display();

        // Call bindings on the current instance.
        self.bindings
            .iter()
            .map(|(name, value)| (name.clone(), format_runtime_value(value)))
            .collect()
    }
}

impl Default for Environment {
    fn default() -> Self {
        //
        // Parameters:
        // None.
        //
        // Returns:
        // A new instance of this type.
        //
        // Options:
        // None.
        //
        // Example:
        // let value = spanda_core::runtime::default();

        // Build the result via new.
        Self::new()
    }
}
