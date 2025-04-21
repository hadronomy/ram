//! Definition identifiers
//!
//! This module defines the various IDs used to reference definitions
//! within the HIR system.

use std::fmt;

use base_db::input::FileId;

/// A unique identifier for a definition in the HIR
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DefId {
    /// The file containing this definition
    pub file_id: FileId,

    /// The local ID within the file
    pub local_id: LocalDefId,
}

/// A definition ID that is local to a specific file
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct LocalDefId(pub u32);

/// A reference to a definition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DefReference {
    /// The definition being referenced
    pub def_id: DefId,

    /// The source location of this reference
    pub source_file: FileId,

    /// The start offset of this reference
    pub start_offset: usize,

    /// The end offset of this reference
    pub end_offset: usize,
}

impl fmt::Display for DefId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.file_id.0, self.local_id.0)
    }
}

impl fmt::Display for LocalDefId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for DefId {
    fn default() -> Self {
        Self { file_id: FileId(0), local_id: LocalDefId::default() }
    }
}
