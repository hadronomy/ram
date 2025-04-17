//! Input types for the database

use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};

/// A unique identifier for a file in the database
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileId(pub u32);

impl fmt::Display for FileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A unique identifier for a source root
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceRootId(pub u32);

impl fmt::Display for SourceRootId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A source root is a set of files that form a single unit of code
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRoot {
    /// The path to the source root
    pub path: PathBuf,

    /// The files in this source root
    pub files: Vec<FileId>,

    /// Map from file paths to file IDs
    pub path_to_file: HashMap<PathBuf, FileId>,
}

impl SourceRoot {
    /// Create a new source root
    pub fn new(path: PathBuf) -> Self {
        Self { path, files: Vec::new(), path_to_file: HashMap::new() }
    }

    /// Add a file to this source root
    pub fn add_file(&mut self, file_id: FileId) {
        self.files.push(file_id);
    }

    /// Add a file with a specific path to this source root
    pub fn add_file_with_path(&mut self, file_id: FileId, path: PathBuf) {
        self.files.push(file_id);
        self.path_to_file.insert(path, file_id);
    }

    /// Resolve a path relative to this source root
    pub fn resolve_path(&self, path: &Path) -> Option<FileId> {
        // First, try to find the exact path in our map
        if let Some(file_id) = self.path_to_file.get(path) {
            return Some(*file_id);
        }

        // If the path is absolute, try to make it relative to the source root
        let relative_path = if path.is_absolute() {
            match path.strip_prefix(&self.path) {
                Ok(rel_path) => rel_path,
                Err(_) => return None, // Path is not under this source root
            }
        } else {
            path
        };

        // Try to find the relative path in our map
        let full_path = self.path.join(relative_path);
        self.path_to_file.get(&full_path).copied()
    }
}
