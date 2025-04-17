//! Database interface for incremental computation

use std::sync::Arc;

use base_db::SourceDatabase;
use base_db::input::FileId;
use ram_syntax::ast;

#[salsa::db]
pub trait HirDefDatabase: SourceDatabase {
    /// Returns the ItemTree for a file.
    fn item_tree(&self, file_id: FileId) -> Arc<crate::item_tree::ItemTree>;
}

/// Represents a file's content and parse result
#[derive(Debug)]
pub struct SourceFile {
    pub text: String,
    pub ast: ast::Program,
}

impl SourceFile {
    /// Creates a new source file from text
    pub fn new(text: String, ast: ast::Program) -> Self {
        Self { text, ast }
    }
}
