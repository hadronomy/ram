//! Syntax definitions for the RAM assembly language.
//!
//! This crate provides the syntax tree definitions and AST wrappers for the RAM assembly language.
//! It is used by the parser to build a syntax tree from source code.

pub mod ast;
pub mod nodes;
mod syntax_kind;

pub use ast::*;
pub use cstree;
pub use syntax_kind::*;
