//! Type checking visitor for HIR
//!
//! This module provides a visitor implementation for type checking HIR.
//! Type checking ensures that operations are performed on values of the correct type,
//! catching potential errors before execution.

use std::collections::HashMap;
use std::sync::Arc;

use hir::expr::ExprId;
use hir::body::{Body, Expr, Instruction};
use hir::ids::LocalDefId;

use crate::types::{Type, TypeId, TypeSystem};
use crate::visitors::{VisitResult, Visitor, VisitorConfig, VisitorContext};

/// Visitor for type checking
///
/// This visitor traverses the HIR and performs type checking on instructions and expressions.
/// It reports type errors as diagnostics and populates the type information in the analysis context.
#[derive(Debug)]
pub struct TypeCheckVisitor {
    /// Type system
    type_system: TypeSystem,

    /// Map of variable names to their types
    var_types: HashMap<String, TypeId>,

    /// Map of label names to their instruction IDs
    label_map: HashMap<String, hir::InstrId>,
}

impl TypeCheckVisitor {
    /// Create a new type checking visitor
    pub fn new() -> Self {
        Self {
            type_system: TypeSystem::new(),
            var_types: HashMap::new(),
            label_map: HashMap::new(),
        }
    }

    /// Run type checking on a body
    pub fn check(mut self, analysis_ctx: &mut crate::AnalysisContext) -> Arc<TypeSystem> {
        // First pass: collect all labels
        let mut ctx = VisitorContext::new(analysis_ctx);
        self.collect_labels(&mut ctx, analysis_ctx.body());

        // Second pass: perform type checking
        let mut ctx = VisitorContext::new(analysis_ctx);
        self.visit_body(&mut ctx, analysis_ctx.body());

        // Return the type system
        Arc::new(self.type_system)
    }

    /// Collect all labels in the body
    fn collect_labels(&mut self, ctx: &mut VisitorContext, body: &Body) {
        for instruction in &body.instructions {
            if let Some(label) = &instruction.label {
                self.label_map.insert(label.clone(), instruction.id);
            }
        }
    }

    /// Get the expected type of an instruction
    fn instruction_type(&self, instruction: &Instruction) -> TypeId {
        match instruction.opcode.as_str() {
            "LOAD" | "STORE" | "ADD" | "SUB" | "MUL" | "DIV" => self.type_system.int_type,
            "JUMP" | "JZERO" | "JGTZ" => self.type_system.address_type,
            "READ" | "WRITE" => self.type_system.int_type,
            "HALT" => self.type_system.void_type,
            _ => self.type_system.error_type,
        }
    }

    /// Get the expected operand type for an instruction
    fn expected_operand_type(&self, instruction: &Instruction) -> TypeId {
        match instruction.opcode.as_str() {
            "LOAD" | "STORE" | "ADD" | "SUB" | "MUL" | "DIV" | "READ" | "WRITE" => {
                self.type_system.address_type
            }
            "JUMP" | "JZERO" | "JGTZ" => self.type_system.label_type,
            _ => self.type_system.unknown_type,
        }
    }

    /// Check if an operand type is compatible with an instruction
    fn check_operand_compatibility(
        &self,
        ctx: &mut VisitorContext,
        instruction: &Instruction,
        operand_type: TypeId,
    ) {
        let expected_type = self.expected_operand_type(instruction);

        // Check if the types are compatible
        if !self.type_system.is_compatible(operand_type, expected_type) {
            // Report a type error
            ctx.error(
                format!(
                    "Type mismatch: instruction '{}' expects operand of type {}, but got {}",
                    instruction.opcode,
                    self.type_system.type_name(expected_type),
                    self.type_system.type_name(operand_type),
                ),
                instruction.span,
            );
        }
    }

    /// Infer the type of an expression
    fn infer_expr_type(&mut self, ctx: &mut VisitorContext, expr_id: ExprId) -> TypeId {
        let body = ctx.body();

        if let Some(expr) = body.expr(expr_id) {
            match expr {
                Expr::Literal(lit) => {
                    // All literals are integers in RAM
                    self.type_system.int_type
                }
                Expr::Name(name) => {
                    // Check if this is a label
                    if self.label_map.contains_key(name) {
                        self.type_system.label_type
                    } else {
                        // Otherwise, it's a variable
                        *self.var_types.entry(name.clone()).or_insert(self.type_system.int_type)
                    }
                }
                // Add other expression types as needed
                _ => self.type_system.unknown_type,
            }
        } else {
            self.type_system.error_type
        }
    }

    /// Set the type of an expression in the analysis context
    fn set_expr_type(&self, ctx: &mut VisitorContext, expr_id: ExprId, type_id: TypeId) {
        ctx.analysis_ctx.type_info_mut().set_expr_type(expr_id, type_id);
    }

    /// Set the type of an instruction in the analysis context
    fn set_instr_type(&self, ctx: &mut VisitorContext, instr_id: hir::InstrId, type_id: TypeId) {
        ctx.analysis_ctx.type_info_mut().set_instr_type(instr_id, type_id);
    }
}

impl Visitor for TypeCheckVisitor {
    fn visit_instruction(
        &mut self,
        ctx: &mut VisitorContext,
        instruction: &Instruction,
    ) -> VisitResult {
        // Set the type of the instruction
        let instr_type = self.instruction_type(instruction);
        self.set_instr_type(ctx, instruction.id, instr_type);

        // Check the operand type if there is one
        if let Some(operand) = instruction.operand {
            // Infer the type of the operand
            let operand_type = self.infer_expr_type(ctx, operand);

            // Set the type in the analysis context
            self.set_expr_type(ctx, operand, operand_type);

            // Check if the operand type is compatible with the instruction
            self.check_operand_compatibility(ctx, instruction, operand_type);
        } else if self.expected_operand_type(instruction) != self.type_system.void_type {
            // Instruction requires an operand but none was provided
            ctx.error(
                format!("Missing operand for instruction '{}'", instruction.opcode),
                instruction.span,
            );
        }

        VisitResult::Continue
    }

    fn visit_expr(&mut self, ctx: &mut VisitorContext, expr_id: ExprId) -> VisitResult {
        // Infer the type of the expression
        let type_id = self.infer_expr_type(ctx, expr_id);

        // Set the type in the analysis context
        self.set_expr_type(ctx, expr_id, type_id);

        VisitResult::Continue
    }
}
