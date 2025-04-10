//! TODO: This module will contain the ast wrappers around [`SyntaxNode`]
//!
#![allow(unused_imports)]

use crate::ResolvedNode;

pub trait AstNode {
    fn can_cast(node: &ResolvedNode) -> bool;
    fn cast(node: ResolvedNode) -> Option<Self>
    where
        Self: Sized;
    fn syntax(&self) -> &ResolvedNode;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Program(ResolvedNode);

impl std::fmt::Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.syntax().text())
    }
}

impl AstNode for Program {
    fn can_cast(node: &ResolvedNode) -> bool {
        node.kind() == crate::SyntaxKind::ROOT
    }

    fn cast(node: ResolvedNode) -> Option<Self> {
        if Self::can_cast(&node) { Some(Self(node)) } else { None }
    }

    fn syntax(&self) -> &ResolvedNode {
        &self.0
    }
}
