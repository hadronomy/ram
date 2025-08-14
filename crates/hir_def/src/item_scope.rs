//! Item scope for name resolution
//!
//! This module defines the scope structures used for name resolution.
//! A scope holds mappings from names to items defined in that scope.

use std::collections::HashMap;

use crate::item_tree::{ItemTreeId, LabelDef, ModuleDef};

/// A scope containing named items
#[derive(Debug, Default)]
pub struct ItemScope {
    /// Map of module names to their definitions
    modules: HashMap<String, ItemTreeId>,

    /// Map of label names to their definitions
    labels: HashMap<String, ItemTreeId>,
}

impl ItemScope {
    /// Creates a new empty scope
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a module to this scope
    pub fn add_module(&mut self, module: &ModuleDef) {
        self.modules.insert(module.name.clone(), module.id);
    }

    /// Add a label to this scope
    pub fn add_label(&mut self, label: &LabelDef) {
        self.labels.insert(label.name.clone(), label.id);
    }

    /// Look up a module by name
    pub fn lookup_module(&self, name: &str) -> Option<ItemTreeId> {
        self.modules.get(name).copied()
    }

    /// Look up a label by name
    pub fn lookup_label(&self, name: &str) -> Option<ItemTreeId> {
        self.labels.get(name).copied()
    }
}
