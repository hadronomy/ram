//! Name resolution for HIR
//!
//! This module is responsible for resolving names to their definitions
//! and keeping track of all references to definitions.

use std::collections::HashMap;
use std::sync::Arc;

use base_db::input::FileId;

use crate::ids::{DefId, DefReference, LocalDefId};

/// A fully resolved file with all names resolved to their definitions
#[derive(Debug, Default)]
pub struct ResolvedFile {
    /// Map of names to their definitions in this file
    definitions: HashMap<String, DefId>,

    /// References to definitions in this file
    references: Vec<DefReference>,
}

impl ResolvedFile {
    /// Create a new resolved file
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a definition to the resolved file
    pub fn add_definition(&mut self, name: String, def_id: DefId) {
        self.definitions.insert(name, def_id);
    }

    /// Add a reference to the resolved file
    pub fn add_reference(&mut self, reference: DefReference) {
        self.references.push(reference);
    }

    /// Look up a definition by name
    pub fn lookup_def(&self, name: &str) -> Option<DefId> {
        self.definitions.get(name).copied()
    }

    /// Get all references in this file
    pub fn references(&self) -> &[DefReference] {
        &self.references
    }
}

/// Query implementation for resolving a file
pub(crate) fn resolve_file_query(
    db: &dyn crate::db::HirDatabase,
    file_id: FileId,
) -> Arc<ResolvedFile> {
    // Get the ItemTree for this file
    let item_tree = db.item_tree(file_id);

    // Create a new ResolvedFile
    let mut resolved = ResolvedFile::new();

    // For each module in the ItemTree, create a definition
    for (i, module) in item_tree.modules.iter().enumerate() {
        let def_id = DefId { file_id, local_id: LocalDefId(i as u32) };

        resolved.add_definition(module.name.clone(), def_id);
    }

    // For each label in the ItemTree, create a definition
    let start_id = item_tree.modules.len();
    for (i, label) in item_tree.labels.iter().enumerate() {
        let def_id = DefId { file_id, local_id: LocalDefId((start_id + i) as u32) };

        resolved.add_definition(label.name.clone(), def_id);
    }

    Arc::new(resolved)
}
