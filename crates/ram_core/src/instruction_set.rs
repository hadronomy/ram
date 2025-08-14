//! Instruction set definitions for the RAM virtual machine
//!
//! This module provides a way to define and manage sets of instructions.

use std::collections::HashMap;
use std::sync::Arc;

use dashmap::DashMap;
use once_cell::sync::Lazy;

use crate::instruction::{InstructionDefinition, InstructionInfo, InstructionKind};
use crate::instructions::standard_instructions;
use crate::registry::InstructionRegistry;

/// A set of instructions for the RAM virtual machine
#[derive(Debug, Clone)]
pub struct InstructionSet {
    /// The name of the instruction set
    pub name: String,
    /// A description of the instruction set
    pub description: String,
    /// The registry containing the instructions
    registry: InstructionRegistry,
    /// Metadata about the instruction set
    metadata: HashMap<String, String>,
}

impl InstructionSet {
    /// Create a new empty instruction set
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            registry: InstructionRegistry::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add an instruction to the set
    pub fn add_instruction(
        &mut self,
        kind: InstructionKind,
        definition: Arc<dyn InstructionDefinition>,
    ) -> &mut Self {
        self.registry.register(kind, definition);
        self
    }

    /// Add metadata to the instruction set
    pub fn add_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Get the registry for this instruction set
    pub fn registry(&self) -> &InstructionRegistry {
        &self.registry
    }

    /// Get a mutable reference to the registry for this instruction set
    pub fn registry_mut(&mut self) -> &mut InstructionRegistry {
        &mut self.registry
    }

    /// Get metadata about the instruction set
    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    /// Get a specific metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    /// Get information about all instructions in the set
    pub fn get_all_info(&self) -> Vec<InstructionInfo> {
        self.registry.get_all_info()
    }

    /// Get information about a specific instruction by kind
    pub fn get_info(&self, kind: &InstructionKind) -> Option<InstructionInfo> {
        self.registry.get_info(kind)
    }

    /// Get information about a specific instruction by name (case-sensitive)
    pub fn get_info_by_name(&self, name: &str) -> Option<InstructionInfo> {
        self.registry.get_info_by_name(name)
    }

    /// Get information about a specific instruction by name (case-insensitive)
    pub fn get_info_by_name_case_insensitive(&self, name: &str) -> Option<InstructionInfo> {
        self.registry.get_info_by_name_case_insensitive(name)
    }

    /// Get an instruction definition by kind
    pub fn get(&self, kind: &InstructionKind) -> Option<Arc<dyn InstructionDefinition>> {
        self.registry.get(kind)
    }

    /// Get an instruction definition by name (case-sensitive)
    pub fn get_by_name(&self, name: &str) -> Option<Arc<dyn InstructionDefinition>> {
        self.registry.get_by_name(name)
    }

    /// Get an instruction definition by name (case-insensitive)
    pub fn get_by_name_case_insensitive(
        &self,
        name: &str,
    ) -> Option<Arc<dyn InstructionDefinition>> {
        self.registry.get_by_name_case_insensitive(name)
    }

    /// Check if the set contains an instruction by kind
    pub fn contains(&self, kind: &InstructionKind) -> bool {
        self.registry.contains(kind)
    }

    /// Check if the set contains an instruction by name (case-sensitive)
    pub fn contains_name(&self, name: &str) -> bool {
        self.registry.kind_by_name(name).is_some()
    }

    /// Check if the set contains an instruction by name (case-insensitive)
    pub fn contains_name_case_insensitive(&self, name: &str) -> bool {
        self.registry.kind_by_name_case_insensitive(name).is_some()
    }

    /// Get all instruction kinds in the set
    pub fn kinds(&self) -> impl Iterator<Item = InstructionKind> + '_ {
        self.registry.kinds()
    }

    /// Get all instruction names in the set
    pub fn names(&self) -> impl Iterator<Item = String> + '_ {
        self.registry.names()
    }

    /// Create the standard instruction set
    pub fn standard() -> Self {
        STANDARD_INSTRUCTION_SET.clone()
    }

    /// Merge another instruction set into this one
    pub fn merge(&mut self, other: &InstructionSet) -> &mut Self {
        // Merge the registries
        for kind in other.kinds() {
            if let Some(definition) = other.get(&kind) {
                self.registry.register(kind, definition);
            }
        }

        // Merge metadata (only if not already present)
        for (key, value) in other.metadata() {
            if !self.metadata.contains_key(key) {
                self.metadata.insert(key.clone(), value.clone());
            }
        }

        self
    }
}

/// The standard instruction set for the RAM virtual machine
pub static STANDARD_INSTRUCTION_SET: Lazy<InstructionSet> = Lazy::new(|| {
    let mut set =
        InstructionSet::new("Standard", "The standard instruction set for the RAM virtual machine");

    // Add metadata
    set.add_metadata("version", "1.0.0")
        .add_metadata("author", "RAM VM Team")
        .add_metadata("license", "MIT");

    // Register standard instructions
    let registry = standard_instructions();

    // Copy all instructions from the registry to the set
    for kind in InstructionKind::standard_kinds() {
        if let Some(definition) = registry.get(&kind) {
            set.add_instruction(kind, definition);
        }
    }

    set
});

/// A global registry of all available instruction sets
pub struct InstructionSetRegistry {
    /// Map of instruction set names to instruction sets
    sets: DashMap<String, Arc<InstructionSet>>,
}

impl InstructionSetRegistry {
    /// Create a new empty instruction set registry
    pub fn new() -> Self {
        let mut registry = Self { sets: DashMap::new() };

        // Register the standard instruction set
        registry.register(Arc::new(InstructionSet::standard()));

        registry
    }

    /// Register an instruction set
    pub fn register(&mut self, set: Arc<InstructionSet>) {
        self.sets.insert(set.name.clone(), set);
    }

    /// Get an instruction set by name
    pub fn get(&self, name: &str) -> Option<Arc<InstructionSet>> {
        self.sets.get(name).map(|entry| entry.value().clone())
    }

    /// Check if the registry contains an instruction set
    pub fn contains(&self, name: &str) -> bool {
        self.sets.contains_key(name)
    }

    /// Get all instruction set names
    pub fn names(&self) -> impl Iterator<Item = String> + '_ {
        self.sets.iter().map(|entry| entry.key().clone())
    }

    /// Get all instruction sets
    pub fn sets(&self) -> impl Iterator<Item = Arc<InstructionSet>> + '_ {
        self.sets.iter().map(|entry| entry.value().clone())
    }
}

impl Default for InstructionSetRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// The global instruction set registry
pub static INSTRUCTION_SET_REGISTRY: Lazy<InstructionSetRegistry> =
    Lazy::new(InstructionSetRegistry::new);
