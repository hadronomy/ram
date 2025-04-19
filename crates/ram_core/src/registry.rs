//! Instruction registry for the RAM virtual machine

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use crate::instruction::{InstructionDefinition, InstructionKind};

/// Registry for instruction definitions
pub struct InstructionRegistry {
    /// Map of instruction kinds to their definitions
    definitions: HashMap<InstructionKind, Arc<dyn InstructionDefinition>>,
}

impl fmt::Debug for InstructionRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InstructionRegistry")
            .field("definitions", &self.definitions.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl Clone for InstructionRegistry {
    fn clone(&self) -> Self {
        Self { definitions: self.definitions.clone() }
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
        Self { definitions: HashMap::new() }
    }

    /// Register an instruction definition
    pub fn register(&mut self, kind: InstructionKind, definition: Arc<dyn InstructionDefinition>) {
        self.definitions.insert(kind, definition);
    }

    /// Get the instruction definition for a given kind
    pub fn get(&self, kind: &InstructionKind) -> Option<Arc<dyn InstructionDefinition>> {
        self.definitions.get(kind).cloned()
    }

    /// Check if the registry contains a definition for the given kind
    pub fn contains(&self, kind: &InstructionKind) -> bool {
        self.definitions.contains_key(kind)
    }

    /// Get all registered instruction kinds
    pub fn kinds(&self) -> impl Iterator<Item = &InstructionKind> {
        self.definitions.keys()
    }
}
