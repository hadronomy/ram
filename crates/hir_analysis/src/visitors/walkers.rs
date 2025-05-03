//! Walker implementations for HIR traversal
//!
//! This module provides functions for traversing HIR structures using visitors.
//! These functions implement the traversal logic, calling the appropriate visitor
//! methods for each node type.

use std::ops::ControlFlow;

use hir::body::{Body, Expr, ExprKind, Instruction, Label};
use hir::expr::ExprId;

use super::traits::{Visitor, VisitorResult};

/// Walk a body with a visitor
///
/// This function traverses a body, calling the appropriate visitor methods for each node.
/// It returns the result of the visitor if traversal was interrupted with `ControlFlow::Break`,
/// or calls `visitor.finish()` if traversal completed normally.
///
/// # Parameters
///
/// * `visitor` - The visitor to use for traversal
/// * `body` - The body to traverse
///
/// # Returns
///
/// The result of the visitor
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
///
/// fn count_instructions(body: &Body) -> usize {
///     let mut visitor = InstructionCounter { count: 0 };
///     let result = walk_body(visitor, body);
///     result
/// }
/// ```
pub fn walk_body<V: Visitor>(mut visitor: V, body: &Body) -> V::Result {
    // Visit the body itself
    if let ControlFlow::Break(result) = visitor.visit_body(body) {
        return result;
    }

    // Visit all instructions
    for instruction in &body.instructions {
        if let ControlFlow::Break(result) = walk_instruction(&mut visitor, instruction, body) {
            return result;
        }
    }

    // Visit all labels
    for label in &body.labels {
        if let ControlFlow::Break(result) = walk_label(&mut visitor, label, body) {
            return result;
        }
    }

    // Visit all expressions
    for expr in &body.exprs {
        if let ControlFlow::Break(result) = walk_expr(&mut visitor, expr, body) {
            return result;
        }
    }

    visitor.finish()
}

/// Walk an expression with a visitor
///
/// This function traverses an expression, calling the appropriate visitor methods
/// based on the expression kind.
///
/// # Parameters
///
/// * `visitor` - The visitor to use for traversal
/// * `expr` - The expression to traverse
/// * `body` - The body containing the expression
///
/// # Returns
///
/// `ControlFlow::Continue(())` if traversal should continue, or `ControlFlow::Break(R)`
/// if traversal should stop and return a result of type `R`.
pub fn walk_expr<V: Visitor>(
    visitor: &mut V,
    expr: &Expr,
    body: &Body,
) -> VisitorResult<V::Result> {
    // Visit the expression itself
    if let ControlFlow::Break(result) = visitor.visit_expr(expr) {
        return ControlFlow::Break(result);
    }

    // Visit the expression kind
    match &expr.kind {
        ExprKind::Literal(literal) => visitor.visit_literal(literal),
        ExprKind::LabelRef(label_ref) => visitor.visit_label_ref(label_ref),
        ExprKind::MemoryRef(memory_ref) => {
            // Visit the memory reference
            if let ControlFlow::Break(result) = visitor.visit_memory_ref(memory_ref) {
                return ControlFlow::Break(result);
            }

            // Visit the address expression
            visitor.visit_expr_id(memory_ref.address, body)
        }
        ExprKind::ArrayAccess(array_access) => {
            // Visit the array access
            if let ControlFlow::Break(result) = visitor.visit_array_access(array_access) {
                return ControlFlow::Break(result);
            }

            // Visit the array expression
            if let ControlFlow::Break(result) = visitor.visit_expr_id(array_access.array, body) {
                return ControlFlow::Break(result);
            }

            // Visit the index expression
            visitor.visit_expr_id(array_access.index, body)
        }
        ExprKind::InstructionCall(call) => {
            // Visit the instruction call
            if let ControlFlow::Break(result) = visitor.visit_instruction_call(call) {
                return ControlFlow::Break(result);
            }

            // Visit all operands
            for &operand in &call.operands {
                if let ControlFlow::Break(result) = visitor.visit_expr_id(operand, body) {
                    return ControlFlow::Break(result);
                }
            }

            ControlFlow::Continue(())
        }
    }
}

/// Walk an instruction with a visitor
///
/// This function traverses an instruction, calling the appropriate visitor methods.
///
/// # Parameters
///
/// * `visitor` - The visitor to use for traversal
/// * `instruction` - The instruction to traverse
/// * `body` - The body containing the instruction
///
/// # Returns
///
/// `ControlFlow::Continue(())` if traversal should continue, or `ControlFlow::Break(R)`
/// if traversal should stop and return a result of type `R`.
pub fn walk_instruction<V: Visitor>(
    visitor: &mut V,
    instruction: &Instruction,
    body: &Body,
) -> VisitorResult<V::Result> {
    // Visit the instruction itself
    if let ControlFlow::Break(result) = visitor.visit_instruction(instruction) {
        return ControlFlow::Break(result);
    }

    // Visit the operand if present
    if let Some(operand) = instruction.operand {
        visitor.visit_expr_id(operand, body)
    } else {
        ControlFlow::Continue(())
    }
}

/// Walk a label with a visitor
///
/// This function traverses a label, calling the appropriate visitor methods.
///
/// # Parameters
///
/// * `visitor` - The visitor to use for traversal
/// * `label` - The label to traverse
/// * `body` - The body containing the label (unused but provided for consistency)
///
/// # Returns
///
/// `ControlFlow::Continue(())` if traversal should continue, or `ControlFlow::Break(R)`
/// if traversal should stop and return a result of type `R`.
pub fn walk_label<V: Visitor>(
    visitor: &mut V,
    label: &Label,
    _body: &Body, // Unused but kept for API consistency
) -> VisitorResult<V::Result> {
    // Visit the label itself
    visitor.visit_label(label)
}

/// Walk an expression ID with a visitor
///
/// This function traverses an expression by its ID, calling the appropriate visitor methods.
///
/// # Parameters
///
/// * `visitor` - The visitor to use for traversal
/// * `expr_id` - The ID of the expression to traverse
/// * `body` - The body containing the expression
///
/// # Returns
///
/// `ControlFlow::Continue(())` if traversal should continue, or `ControlFlow::Break(R)`
/// if traversal should stop and return a result of type `R`.
pub fn walk_expr_id<V: Visitor>(
    visitor: &mut V,
    expr_id: ExprId,
    body: &Body,
) -> VisitorResult<V::Result> {
    visitor.visit_expr_id(expr_id, body)
}
