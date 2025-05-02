//! Change tracking for the database
#![allow(dead_code)]

use std::sync::Arc;

use crate::input::{FileId, SourceRoot, SourceRootId};

/// A change to a file
#[derive(Debug, Clone)]
pub enum FileChange {
    /// The file was created or modified
    Modified {
        /// The file ID
        file_id: FileId,
        /// The new text
        new_text: Arc<str>,
    },

    /// The file was removed
    Removed {
        /// The file ID
        file_id: FileId,
    },
}

/// A change to the database
#[derive(Debug, Default)]
pub struct Change {
    /// Changes to files
    pub files: Vec<FileChange>,

    /// Changes to source roots
    pub roots: Vec<(SourceRootId, Arc<SourceRoot>)>,
}

impl Change {
    /// Create a new empty change
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a file modification
    pub fn modify_file(&mut self, file_id: FileId, new_text: String) {
        self.files.push(FileChange::Modified { file_id, new_text: Arc::from(new_text) });
    }

    /// Add a file removal
    pub fn remove_file(&mut self, file_id: FileId) {
        self.files.push(FileChange::Removed { file_id });
    }

    /// Add a source root change
    pub fn set_source_root(&mut self, id: SourceRootId, root: SourceRoot) {
        self.roots.push((id, Arc::new(root)));
    }
}
