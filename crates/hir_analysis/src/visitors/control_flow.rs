//! Control flow visitor for HIR
//!
//! This module provides a visitor implementation for building control flow graphs.

use hir::ids::LocalDefId;
use hir::ExprId;
use hir::body::{Instruction, Expr};

use crate::analysis::control_flow::ControlFlowGraph;
use crate::visitors::{Visitor, VisitorContext, VisitResult};

/// Visitor for building control flow graphs
pub struct ControlFlowVisitor<'a, 'db, 'body> {
    /// Context for control flow analysis
    ctx: &'a mut crate::AnalysisContext<'db, 'body>,
    
    /// Control flow graph being built
    cfg: ControlFlowGraph,
    
    /// Current basic block
    current_block: usize,
}

impl<'a, 'db, 'body> ControlFlowVisitor<'a, 'db, 'body> {
    /// Create a new control flow visitor
    pub fn new(ctx: &'a mut crate::AnalysisContext<'db, 'body>) -> Self {
        Self {
            ctx,
            cfg: ControlFlowGraph::new(),
            current_block: 0,
        }
    }
    
    /// Build the control flow graph
    pub fn build(&mut self) -> ControlFlowGraph {
        let mut visitor_ctx = VisitorContext {
            analysis_ctx: self.ctx,
        };
        
        // Create the entry block
        self.cfg.add_block();
        
        // Visit the body to build the CFG
        self.visit_body(&mut visitor_ctx, self.ctx.body());
        
        // Return the built CFG
        std::mem::take(&mut self.cfg)
    }
    
    /// Start a new basic block
    fn start_new_block(&mut self) -> usize {
        let new_block = self.cfg.add_block();
        self.current_block = new_block;
        new_block
    }
    
    /// Add an instruction to the current block
    fn add_instruction(&mut self, instruction: &Instruction) {
        self.cfg.add_instruction(self.current_block, instruction.id);
    }
    
    /// Add an edge between blocks
    fn add_edge(&mut self, from: usize, to: usize) {
        self.cfg.add_edge(from, to);
    }
    
    /// Check if an instruction is a terminator (ends a basic block)
    fn is_terminator(&self, instruction: &Instruction) -> bool {
        matches!(
            instruction.opcode.as_str(),
            "JUMP" | "JZERO" | "JGTZ" | "HALT"
        )
    }
    
    /// Get the target of a jump instruction
    fn get_jump_target(&self, instruction: &Instruction) -> Option<LocalDefId> {
        if let Some(operand) = instruction.operand {
            let body = self.ctx.body();
            if let Some(expr) = body.expr(operand) {
                // In a real implementation, we would resolve the label to an instruction ID
                // For now, we'll just return None
                None
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl<'a, 'db, 'body> Visitor for ControlFlowVisitor<'a, 'db, 'body> {
    fn visit_instruction(
        &self,
        ctx: &mut VisitorContext,
        instruction: &Instruction,
    ) -> VisitResult {
        // Add the instruction to the current block
        self.add_instruction(instruction);
        
        // If this is a terminator instruction, end the current block
        if self.is_terminator(instruction) {
            let current_block = self.current_block;
            
            match instruction.opcode.as_str() {
                "JUMP" => {
                    // Unconditional jump
                    if let Some(target) = self.get_jump_target(instruction) {
                        // Add an edge to the target block
                        // In a real implementation, we would resolve the target to a block
                        // For now, we'll just create a new block
                        let target_block = self.start_new_block();
                        self.add_edge(current_block, target_block);
                    }
                },
                "JZERO" | "JGTZ" => {
                    // Conditional jump
                    if let Some(target) = self.get_jump_target(instruction) {
                        // Add an edge to the target block
                        // In a real implementation, we would resolve the target to a block
                        // For now, we'll just create a new block
                        let target_block = self.start_new_block();
                        self.add_edge(current_block, target_block);
                    }
                    
                    // Also add an edge to the fall-through block
                    let fallthrough_block = self.start_new_block();
                    self.add_edge(current_block, fallthrough_block);
                },
                "HALT" => {
                    // No successors for HALT
                },
                _ => unreachable!(),
            }
        }
        
        VisitResult::Continue
    }
}
