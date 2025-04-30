//! Visitor pattern implementation for HIR
//!
//! This module provides a visitor pattern implementation for traversing HIR structures.
//! It offers a flexible and ergonomic API for implementing visitors that can traverse
//! the HIR tree and perform various analyses.
//!
//! # Features
//!
//! - Type-safe visitor pattern with generic return types
//! - Control flow management using `std::ops::ControlFlow`
//! - Default implementations for all visitor methods
//! - Convenient trait implementations for common use cases
//!
//! # Example
//!
//! ```rust
//! use hir_analysis::visitors::{Visitor, VisitorResult, walk_body};
//! use hir::body::{Body, Expr, ExprKind, Instruction, Label};
//! use std::ops::ControlFlow;
//!
//! // A visitor that counts the number of instructions
//! struct InstructionCounter {
//!     count: usize,
//! }
//!
//! impl Visitor for InstructionCounter {
//!     type Result = usize;
//!
//!     fn visit_instruction(&mut self, instruction: &Instruction) -> VisitorResult<Self::Result> {
//!         self.count += 1;
//!         ControlFlow::Continue(())
//!     }
//!
//!     fn finish(self) -> Self::Result {
//!         self.count
//!     }
//! }
//!
//! // Usage
//! fn count_instructions(body: &Body) -> usize {
//!     let visitor = InstructionCounter { count: 0 };
//!     walk_body(visitor, body)
//! }
//! ```

#[cfg(test)]
pub mod examples;
mod prelude;
#[cfg(test)]
mod tests;
mod traits;
mod walkers;

// Re-export the main components
pub use prelude::*;
pub use traits::*;
pub use walkers::{walk_body, walk_expr, walk_expr_id, walk_instruction, walk_label};
