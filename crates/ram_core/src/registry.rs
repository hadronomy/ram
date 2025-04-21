//! Instruction registry for the RAM virtual machine
//!
//! This module provides a thread-safe registry for instruction definitions, allowing
//! for efficient concurrent lookup of instruction implementations by name or kind.
//! Uses DashMap for better performance in concurrent scenarios.

use std::fmt;
use std::sync::Arc;

use dashmap::DashMap;

use crate::instruction::{InstructionDefinition, InstructionInfo, InstructionKind};

/// Thread-safe registry for instruction definitions
pub struct InstructionRegistry {
    /// Map of instruction kinds to their definitions
    definitions: DashMap<InstructionKind, Arc<dyn InstructionDefinition>>,
    /// Map of instruction names to their kinds for faster lookup
    name_to_kind: DashMap<String, InstructionKind>,
    /// Map of instruction names (lowercase) for case-insensitive lookup
    lowercase_names: DashMap<String, InstructionKind>,
}

impl fmt::Debug for InstructionRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let keys: Vec<_> = self.definitions.iter().map(|entry| entry.key().clone()).collect();
        f.debug_struct("InstructionRegistry").field("definitions", &keys).finish()
    }
}

impl Clone for InstructionRegistry {
    fn clone(&self) -> Self {
        // DashMap implements Clone, so we can just clone the fields
        Self {
            definitions: self.definitions.clone(),
            name_to_kind: self.name_to_kind.clone(),
            lowercase_names: self.lowercase_names.clone(),
        }
    }
}

impl Default for InstructionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl InstructionRegistry {
    /// Create a new empty instruction registry
    pub fn new() -> Self {
        Self {
            definitions: DashMap::new(),
            name_to_kind: DashMap::new(),
            lowercase_names: DashMap::new(),
        }
    }

    /// Register an instruction definition
    pub fn register(&mut self, kind: InstructionKind, definition: Arc<dyn InstructionDefinition>) {
        let name = definition.name().to_string();
        let lowercase_name = name.to_lowercase();

        // Clone the kind for the maps
        let kind_for_name = kind.clone();
        let kind_for_lowercase = kind.clone();

        self.definitions.insert(kind, definition);
        self.name_to_kind.insert(name, kind_for_name);
        self.lowercase_names.insert(lowercase_name, kind_for_lowercase);
    }

    /// Get the instruction definition for a given kind
    pub fn get(&self, kind: &InstructionKind) -> Option<Arc<dyn InstructionDefinition>> {
        self.definitions.get(kind).map(|entry| entry.value().clone())
    }

    /// Check if the registry contains a definition for the given kind
    pub fn contains(&self, kind: &InstructionKind) -> bool {
        self.definitions.contains_key(kind)
    }

    /// Get all registered instruction kinds
    pub fn kinds(&self) -> impl Iterator<Item = InstructionKind> + '_ {
        self.definitions.iter().map(|entry| entry.key().clone())
    }

    /// Get the instruction definition for a given name (case-sensitive)
    pub fn get_by_name(&self, name: &str) -> Option<Arc<dyn InstructionDefinition>> {
        self.name_to_kind.get(name).and_then(|entry| self.get(entry.value()))
    }

    /// Get the instruction definition for a given name (case-insensitive)
    pub fn get_by_name_case_insensitive(
        &self,
        name: &str,
    ) -> Option<Arc<dyn InstructionDefinition>> {
        let lowercase = name.to_lowercase();
        self.lowercase_names.get(&lowercase).and_then(|entry| self.get(entry.value()))
    }

    /// Get the instruction kind for a given name (case-sensitive)
    pub fn kind_by_name(&self, name: &str) -> Option<InstructionKind> {
        self.name_to_kind.get(name).map(|entry| entry.value().clone())
    }

    /// Get the instruction kind for a given name (case-insensitive)
    pub fn kind_by_name_case_insensitive(&self, name: &str) -> Option<InstructionKind> {
        let lowercase = name.to_lowercase();
        self.lowercase_names.get(&lowercase).map(|entry| entry.value().clone())
    }

    /// Get all registered instruction names
    pub fn names(&self) -> impl Iterator<Item = String> + '_ {
        self.name_to_kind.iter().map(|entry| entry.key().clone())
    }

    /// Get information about a registered instruction by kind
    pub fn get_info(&self, kind: &InstructionKind) -> Option<InstructionInfo> {
        Some(kind.info())
    }

    /// Get information about a registered instruction by name (case-sensitive)
    pub fn get_info_by_name(&self, name: &str) -> Option<InstructionInfo> {
        self.kind_by_name(name).map(|kind| kind.info())
    }

    /// Get information about a registered instruction by name (case-insensitive)
    pub fn get_info_by_name_case_insensitive(&self, name: &str) -> Option<InstructionInfo> {
        self.kind_by_name_case_insensitive(name).map(|kind| kind.info())
    }

    /// Get information about all registered instructions
    pub fn get_all_info(&self) -> Vec<InstructionInfo> {
        self.kinds().map(|kind| kind.info()).collect()
    }
}
