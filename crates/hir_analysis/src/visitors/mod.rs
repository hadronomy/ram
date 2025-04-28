//! Visitor pattern implementations for HIR
//!
//! This module provides visitor traits and implementations for traversing
//! the HIR in various ways.

mod base;
mod type_check;
mod control_flow;

pub use base::{Visitor, VisitorContext, VisitResult};
pub use type_check::TypeCheckVisitor;
pub use control_flow::ControlFlowVisitor;
