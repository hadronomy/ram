//! AST traits and utilities
//!
//! This module provides the core traits and utilities for working with the AST.
//! The actual node implementations are in the nodes module.

use either::Either;

use crate::ResolvedNode;

/// Trait for all AST node types
pub trait AstNode {
    /// Checks if the given node can be cast to this type
    fn can_cast(node: &ResolvedNode) -> bool;

    /// Attempts to cast the node to this type
    fn cast(node: ResolvedNode) -> Option<Self>
    where
        Self: Sized;

    /// Returns the underlying syntax node
    fn syntax(&self) -> &ResolvedNode;

    #[must_use]
    fn clone_for_update(&self) -> Self
    where
        Self: Sized,
    {
        Self::cast(self.syntax().clone()).unwrap()
    }

    #[must_use]
    fn clone_subtree(&self) -> Self
    where
        Self: Sized,
    {
        Self::cast(self.syntax().clone()).unwrap()
    }
}

/// Iterator over children of a specific AST node type.
/// Filters out nodes that aren't the correct type.
#[derive(Debug, Clone)]
pub struct AstChildren<'a, N: AstNode> {
    parent: &'a ResolvedNode,
    position: usize,
    _marker: std::marker::PhantomData<N>,
}

impl<'a, N: AstNode> AstChildren<'a, N> {
    pub fn new(parent: &'a ResolvedNode) -> Self {
        Self { parent, position: 0, _marker: std::marker::PhantomData }
    }
}

#[allow(clippy::elidable_lifetime_names)]
impl<'a, N: AstNode> Iterator for AstChildren<'a, N> {
    type Item = N;

    fn next(&mut self) -> Option<Self::Item> {
        // Get the remaining children starting from our current position
        let children = self.parent.children().skip(self.position);

        for child in children {
            self.position += 1;
            if let Some(node) = N::cast(child.clone()) {
                return Some(node);
            }
        }

        None
    }
}

impl<L, R> AstNode for Either<L, R>
where
    L: AstNode,
    R: AstNode,
{
    fn can_cast(node: &ResolvedNode) -> bool {
        L::can_cast(node) || R::can_cast(node)
    }

    fn cast(node: ResolvedNode) -> Option<Self>
    where
        Self: Sized,
    {
        if L::can_cast(&node) {
            L::cast(node.clone()).map(Either::Left)
        } else {
            R::cast(node).map(Either::Right)
        }
    }

    fn syntax(&self) -> &ResolvedNode {
        match self {
            Either::Left(l) => l.syntax(),
            Either::Right(r) => r.syntax(),
        }
    }
}

// Re-export all node types from the nodes module
pub use crate::nodes::*;
