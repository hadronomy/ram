//! Base visitor trait for HIR
//!
//! This module provides the base visitor trait for traversing the HIR.

use hir::ids::LocalDefId;
use hir::ExprId;
use hir::body::{Body, Instruction, Expr};

/// Result of visiting a node
pub enum VisitResult {
    /// Continue visiting children
    Continue,
    
    /// Skip visiting children
    SkipChildren,
    
    /// Stop visiting entirely
    Stop,
}

/// Context for visitors
pub struct VisitorContext<'a, 'db, 'body> {
    /// Analysis context
    pub analysis_ctx: &'a mut crate::AnalysisContext<'db, 'body>,
}

/// Trait for visiting HIR nodes
pub trait Visitor {
    /// Visit an instruction
    fn visit_instruction(
        &self,
        ctx: &mut VisitorContext,
        instruction: &Instruction,
    ) -> VisitResult {
        // Default implementation visits the operand
        if let Some(operand) = instruction.operand {
            self.visit_expr(ctx, operand);
        }
        
        VisitResult::Continue
    }
    
    /// Visit an expression
    fn visit_expr(
        &self,
        ctx: &mut VisitorContext,
        expr_id: ExprId,
    ) -> VisitResult {
        let body = ctx.analysis_ctx.body();
        
        if let Some(expr) = body.expr(expr_id) {
            match expr {
                Expr::Literal(_) => self.visit_literal(ctx, expr_id),
                Expr::Name(_) => self.visit_name(ctx, expr_id),
                // Add other expression types as needed
                _ => VisitResult::Continue,
            }
        } else {
            VisitResult::Continue
        }
    }
    
    /// Visit a literal expression
    fn visit_literal(
        &self,
        _ctx: &mut VisitorContext,
        _expr_id: ExprId,
    ) -> VisitResult {
        VisitResult::Continue
    }
    
    /// Visit a name expression
    fn visit_name(
        &self,
        _ctx: &mut VisitorContext,
        _expr_id: ExprId,
    ) -> VisitResult {
        VisitResult::Continue
    }
    
    /// Visit a body
    fn visit_body(
        &self,
        ctx: &mut VisitorContext,
        body: &Body,
    ) {
        // Visit all instructions
        for instruction in &body.instructions {
            match self.visit_instruction(ctx, instruction) {
                VisitResult::Continue => {},
                VisitResult::SkipChildren => {},
                VisitResult::Stop => return,
            }
        }
    }
}
