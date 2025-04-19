//! Module path handling
//!
//! This module defines structures for representing and resolving paths
//! for module imports and references.

use std::fmt;

/// A path in the module tree
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModPath {
    /// The segments of the path
    pub segments: Vec<String>,
}

impl ModPath {
    /// Create a new module path
    pub fn new(segments: Vec<String>) -> Self {
        Self { segments }
    }

    /// Create a module path from a string representation
    pub fn from_string(path: &str) -> Self {
        let segments = path.split("::").map(String::from).collect();
        Self { segments }
    }

    /// Returns true if this path is empty
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// Returns the first segment of the path, if any
    pub fn first_segment(&self) -> Option<&str> {
        self.segments.first().map(|s| s.as_str())
    }

    /// Returns a new path with the first segment removed
    pub fn skip_first(&self) -> Option<Self> {
        if self.segments.len() <= 1 {
            None
        } else {
            Some(Self { segments: self.segments[1..].to_vec() })
        }
    }
}

impl fmt::Display for ModPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.segments.join("::"))
    }
}
