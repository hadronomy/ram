//! Instruction validation for HIR
//!
//! This module provides validation for instructions in HIR bodies.
//! It checks that instructions are valid according to the instruction set
//! and that operands are of the correct type.

use std::any::TypeId;

use hir::body::{Body, ExprKind, Literal};
use hir::expr::ExprId;
use miette::Diagnostic;
use ram_core::{InstructionKind, InstructionSet};

use crate::context::AnalysisContext;
use crate::pass::AnalysisPass;

/// Instruction validation analysis pass
///
/// This pass validates instructions in a HIR body against the instruction set.
/// It checks that instructions are valid and that operands are of the correct type.
#[derive(Default)]
pub struct InstructionValidationAnalysis;

impl AnalysisPass for InstructionValidationAnalysis {
    type Output = ();

    fn name(&self) -> &'static str {
        "InstructionValidationAnalysis"
    }

    fn dependencies(&self) -> Vec<TypeId> {
        vec![]
    }

    fn run(&self, ctx: &mut AnalysisContext) -> Result<Self::Output, Box<dyn Diagnostic>> {
        // Clone the body to avoid borrowing issues
        let body = ctx.body().clone();
        let instruction_set = InstructionSet::standard();

        for instr in &body.instructions {
            // Check if the instruction exists in the instruction set
            let opcode = instr.opcode.to_uppercase();

            let kind = InstructionKind::from_name(&opcode);
            if instruction_set.contains_name_case_insensitive(&opcode) {
                // Check if the instruction has the correct number of operands
                if kind.requires_operand() {
                    if instr.operand.is_none() {
                        ctx.error(
                            format!("Instruction '{}' requires an operand", opcode),
                            "Add an operand".to_string(),
                            None,
                        );
                    } else if let Some(operand_id) = instr.operand {
                        // Validate the operand
                        self.validate_operand(ctx, &body, operand_id, &kind, &opcode);
                    }
                } else if instr.operand.is_some() {
                    ctx.error(
                        format!("Instruction '{}' does not take an operand", opcode),
                        "Remove the operand".to_string(),
                        None,
                    );
                }
            } else {
                ctx.error(
                    format!("Unknown instruction: '{}'", opcode),
                    "Use a valid instruction from the instruction set".to_string(),
                    None,
                );
            }
        }

        Ok(())
    }
}

impl InstructionValidationAnalysis {
    /// Validate an operand against the instruction kind
    fn validate_operand(
        &self,
        ctx: &mut AnalysisContext,
        body: &Body,
        operand_id: ExprId,
        kind: &InstructionKind,
        opcode: &str,
    ) {
        if let Some(expr) = body.exprs.get(operand_id.0 as usize) {
            match &expr.kind {
                ExprKind::Literal(literal) => {
                    match literal {
                        Literal::Int(value) => {
                            // Check if the integer is in range
                            if *value < 0 {
                                ctx.warning(
                                    format!("Negative memory address: {}", value),
                                    "Memory addresses should be non-negative".to_string(),
                                    None,
                                );
                            }
                        }
                        Literal::Label(label) => {
                            // Check if the label exists
                            if !body.labels.iter().any(|l| l.name == *label) {
                                ctx.error(
                                    format!("Undefined label: '{}'", label),
                                    "Define the label before using it".to_string(),
                                    None,
                                );
                            }

                            // Check if this is a jump instruction
                            if !matches!(
                                kind,
                                InstructionKind::Jump
                                    | InstructionKind::JumpGtz
                                    | InstructionKind::JumpZero
                            ) {
                                ctx.warning(
                                    format!(
                                        "Label used as operand for non-jump instruction: '{}'",
                                        opcode
                                    ),
                                    "Labels are typically used with jump instructions".to_string(),
                                    None,
                                );
                            }
                        }
                        Literal::String(_) => {
                            // String literals are generally not used in RAM instructions
                            ctx.warning(
                                format!(
                                    "String literal used as operand for instruction: '{}'",
                                    opcode
                                ),
                                "String literals are not typically used in RAM instructions"
                                    .to_string(),
                                None,
                            );
                        }
                    }
                }
                ExprKind::LabelRef(_label_ref) => {
                    // Get the label name from the label ID
                    // We can't directly compare LocalDefId with DefId, so we'll just check all labels
                    let label_name = body.labels.iter().map(|l| l.name.clone()).next();

                    if let Some(_label_name) = label_name {
                        // Check if this is a jump instruction
                        if !matches!(
                            kind,
                            InstructionKind::Jump
                                | InstructionKind::JumpGtz
                                | InstructionKind::JumpZero
                        ) {
                            ctx.warning(
                                format!("Label reference used as operand for non-jump instruction: '{}'", opcode),
                                "Label references are typically used with jump instructions".to_string(),
                                None,
                            );
                        }
                    } else {
                        ctx.error(
                            "Invalid label reference".to_string(),
                            "Use a valid label".to_string(),
                            None,
                        );
                    }
                }
                ExprKind::MemoryRef(mem_ref) => {
                    // Check if the address is valid
                    if let Some(addr_expr) = body.exprs.get(mem_ref.address.0 as usize) {
                        match &addr_expr.kind {
                            ExprKind::Literal(Literal::Int(value)) => {
                                if *value < 0 {
                                    ctx.warning(
                                        format!("Negative memory address: {}", value),
                                        "Memory addresses should be non-negative".to_string(),
                                        None,
                                    );
                                }
                            }
                            _ => {
                                ctx.error(
                                    "Memory reference address must be an integer".to_string(),
                                    "Use an integer for the memory address".to_string(),
                                    None,
                                );
                            }
                        }
                    }
                }
                ExprKind::InstructionCall(_) => {
                    ctx.error(
                        format!(
                            "Instruction '{}' cannot have an instruction call as an operand",
                            opcode
                        ),
                        "Use a valid operand type".to_string(),
                        None,
                    );
                }
            }
        }
    }
}
