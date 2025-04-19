//! Definition of the ItemTree intermediate representation
//!
//! This crate implements the ItemTree, which serves as a condensed "summary"
//! of top-level items derived from the syntax tree. The ItemTree sits between
//! the raw AST and the complete HIR, providing a stable intermediate representation
//! that is less affected by common code edits.

pub mod db;
pub mod item_scope;
pub mod item_tree;
mod lower;
pub mod path;

pub use crate::item_tree::ItemTree;
