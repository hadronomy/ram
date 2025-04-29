//! Specialized visitors for common analysis tasks
//!
//! This module provides pre-built visitors for common analysis tasks,
//! making it easy to perform standard analyses without writing custom visitors.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use hir::expr::ExprId;
use hir::body::{Body, Instruction, Expr};
use hir::ids::{DefId, LocalDefId};

use crate::visitors::{Visitor, VisitorContext, VisitResult};

/// A visitor that counts instructions by opcode
#[derive(Debug, Default)]
pub struct InstructionCounter {
    /// Map of opcode to count
    pub counts: HashMap<String, usize>,
    
    /// Total number of instructions
    pub total: usize,
}

impl InstructionCounter {
    /// Create a new instruction counter
    pub fn new() -> Self {
        Self {
            counts: HashMap::new(),
            total: 0,
        }
    }
    
    /// Get the count for a specific opcode
    pub fn count_for(&self, opcode: &str) -> usize {
        *self.counts.get(opcode).unwrap_or(&0)
    }
}

impl Visitor for InstructionCounter {
    fn visit_instruction(
        &mut self,
        _ctx: &mut VisitorContext,
        instruction: &Instruction,
    ) -> VisitResult {
        *self.counts.entry(instruction.opcode.clone()).or_insert(0) += 1;
        self.total += 1;
        VisitResult::Continue
    }
}

/// A visitor that collects all labels used in the program
#[derive(Debug, Default)]
pub struct LabelCollector {
    /// Set of all labels
    pub labels: HashSet<String>,
    
    /// Map of label to instruction ID
    pub label_to_instr: HashMap<String, hir::InstrId>,
}

impl LabelCollector {
    /// Create a new label collector
    pub fn new() -> Self {
        Self {
            labels: HashSet::new(),
            label_to_instr: HashMap::new(),
        }
    }
    
    /// Check if a label exists
    pub fn has_label(&self, label: &str) -> bool {
        self.labels.contains(label)
    }
    
    /// Get the instruction ID for a label
    pub fn get_instr_for_label(&self, label: &str) -> Option<hir::InstrId> {
        self.label_to_instr.get(label).copied()
    }
}

impl Visitor for LabelCollector {
    fn visit_instruction(
        &mut self,
        ctx: &mut VisitorContext,
        instruction: &Instruction,
    ) -> VisitResult {
        // Check if this instruction has a label
        if let Some(label) = &instruction.label {
            self.labels.insert(label.clone());
            self.label_to_instr.insert(label.clone(), instruction.id);
        }
        
        // Also check operands for label references
        if let Some(operand) = instruction.operand {
            let body = ctx.analysis_ctx.body();
            if let Some(Expr::Name(name)) = body.expr(operand) {
                // In a real implementation, we would check if this is a label reference
                // For now, we'll just assume all name expressions are label references
                self.labels.insert(name.clone());
            }
        }
        
        VisitResult::Continue
    }
}

/// A visitor that detects unreachable code
#[derive(Debug, Default)]
pub struct UnreachableCodeDetector {
    /// Set of reachable instruction IDs
    pub reachable: HashSet<hir::InstrId>,
    
    /// Set of unreachable instruction IDs
    pub unreachable: HashSet<hir::InstrId>,
    
    /// Whether we've seen a terminator instruction
    seen_terminator: bool,
}

impl UnreachableCodeDetector {
    /// Create a new unreachable code detector
    pub fn new() -> Self {
        Self {
            reachable: HashSet::new(),
            unreachable: HashSet::new(),
            seen_terminator: false,
        }
    }
    
    /// Check if an instruction is reachable
    pub fn is_reachable(&self, instr_id: hir::InstrId) -> bool {
        self.reachable.contains(&instr_id)
    }
    
    /// Check if an instruction is unreachable
    pub fn is_unreachable(&self, instr_id: hir::InstrId) -> bool {
        self.unreachable.contains(&instr_id)
    }
    
    /// Get all unreachable instructions
    pub fn get_unreachable(&self) -> &HashSet<hir::InstrId> {
        &self.unreachable
    }
}

impl Visitor for UnreachableCodeDetector {
    fn visit_instruction(
        &mut self,
        ctx: &mut VisitorContext,
        instruction: &Instruction,
    ) -> VisitResult {
        if self.seen_terminator {
            // This instruction is unreachable
            self.unreachable.insert(instruction.id);
            
            // Report a warning
            ctx.warning(
                format!("Unreachable instruction: {}", instruction.opcode),
                instruction.span,
            );
        } else {
            // This instruction is reachable
            self.reachable.insert(instruction.id);
            
            // Check if this is a terminator
            if instruction.opcode == "HALT" || instruction.opcode == "JUMP" {
                self.seen_terminator = true;
            }
        }
        
        VisitResult::Continue
    }
}

/// A visitor that validates operand types
#[derive(Debug)]
pub struct OperandValidator {
    /// Map of opcode to expected operand type
    opcode_types: HashMap<String, &'static str>,
}

impl OperandValidator {
    /// Create a new operand validator
    pub fn new() -> Self {
        let mut opcode_types = HashMap::new();
        
        // Define expected operand types for each opcode
        opcode_types.insert("LOAD".to_string(), "address");
        opcode_types.insert("STORE".to_string(), "address");
        opcode_types.insert("ADD".to_string(), "address");
        opcode_types.insert("SUB".to_string(), "address");
        opcode_types.insert("MUL".to_string(), "address");
        opcode_types.insert("DIV".to_string(), "address");
        opcode_types.insert("JUMP".to_string(), "label");
        opcode_types.insert("JZERO".to_string(), "label");
        opcode_types.insert("JGTZ".to_string(), "label");
        opcode_types.insert("READ".to_string(), "address");
        opcode_types.insert("WRITE".to_string(), "address");
        
        Self { opcode_types }
    }
    
    /// Get the expected operand type for an opcode
    pub fn expected_type(&self, opcode: &str) -> Option<&'static str> {
        self.opcode_types.get(opcode).copied()
    }
    
    /// Check if an operand is valid for an opcode
    fn is_valid_operand(
        &self,
        ctx: &VisitorContext,
        opcode: &str,
        operand: ExprId,
    ) -> bool {
        let expected_type = match self.expected_type(opcode) {
            Some(t) => t,
            None => return true, // Unknown opcode, assume valid
        };
        
        let body = ctx.analysis_ctx.body();
        if let Some(expr) = body.expr(operand) {
            match expr {
                Expr::Literal(_) => expected_type == "address" || expected_type == "immediate",
                Expr::Name(_) => true, // Names can be labels or variables
                _ => false,
            }
        } else {
            false
        }
    }
}

impl Visitor for OperandValidator {
    fn visit_instruction(
        &mut self,
        ctx: &mut VisitorContext,
        instruction: &Instruction,
    ) -> VisitResult {
        if let Some(operand) = instruction.operand {
            if !self.is_valid_operand(ctx, &instruction.opcode, operand) {
                ctx.error(
                    format!(
                        "Invalid operand for {}: expected {}",
                        instruction.opcode,
                        self.expected_type(&instruction.opcode).unwrap_or("unknown")
                    ),
                    instruction.span,
                );
            }
        } else if self.opcode_types.contains_key(&instruction.opcode) {
            // Opcode requires an operand but none was provided
            ctx.error(
                format!("Missing operand for {}", instruction.opcode),
                instruction.span,
            );
        }
        
        VisitResult::Continue
    }
}

/// A visitor that detects infinite loops
#[derive(Debug, Default)]
pub struct InfiniteLoopDetector {
    /// Set of potential infinite loops (instruction IDs)
    pub infinite_loops: HashSet<hir::InstrId>,
}

impl InfiniteLoopDetector {
    /// Create a new infinite loop detector
    pub fn new() -> Self {
        Self {
            infinite_loops: HashSet::new(),
        }
    }
    
    /// Check if an instruction is part of a potential infinite loop
    pub fn is_infinite_loop(&self, instr_id: hir::InstrId) -> bool {
        self.infinite_loops.contains(&instr_id)
    }
}

impl Visitor for InfiniteLoopDetector {
    fn visit_instruction(
        &mut self,
        ctx: &mut VisitorContext,
        instruction: &Instruction,
    ) -> VisitResult {
        // Check for unconditional jumps to self
        if instruction.opcode == "JUMP" {
            if let Some(operand) = instruction.operand {
                let body = ctx.analysis_ctx.body();
                if let Some(Expr::Name(name)) = body.expr(operand) {
                    // Check if this is a jump to self
                    if let Some(label) = &instruction.label {
                        if label == name {
                            self.infinite_loops.insert(instruction.id);
                            ctx.warning(
                                format!("Potential infinite loop: unconditional jump to self"),
                                instruction.span,
                            );
                        }
                    }
                }
            }
        }
        
        VisitResult::Continue
    }
}
