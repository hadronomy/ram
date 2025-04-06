//! This module provides a way to construct a syntax tree.
//! It is intended to be completely decoupled from the
//! parser, so as to allow to evolve the tree representation
//! without affecting the parser.

use std::ops::Range;

use crate::SyntaxKind;

/// [`crate::Parser`] produces a flat list of [`Event`]s.
/// They are converted to a tree-structure in
/// a separate pass, via `TreeBuilder`
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// Start a new node with the specified kind
    StartNode { kind: SyntaxKind },
    /// Add a token with the specified kind, text, and span
    AddToken { kind: SyntaxKind, text: String, span: Range<usize> },
    /// Finish the current node
    FinishNode,
    /// Report an error at the specified position
    Error { message: String, pos: usize },
}
