//! Type checking visitor for HIR
//!
//! This module provides a visitor implementation for type checking HIR.

use hir::ids::LocalDefId;
use hir::ExprId;
use hir::body::{Instruction, Expr};

use crate::types::{Type, TypeId, TypeSystem};
use crate::visitors::{Visitor, VisitorContext, VisitResult};

/// Visitor for type checking
pub struct TypeCheckVisitor<'a, 'db, 'body> {
    /// Context for type checking
    ctx: &'a mut crate::AnalysisContext<'db, 'body>,
    
    /// Type system
    type_system: TypeSystem,
}

impl<'a, 'db, 'body> TypeCheckVisitor<'a, 'db, 'body> {
    /// Create a new type checking visitor
    pub fn new(ctx: &'a mut crate::AnalysisContext<'db, 'body>) -> Self {
        Self {
            ctx,
            type_system: TypeSystem::new(),
        }
    }
    
    /// Run type checking on the body
    pub fn check(&self) {
        let mut visitor_ctx = VisitorContext {
            analysis_ctx: self.ctx,
        };
        
        self.visit_body(&mut visitor_ctx, self.ctx.body());
    }
    
    /// Get the type of an instruction
    fn instruction_type(&self, instruction: &Instruction) -> TypeId {
        match instruction.opcode.as_str() {
            "LOAD" | "STORE" | "ADD" | "SUB" | "MUL" | "DIV" => self.type_system.int_type,
            "JUMP" | "JZERO" | "JGTZ" => self.type_system.address_type,
            "READ" | "WRITE" => self.type_system.int_type,
            "HALT" => self.type_system.unknown_type,
            _ => self.type_system.error_type,
        }
    }
    
    /// Check if an operand type is compatible with an instruction
    fn check_operand_type(
        &self,
        instruction: &Instruction,
        operand_type: TypeId,
    ) -> bool {
        let instr_type = self.instruction_type(instruction);
        
        // For simplicity, we'll just check if the types are the same
        // In a real implementation, we would have a more sophisticated type compatibility check
        instr_type == operand_type
    }
}

impl<'a, 'db, 'body> Visitor for TypeCheckVisitor<'a, 'db, 'body> {
    fn visit_instruction(
        &self,
        ctx: &mut VisitorContext,
        instruction: &Instruction,
    ) -> VisitResult {
        // Set the type of the instruction
        let instr_type = self.instruction_type(instruction);
        ctx.analysis_ctx.type_info_mut().set_instr_type(instruction.id, instr_type);
        
        // Check the operand type if there is one
        if let Some(operand) = instruction.operand {
            // Visit the operand first to determine its type
            self.visit_expr(ctx, operand);
            
            // Get the type of the operand
            if let Some(operand_type) = ctx.analysis_ctx.type_info().get_expr_type(operand) {
                // Check if the operand type is compatible with the instruction
                if !self.check_operand_type(instruction, operand_type) {
                    // If not, report an error
                    ctx.analysis_ctx.diagnostics_mut().error(
                        format!(
                            "Type mismatch: instruction '{}' expects operand of type {:?}, but got {:?}",
                            instruction.opcode,
                            self.type_system.get_type(instr_type).unwrap(),
                            self.type_system.get_type(operand_type).unwrap(),
                        ),
                        None,
                    );
                }
            }
        }
        
        VisitResult::Continue
    }
    
    fn visit_expr(
        &self,
        ctx: &mut VisitorContext,
        expr_id: ExprId,
    ) -> VisitResult {
        let body = ctx.analysis_ctx.body();
        
        if let Some(expr) = body.expr(expr_id) {
            let type_id = match expr {
                Expr::Literal(lit) => {
                    // All literals are integers in RAM
                    self.type_system.int_type
                },
                Expr::Name(name) => {
                    // Names can be variables (integers) or labels (addresses)
                    // For simplicity, we'll assume all names are variables
                    self.type_system.int_type
                },
                // Add other expression types as needed
                _ => self.type_system.unknown_type,
            };
            
            // Set the type of the expression
            ctx.analysis_ctx.type_info_mut().set_expr_type(expr_id, type_id);
        }
        
        VisitResult::Continue
    }
}
