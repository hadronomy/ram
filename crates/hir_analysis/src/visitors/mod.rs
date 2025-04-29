//! Visitor pattern implementations for HIR
//!
//! This module provides a comprehensive visitor pattern implementation for traversing
//! and analyzing HIR (High-level Intermediate Representation) structures. The visitor
//! pattern allows for separation of algorithms from the data structures they operate on,
//! enabling clean, modular code that can be easily extended.
//!
//! # Overview
//!
//! The visitor system is designed around several key components:
//!
//! - **Visitor Traits**: Define the interface for traversing different node types
//! - **Visit Results**: Control the traversal flow
//! - **Visitor Context**: Provides access to analysis data and configuration
//! - **Traversal Strategies**: Support for different traversal orders and patterns
//! - **Visitor Combinators**: Allow composing visitors for complex analyses
//!
//! # Examples
//!
//! ```
//! use hir_analysis::visitors::{Visitor, VisitorContext, VisitResult, VisitorConfig};
//! use hir_analysis::AnalysisContext;
//! use hir::body::{Body, Instruction};
//!
//! // Define a simple visitor
//! struct InstructionCounter {
//!     count: usize,
//! }
//!
//! impl Visitor for InstructionCounter {
//!     fn visit_instruction(&mut self, ctx: &mut VisitorContext, instr: &Instruction) -> VisitResult {
//!         self.count += 1;
//!         VisitResult::Continue
//!     }
//! }
//!
//! // Usage:
//! // let mut counter = InstructionCounter { count: 0 };
//! // counter.run(analysis_ctx);
//! // println!("Found {} instructions", counter.count);
//! ```
//!
//! # Advanced Usage
//!
//! Visitors can be combined and filtered:
//!
//! ```
//! use hir_analysis::visitors::{Visitor, VisitorExt};
//!
//! // Create a visitor that only processes LOAD instructions
//! let load_visitor = MyVisitor.with_filter(|instr| instr.opcode == "LOAD");
//!
//! // Chain multiple visitors together
//! let combined = visitor1.chain(visitor2);
//! ```

pub(crate) mod base;
pub(crate) mod control_flow;
pub(crate) mod specialized;
pub(crate) mod type_check;

// Re-export the core visitor API
pub use base::{
    ChainVisitor, CollectVisitor, FilterVisitor, VisitResult, Visitor, VisitorConfig,
    VisitorContext, VisitorExt,
};
pub use control_flow::ControlFlowVisitor;
pub use specialized::*;
// Re-export specialized visitors
