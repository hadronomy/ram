//! Control flow analysis for HIR
//!
//! This module provides control flow analysis for HIR.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use hir::ids::{DefId, LocalDefId};
use hir::body::Body;

use crate::visitors::control_flow::ControlFlowVisitor;

/// A basic block in the control flow graph
#[derive(Debug, Clone)]
pub struct BasicBlock {
    /// ID of the block
    pub id: usize,
    
    /// Instructions in the block
    pub instructions: Vec<LocalDefId>,
    
    /// Predecessor blocks
    pub predecessors: HashSet<usize>,
    
    /// Successor blocks
    pub successors: HashSet<usize>,
}

/// A control flow graph
#[derive(Debug, Clone)]
pub struct ControlFlowGraph {
    /// Basic blocks in the graph
    pub blocks: Vec<BasicBlock>,
    
    /// Entry block
    pub entry_block: usize,
    
    /// Exit blocks
    pub exit_blocks: HashSet<usize>,
    
    /// Map from instruction ID to block ID
    pub instruction_to_block: HashMap<LocalDefId, usize>,
}

impl ControlFlowGraph {
    /// Create a new empty control flow graph
    pub fn new() -> Self {
        Self {
            blocks: Vec::new(),
            entry_block: 0,
            exit_blocks: HashSet::new(),
            instruction_to_block: HashMap::new(),
        }
    }
    
    /// Add a new basic block and return its ID
    pub fn add_block(&mut self) -> usize {
        let id = self.blocks.len();
        self.blocks.push(BasicBlock {
            id,
            instructions: Vec::new(),
            predecessors: HashSet::new(),
            successors: HashSet::new(),
        });
        id
    }
    
    /// Add an instruction to a block
    pub fn add_instruction(&mut self, block_id: usize, instruction_id: LocalDefId) {
        if let Some(block) = self.blocks.get_mut(block_id) {
            block.instructions.push(instruction_id);
            self.instruction_to_block.insert(instruction_id, block_id);
        }
    }
    
    /// Add an edge between blocks
    pub fn add_edge(&mut self, from: usize, to: usize) {
        if let Some(from_block) = self.blocks.get_mut(from) {
            from_block.successors.insert(to);
        }
        
        if let Some(to_block) = self.blocks.get_mut(to) {
            to_block.predecessors.insert(from);
        }
    }
    
    /// Get the block containing an instruction
    pub fn get_block_for_instruction(&self, instruction_id: LocalDefId) -> Option<usize> {
        self.instruction_to_block.get(&instruction_id).copied()
    }
    
    /// Check if an instruction is reachable from the entry block
    pub fn is_instruction_reachable(&self, instruction_id: LocalDefId) -> bool {
        if let Some(block_id) = self.get_block_for_instruction(instruction_id) {
            self.is_block_reachable(block_id)
        } else {
            false
        }
    }
    
    /// Check if a block is reachable from the entry block
    pub fn is_block_reachable(&self, block_id: usize) -> bool {
        let mut visited = HashSet::new();
        let mut worklist = vec![self.entry_block];
        
        while let Some(current) = worklist.pop() {
            if current == block_id {
                return true;
            }
            
            if visited.insert(current) {
                if let Some(block) = self.blocks.get(current) {
                    worklist.extend(block.successors.iter().copied());
                }
            }
        }
        
        false
    }
    
    /// Get all reachable blocks
    pub fn reachable_blocks(&self) -> HashSet<usize> {
        let mut visited = HashSet::new();
        let mut worklist = vec![self.entry_block];
        
        while let Some(current) = worklist.pop() {
            if visited.insert(current) {
                if let Some(block) = self.blocks.get(current) {
                    worklist.extend(block.successors.iter().copied());
                }
            }
        }
        
        visited
    }
    
    /// Get all unreachable blocks
    pub fn unreachable_blocks(&self) -> HashSet<usize> {
        let reachable = self.reachable_blocks();
        (0..self.blocks.len()).filter(|&id| !reachable.contains(&id)).collect()
    }
}

/// Analyze the control flow of a body
pub fn analyze(ctx: &mut crate::AnalysisContext) -> Arc<ControlFlowGraph> {
    let mut visitor = ControlFlowVisitor::new(ctx);
    let cfg = visitor.build();
    Arc::new(cfg)
}

/// Query function for getting the control flow graph of a body
pub(crate) fn control_flow_query(db: &dyn crate::AnalysisDatabase, def_id: DefId) -> Arc<ControlFlowGraph> {
    let body = db.body(def_id);
    let mut context = crate::AnalysisContext::new(db, &body);
    
    analyze(&mut context)
}

/// Query function for checking if an instruction is reachable
pub(crate) fn is_reachable_query(
    db: &dyn crate::AnalysisDatabase,
    def_id: DefId,
    instr_id: LocalDefId,
) -> bool {
    let cfg = db.control_flow_graph(def_id);
    cfg.is_instruction_reachable(instr_id)
}
