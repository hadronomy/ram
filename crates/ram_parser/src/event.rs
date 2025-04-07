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

    /// Start a new node that will be positioned before the node at the given position
    StartNodeBefore { kind: SyntaxKind, before_pos: usize },

    /// Placeholder for a future node (used during parsing)
    Placeholder { kind_slot: SyntaxKind },

    /// Add a token with the specified kind, text, and span
    AddToken { kind: SyntaxKind, text: String, span: Range<usize> },

    /// Finish the current node
    FinishNode,

    /// Report an error at the specified position
    Error { msg: String },

    /// Special marker for nodes that have been abandoned
    Tombstone,

    /// Split a float token into parts
    FloatSplitHack { ends_in_dot: bool },
}
