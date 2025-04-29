//! Control flow visitor for HIR
//!
//! This module provides a visitor implementation for building control flow graphs.
//! The control flow graph (CFG) represents the flow of execution through a program,
//! with nodes representing basic blocks and edges representing possible control flow paths.

use std::collections::HashMap;
use std::sync::Arc;

use hir::expr::ExprId;
use hir::body::{Body, Expr, Instruction};
use hir::ids::LocalDefId;

use crate::analysis::control_flow::{BasicBlock, ControlFlowGraph};
use crate::visitors::{VisitResult, Visitor, VisitorConfig, VisitorContext};

/// Visitor for building control flow graphs
///
/// This visitor traverses the HIR and constructs a control flow graph (CFG)
/// representing the possible execution paths through the program.
#[derive(Debug)]
pub struct ControlFlowVisitor {
    /// Control flow graph being built
    cfg: ControlFlowGraph,

    /// Current basic block
    current_block: usize,

    /// Map of label names to instruction IDs
    label_map: HashMap<String, hir::expr::ExprId>,

    /// Map of instruction IDs to basic blocks
    instr_to_block: HashMap<hir::expr::ExprId, usize>,

    /// Whether we're in the first pass (collecting labels)
    first_pass: bool,
}

impl ControlFlowVisitor {
    /// Create a new control flow visitor
    pub fn new() -> Self {
        Self {
            cfg: ControlFlowGraph::new(),
            current_block: 0,
            label_map: HashMap::new(),
            instr_to_block: HashMap::new(),
            first_pass: true,
        }
    }

    /// Build the control flow graph for a body
    pub fn build(mut self, analysis_ctx: &mut crate::AnalysisContext) -> Arc<ControlFlowGraph> {
        // First pass: collect all labels
        let mut ctx = VisitorContext::new(analysis_ctx);
        self.first_pass = true;
        self.visit_body(&mut ctx, analysis_ctx.body());

        // Reset for second pass
        self.first_pass = false;
        self.cfg = ControlFlowGraph::new();

        // Create the entry block
        self.cfg.add_block();

        // Second pass: build the CFG
        let mut ctx = VisitorContext::new(analysis_ctx);
        self.visit_body(&mut ctx, analysis_ctx.body());

        // Finalize the CFG
        self.finalize_cfg();

        // Return the built CFG
        Arc::new(self.cfg)
    }

    /// Finalize the control flow graph
    fn finalize_cfg(&mut self) {
        // Add any missing edges or perform final optimizations
        // For example, we might want to merge adjacent blocks with no branches

        // For now, we'll just ensure all blocks have at least one instruction
        for block_id in 0..self.cfg.block_count() {
            if self.cfg.block_instructions(block_id).is_empty() {
                // Empty block, add a dummy instruction or remove it
                // For now, we'll just leave it as is
            }
        }
    }

    /// Start a new basic block
    fn start_new_block(&mut self) -> usize {
        let new_block = self.cfg.add_block();
        self.current_block = new_block;
        new_block
    }

    /// Add an instruction to the current block
    fn add_instruction(&mut self, instruction: &Instruction) {
        if !self.first_pass {
            self.cfg.add_instruction(self.current_block, instruction.id);
            self.instr_to_block.insert(instruction.id, self.current_block);
        }
    }

    /// Add an edge between blocks
    fn add_edge(&mut self, from: usize, to: usize) {
        if !self.first_pass {
            self.cfg.add_edge(from, to);
        }
    }

    /// Check if an instruction is a terminator (ends a basic block)
    fn is_terminator(&self, instruction: &Instruction) -> bool {
        matches!(instruction.opcode.as_str(), "JUMP" | "JZERO" | "JGTZ" | "HALT")
    }

    /// Get the target block of a jump instruction
    fn get_jump_target_block(&self, instruction: &Instruction) -> Option<usize> {
        if let Some(operand) = instruction.operand {
            let body = instruction.body();
            if let Some(Expr::Name(name)) = body.expr(operand) {
                // Look up the label in our map
                if let Some(instr_id) = self.label_map.get(name) {
                    // Look up the block containing this instruction
                    return self.instr_to_block.get(instr_id).copied();
                }
            }
        }
        None
    }

    /// Process a label definition
    fn process_label(&mut self, instruction: &Instruction) {
        if let Some(label) = &instruction.label {
            if self.first_pass {
                // First pass: collect the label
                self.label_map.insert(label.clone(), instruction.id);
            }
        }
    }
}

impl Visitor for ControlFlowVisitor {
    fn visit_instruction(
        &mut self,
        ctx: &mut VisitorContext,
        instruction: &Instruction,
    ) -> VisitResult {
        // Process any labels on this instruction
        self.process_label(instruction);

        if self.first_pass {
            // First pass: just collect labels
            return VisitResult::Continue;
        }

        // Add the instruction to the current block
        self.add_instruction(instruction);

        // If this is a terminator instruction, end the current block
        if self.is_terminator(instruction) {
            let current_block = self.current_block;

            match instruction.opcode.as_str() {
                "JUMP" => {
                    // Unconditional jump
                    if let Some(target_block) = self.get_jump_target_block(instruction) {
                        // Add an edge to the target block
                        self.add_edge(current_block, target_block);
                    } else {
                        // Unknown target, create a new block
                        let target_block = self.start_new_block();
                        self.add_edge(current_block, target_block);
                    }
                }
                "JZERO" | "JGTZ" => {
                    // Conditional jump
                    if let Some(target_block) = self.get_jump_target_block(instruction) {
                        // Add an edge to the target block
                        self.add_edge(current_block, target_block);
                    } else {
                        // Unknown target, create a new block
                        let target_block = self.start_new_block();
                        self.add_edge(current_block, target_block);
                    }

                    // Also add an edge to the fall-through block
                    let fallthrough_block = self.start_new_block();
                    self.add_edge(current_block, fallthrough_block);
                }
                "HALT" => {
                    // No successors for HALT
                    // But we'll start a new block for any code that follows
                    self.start_new_block();
                }
                _ => unreachable!(),
            }
        }

        VisitResult::Continue
    }

    fn visit_body(&mut self, ctx: &mut VisitorContext, body: &Body) {
        // Visit all instructions
        for instruction in &body.instructions {
            let result = self.visit_instruction(ctx, instruction);

            if result.should_stop() {
                return;
            }

            if result.should_skip_siblings() {
                break;
            }
        }
    }
}
