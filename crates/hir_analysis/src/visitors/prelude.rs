//! Common imports for the visitor pattern
//!
//! This module re-exports common types and traits used in the visitor pattern.
//! It's intended to be used with a glob import to bring all necessary components
//! into scope.

pub use std::ops::ControlFlow;

pub use hir::body::{
    ArrayAccess, Body, Expr, ExprKind, Instruction, InstructionCall, Label, Literal, MemoryRef,
};
pub use hir::expr::ExprId;
pub use hir::ids::{DefId, LocalDefId};

pub use super::traits::{Visitor, VisitorResult};
