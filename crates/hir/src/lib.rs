//! High-level Intermediate Representation (HIR)
//!
//! This crate defines the HIR, which is the core data structure for semantic
//! analysis in the RAM compiler. HIR is derived from the AST but incorporates
//! significantly more semantic context.
//!
//! The HIR represents the semantic reality of the code after considering
//! configuration flags and enabled features. It is a crate-specific representation
//! that reflects the actual semantics that will be executed.

pub mod body;
pub mod db;
pub mod expr;
pub mod ids;
pub mod lower;
pub mod name_resolution;
pub mod source_analyzer;
pub mod ty;

/// The HIR crate facade that provides access to the HIR-level APIs
#[derive(Debug, Default)]
pub struct Hir {
    _private: (),
}

impl Hir {
    /// Create a new HIR facade
    pub fn new() -> Self {
        Self { _private: () }
    }
}
