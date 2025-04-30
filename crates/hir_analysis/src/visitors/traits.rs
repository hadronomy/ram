//! Visitor traits for HIR traversal
//!
//! This module defines the core traits for implementing visitors over HIR structures.
//! The main trait is `Visitor`, which provides methods for visiting each type of HIR node.

use std::ops::ControlFlow;

use hir::body::{Body, Expr, Instruction, InstructionCall, Label, Literal, MemoryRef};
use hir::expr::ExprId;

/// Result type for visitor methods
///
/// This type alias represents the result of a visitor method. It uses `std::ops::ControlFlow`
/// to allow visitors to control the traversal flow:
///
/// - `ControlFlow::Continue(())` continues the traversal
/// - `ControlFlow::Break(R)` stops the traversal and returns a result of type `R`
pub type VisitorResult<R> = ControlFlow<R, ()>;

/// Trait for implementing visitors over HIR structures
///
/// This trait provides methods for visiting each type of HIR node. By default,
/// each method continues the traversal without doing anything. Implementors can
/// override specific methods to perform custom actions on specific node types.
///
/// The visitor pattern allows for flexible traversal of the HIR tree, with the
/// ability to control the flow of traversal and accumulate results.
///
/// # Type Parameters
///
/// * `R` - The result type of the visitor
///
/// # Examples
///
/// ```rust
/// use hir_analysis::visitors::{Visitor, VisitorResult, walk_body};
/// use hir::body::{Body, Instruction};
/// use std::ops::ControlFlow;
///
/// struct InstructionCounter {
///     count: usize,
/// }
///
/// impl Visitor for InstructionCounter {
///     type Result = usize;
///
///     fn visit_instruction(&mut self, instruction: &Instruction) -> VisitorResult<Self::Result> {
///         self.count += 1;
///         ControlFlow::Continue(())
///     }
///
///     fn finish(self) -> Self::Result {
///         self.count
///     }
/// }
/// ```
pub trait Visitor {
    /// The result type of the visitor
    ///
    /// This is the type that will be returned when the visitor is finished.
    type Result;

    /// Visit a body
    ///
    /// This method is called when visiting a body. By default, it delegates to the walker
    /// to visit the body's contents.
    fn visit_body(&mut self, _body: &Body) -> VisitorResult<Self::Result> {
        ControlFlow::Continue(())
    }

    /// Visit an expression
    ///
    /// This method is called when visiting an expression. By default, it delegates to the
    /// appropriate method based on the expression kind.
    fn visit_expr(&mut self, _expr: &Expr) -> VisitorResult<Self::Result> {
        ControlFlow::Continue(())
    }

    /// Visit an expression by ID
    ///
    /// This method is called when visiting an expression by its ID. It's useful when
    /// traversing references to expressions.
    fn visit_expr_id(&mut self, expr_id: ExprId, body: &Body) -> VisitorResult<Self::Result> {
        if let Some(expr) = body.exprs.get(expr_id.0 as usize) {
            self.visit_expr(expr)
        } else {
            ControlFlow::Continue(())
        }
    }

    /// Visit a literal expression
    ///
    /// This method is called when visiting a literal expression.
    fn visit_literal(&mut self, _literal: &Literal) -> VisitorResult<Self::Result> {
        ControlFlow::Continue(())
    }

    /// Visit a label reference expression
    ///
    /// This method is called when visiting a label reference expression.
    fn visit_label_ref(&mut self, _label_ref: &hir::body::LabelRef) -> VisitorResult<Self::Result> {
        ControlFlow::Continue(())
    }

    /// Visit a memory reference expression
    ///
    /// This method is called when visiting a memory reference expression.
    fn visit_memory_ref(&mut self, _memory_ref: &MemoryRef) -> VisitorResult<Self::Result> {
        ControlFlow::Continue(())
    }

    /// Visit an instruction call expression
    ///
    /// This method is called when visiting an instruction call expression.
    fn visit_instruction_call(&mut self, _call: &InstructionCall) -> VisitorResult<Self::Result> {
        ControlFlow::Continue(())
    }

    /// Visit an instruction
    ///
    /// This method is called when visiting an instruction.
    fn visit_instruction(&mut self, _instruction: &Instruction) -> VisitorResult<Self::Result> {
        ControlFlow::Continue(())
    }

    /// Visit a label
    ///
    /// This method is called when visiting a label.
    fn visit_label(&mut self, _label: &Label) -> VisitorResult<Self::Result> {
        ControlFlow::Continue(())
    }

    /// Finish the visitor and return the result
    ///
    /// This method is called after traversal is complete to produce the final result.
    fn finish(self) -> Self::Result;
}

/// Trait for implementing visitors that collect results
///
/// This trait extends the `Visitor` trait with methods for collecting results
/// during traversal. It's useful for visitors that need to accumulate data.
pub trait CollectingVisitor: Visitor {
    /// Add a result to the collection
    ///
    /// This method is called to add a result to the collection during traversal.
    fn collect(&mut self, result: Self::Result);
}

/// Trait for implementing visitors that search for specific nodes
///
/// This trait extends the `Visitor` trait with methods for searching for specific
/// nodes during traversal. It's useful for visitors that need to find nodes that
/// match certain criteria.
pub trait SearchingVisitor: Visitor {
    /// The criteria type for searching
    type Criteria;

    /// Check if a node matches the search criteria
    ///
    /// This method is called to check if a node matches the search criteria.
    fn matches(&self, criteria: &Self::Criteria) -> bool;
}
