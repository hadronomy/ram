//! The ItemTree intermediate representation
//!
//! This module defines the ItemTree, which acts as a condensed "summary"
//! of the top-level items in a source file. The ItemTree sits between
//! the AST and HIR, providing a stable representation that is less affected
//! by edits inside function bodies.

use base_db::input::FileId;
use ram_syntax::{ResolvedNode, ast};

/// A unique identifier for an item within an ItemTree
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ItemTreeId(pub u32); // Make pub for use in hir::lower

/// The ItemTree holds a summary of the top-level items in a source file.
/// It extracts the module structure and item signatures but ignores function bodies.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ItemTree {
    /// A list of modules declared in this file
    pub modules: Vec<ModuleDef>,

    /// A list of use statements in this file
    pub use_stmts: Vec<UseDef>, // Added field

    /// A list of labels declared in this file
    pub labels: Vec<LabelDef>,

    /// Documentation comments attached to items
    pub doc_comments: Vec<DocComment>,
}

/// A module declaration in the ItemTree
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleDef {
    /// The name of the module
    pub name: String,

    /// The ID of this module in the ItemTree
    pub id: ItemTreeId,

    /// The source location of this module declaration
    pub source: ItemSource,
}

/// A use statement in the ItemTree
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UseDef {
    /// The path being imported
    pub path: ModulePath,

    /// The ID of this use statement in the ItemTree
    pub id: ItemTreeId,

    /// The source location of this use statement
    pub source: ItemSource,
}

/// Represents the path in a `use` statement
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModulePath {
    /// A simple path like `module::symbol` or `module::*`
    Simple {
        /// The module name identifier
        module: String,
        /// The symbol or wildcard being imported
        symbol: Option<String>, // None represents '*'
    },
    /// A nested path like `module::submodule::symbol` (potentially longer)
    Nested {
        /// The segments of the path (e.g., ["module", "submodule", "symbol"])
        segments: Vec<String>,
        /// Indicates if the last segment is a wildcard '*'
        is_wildcard: bool,
    },
}

/// A label declaration in the ItemTree
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabelDef {
    /// The name of the label
    pub name: String,

    /// The ID of this label in the ItemTree
    pub id: ItemTreeId,

    /// The source location of this label
    pub source: ItemSource,
}

/// Documentation comment attached to an item
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocComment {
    /// The text of the documentation comment
    pub text: String,

    /// The item ID this documentation is attached to
    pub item_id: ItemTreeId,
}

/// Source location information for an item
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemSource {
    /// The file ID that contains this item
    pub file_id: FileId,

    /// The syntax node that defines this item
    pub syntax_node: ResolvedNode,
}

impl ItemTree {
    /// Creates a new empty ItemTree
    pub fn new() -> Self {
        Self::default()
    }

    /// Lowers an AST node into an ItemTree
    pub fn lower(ast: &ast::Program, file_id: FileId) -> Self {
        crate::lower::lower_program(ast, file_id)
    }
}
