//! Plugin system for extending the RAM virtual machine with custom instructions
//!
//! This module provides a plugin system that allows for registering custom
//! instructions with the RAM virtual machine. Plugins can be loaded dynamically
//! at runtime, and can provide multiple instructions.

use std::sync::Arc;

use crate::instruction::InstructionDefinition;
use crate::registry::InstructionRegistry;

// Define a type alias for the execution function to reduce complexity
type ExecuteFn = Box<
    dyn Fn(
            Option<&crate::operand::Operand>,
            &mut dyn crate::db::VmState,
        ) -> Result<(), crate::error::VmError>
        + Send
        + Sync
        + 'static,
>;

/// A plugin for the RAM virtual machine
pub trait RamPlugin: Send + Sync + 'static {
    /// Get the name of the plugin
    fn name(&self) -> &str;

    /// Get the version of the plugin
    fn version(&self) -> &str;

    /// Get a description of the plugin
    fn description(&self) -> &str;

    /// Register the plugin's instructions with the registry
    fn register(&self, registry: &mut InstructionRegistry);
}

/// A plugin manager for the RAM virtual machine
#[derive(Default, Clone)]
pub struct PluginManager {
    /// The registered plugins
    plugins: Vec<Arc<dyn RamPlugin>>,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        Self { plugins: Vec::new() }
    }

    /// Register a plugin with the manager
    pub fn register_plugin(&mut self, plugin: Arc<dyn RamPlugin>) {
        self.plugins.push(plugin);
    }

    /// Get all registered plugins
    pub fn plugins(&self) -> &[Arc<dyn RamPlugin>] {
        &self.plugins
    }

    /// Get a plugin by name
    pub fn get_plugin(&self, name: &str) -> Option<&Arc<dyn RamPlugin>> {
        self.plugins.iter().find(|p| p.name() == name)
    }

    /// Register all plugins with the instruction registry
    pub fn register_all(&self, registry: &mut InstructionRegistry) {
        for plugin in &self.plugins {
            plugin.register(registry);
        }
    }
}

/// A builder for creating instruction definitions
pub struct InstructionBuilder {
    /// The name of the instruction
    name: String,
    /// Whether the instruction requires an operand
    requires_operand: bool,
    /// The allowed operand kinds
    allowed_operand_kinds: Vec<crate::operand::OperandKind>,
    /// The execution function
    execute_fn: ExecuteFn,
}

impl InstructionBuilder {
    /// Create a new instruction builder with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            requires_operand: true,
            allowed_operand_kinds: vec![],
            execute_fn: Box::new(|_, _| {
                Err(crate::error::VmError::InvalidInstruction(
                    "Instruction not implemented".to_string(),
                ))
            }),
        }
    }

    /// Set whether the instruction requires an operand
    pub fn requires_operand(mut self, requires: bool) -> Self {
        self.requires_operand = requires;
        self
    }

    /// Add an allowed operand kind
    pub fn allow_operand_kind(mut self, kind: crate::operand::OperandKind) -> Self {
        self.allowed_operand_kinds.push(kind);
        self
    }

    /// Set the execution function
    pub fn execute<F>(mut self, f: F) -> Self
    where
        F: Fn(
                Option<&crate::operand::Operand>,
                &mut dyn crate::db::VmState,
            ) -> Result<(), crate::error::VmError>
            + Send
            + Sync
            + 'static,
    {
        self.execute_fn = Box::new(f);
        self
    }

    /// Build the instruction definition
    pub fn build(self) -> Arc<dyn InstructionDefinition> {
        Arc::new(BuiltInstruction {
            name: self.name,
            requires_operand: self.requires_operand,
            allowed_operand_kinds: self.allowed_operand_kinds,
            execute_fn: self.execute_fn,
        })
    }
}

/// An instruction definition built with the builder
struct BuiltInstruction {
    /// The name of the instruction
    name: String,
    /// Whether the instruction requires an operand
    requires_operand: bool,
    /// The allowed operand kinds
    allowed_operand_kinds: Vec<crate::operand::OperandKind>,
    /// The execution function
    execute_fn: ExecuteFn,
}

impl InstructionDefinition for BuiltInstruction {
    fn name(&self) -> &str {
        &self.name
    }

    fn requires_operand(&self) -> bool {
        self.requires_operand
    }

    fn allowed_operand_kinds(&self) -> &[crate::operand::OperandKind] {
        &self.allowed_operand_kinds
    }

    fn execute(
        &self,
        operand: Option<&crate::operand::Operand>,
        vm_state: &mut dyn crate::db::VmState,
    ) -> Result<(), crate::error::VmError> {
        (self.execute_fn)(operand, vm_state)
    }
}

/// A simple implementation of RamPlugin for testing
#[cfg(test)]
pub struct TestPlugin {
    name: String,
    version: String,
    description: String,
    instructions: Vec<(crate::instruction::InstructionKind, Arc<dyn InstructionDefinition>)>,
}

#[cfg(test)]
impl TestPlugin {
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            description: description.into(),
            instructions: Vec::new(),
        }
    }

    pub fn add_instruction(
        mut self,
        kind: crate::instruction::InstructionKind,
        definition: Arc<dyn InstructionDefinition>,
    ) -> Self {
        self.instructions.push((kind, definition));
        self
    }
}

#[cfg(test)]
impl RamPlugin for TestPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn register(&self, registry: &mut InstructionRegistry) {
        for (kind, definition) in &self.instructions {
            registry.register(kind.clone(), definition.clone());
        }
    }
}
