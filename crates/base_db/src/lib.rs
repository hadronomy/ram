//! Base database traits for the RAM compiler.
//!
//! This crate defines the fundamental database traits and structures
//! that are used throughout the compiler. It provides a foundation for
//! incremental computation using salsa.

mod change;
pub mod input;

use std::hash::BuildHasherDefault;
use std::sync::Arc;

use dashmap::DashMap;
use dashmap::mapref::entry::Entry;
use rustc_hash::FxHasher;
use salsa::{Durability, Setter};
pub use {indexmap, la_arena, salsa, typed_arena};

pub use crate::change::FileChange;
pub use crate::input::{FileId, SourceRoot, SourceRootId};

/// Macro for implementing interned keys
#[macro_export]
macro_rules! impl_intern_key {
    ($id:ident, $loc:ident) => {
        #[salsa::interned(no_debug, no_lifetime)]
        pub struct $id {
            pub loc: $loc,
        }

        // If we derive this salsa prints the values recursively, and this causes us to blow.
        impl ::std::fmt::Debug for $id {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.debug_tuple(stringify!($id))
                    .field(&format_args!("{:04x}", self.0.as_u32()))
                    .finish()
            }
        }
    };
}

/// Default LRU cache capacity for file text
pub const DEFAULT_FILE_TEXT_LRU_CAP: u16 = 16;

/// Default LRU cache capacity for parsing
pub const DEFAULT_PARSE_LRU_CAP: u16 = 128;

/// Files storage for the database
#[derive(Debug, Default)]
pub struct Files {
    files: Arc<DashMap<FileId, FileText, BuildHasherDefault<FxHasher>>>,
    source_roots: Arc<DashMap<SourceRootId, SourceRootInput, BuildHasherDefault<FxHasher>>>,
    file_source_roots: Arc<DashMap<FileId, FileSourceRootInput, BuildHasherDefault<FxHasher>>>,
}

impl Files {
    /// Create a new Files storage
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the text of a file
    pub fn file_text(&self, file_id: FileId) -> FileText {
        *self.files.get(&file_id).expect("Unable to fetch file; this is a bug")
    }

    /// Set the text of a file
    pub fn set_file_text(&self, db: &mut dyn SourceDatabase, file_id: FileId, text: &str) {
        match self.files.entry(file_id) {
            Entry::Occupied(mut occupied) => {
                occupied.get_mut().set_text(db).to(Arc::from(text));
            }
            Entry::Vacant(vacant) => {
                let text = FileText::new(db, Arc::from(text), file_id);
                vacant.insert(text);
            }
        };
    }

    /// Set the text of a file with a specific durability
    pub fn set_file_text_with_durability(
        &self,
        db: &mut dyn SourceDatabase,
        file_id: FileId,
        text: &str,
        durability: Durability,
    ) {
        match self.files.entry(file_id) {
            Entry::Occupied(mut occupied) => {
                occupied.get_mut().set_text(db).with_durability(durability).to(Arc::from(text));
            }
            Entry::Vacant(vacant) => {
                let text =
                    FileText::builder(Arc::from(text), file_id).durability(durability).new(db);
                vacant.insert(text);
            }
        };
    }

    /// Get the source root of a file
    pub fn source_root(&self, source_root_id: SourceRootId) -> SourceRootInput {
        let source_root = self
            .source_roots
            .get(&source_root_id)
            .expect("Unable to fetch source root id; this is a bug");
        *source_root
    }

    /// Set the source root with a specific durability
    pub fn set_source_root_with_durability(
        &self,
        db: &mut dyn SourceDatabase,
        source_root_id: SourceRootId,
        source_root: Arc<SourceRoot>,
        durability: Durability,
    ) {
        match self.source_roots.entry(source_root_id) {
            Entry::Occupied(mut occupied) => {
                occupied.get_mut().set_source_root(db).with_durability(durability).to(source_root);
            }
            Entry::Vacant(vacant) => {
                let source_root =
                    SourceRootInput::builder(source_root).durability(durability).new(db);
                vacant.insert(source_root);
            }
        };
    }

    /// Get the source root of a file
    pub fn file_source_root(&self, id: FileId) -> FileSourceRootInput {
        let file_source_root = self
            .file_source_roots
            .get(&id)
            .expect("Unable to fetch FileSourceRootInput; this is a bug");
        *file_source_root
    }

    /// Set the source root of a file with a specific durability
    pub fn set_file_source_root_with_durability(
        &self,
        db: &mut dyn SourceDatabase,
        id: FileId,
        source_root_id: SourceRootId,
        durability: Durability,
    ) {
        match self.file_source_roots.entry(id) {
            Entry::Occupied(mut occupied) => {
                occupied
                    .get_mut()
                    .set_source_root_id(db)
                    .with_durability(durability)
                    .to(source_root_id);
            }
            Entry::Vacant(vacant) => {
                let file_source_root =
                    FileSourceRootInput::builder(source_root_id).durability(durability).new(db);
                vacant.insert(file_source_root);
            }
        };
    }
}

/// A file ID that can be interned in the salsa database
#[salsa::interned(no_lifetime)]
pub struct InternedFileId {
    pub file_id: FileId,
}

impl InternedFileId {
    /// Get the file ID
    pub fn get(&self, db: &dyn salsa::Database) -> FileId {
        self.file_id(db)
    }
}

/// File text input for salsa
#[salsa::input]
#[derive(Debug)]
pub struct FileText {
    pub text: Arc<str>,
    pub file_id: FileId,
}

/// File source root input for salsa
#[salsa::input]
#[derive(Debug)]
pub struct FileSourceRootInput {
    pub source_root_id: SourceRootId,
}

/// Source root input for salsa
#[salsa::input]
#[derive(Debug)]
pub struct SourceRootInput {
    pub source_root: Arc<SourceRoot>,
}

/// Database trait for source code and project model
#[salsa::db]
pub trait SourceDatabase: salsa::Database {
    /// Text of the file
    fn file_text(&self, file_id: FileId) -> FileText;

    /// Set the text of a file
    fn set_file_text(&mut self, file_id: FileId, text: &str);

    /// Set the text of a file with a specific durability
    fn set_file_text_with_durability(
        &mut self,
        file_id: FileId,
        text: &str,
        durability: Durability,
    );

    /// Contents of the source root
    fn source_root(&self, id: SourceRootId) -> SourceRootInput;

    /// Source root of the file
    fn file_source_root(&self, id: FileId) -> FileSourceRootInput;

    /// Set the source root of a file with a specific durability
    fn set_file_source_root_with_durability(
        &mut self,
        id: FileId,
        source_root_id: SourceRootId,
        durability: Durability,
    );

    /// Set the source root with a specific durability
    fn set_source_root_with_durability(
        &mut self,
        source_root_id: SourceRootId,
        source_root: Arc<SourceRoot>,
        durability: Durability,
    );
}
