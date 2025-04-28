//! Optimization analysis for HIR
//!
//! This module provides optimization analysis for HIR.

use std::collections::HashSet;
use std::sync::Arc;

use hir::ids::{DefId, LocalDefId};
use hir::body::{Body, Instruction};

use crate::analysis::control_flow::ControlFlowGraph;
use crate::analysis::data_flow::{DataFlowResults, Variable};

/// An optimization opportunity
#[derive(Debug, Clone)]
pub enum Optimization {
    /// Dead code elimination
    DeadCode {
        /// Instruction to eliminate
        instruction_id: LocalDefId,
        
        /// Description of the optimization
        description: String,
    },
    
    /// Constant propagation
    ConstantPropagation {
        /// Instruction to optimize
        instruction_id: LocalDefId,
        
        /// Constant value
        value: i32,
        
        /// Description of the optimization
        description: String,
    },
    
    /// Common subexpression elimination
    CommonSubexpression {
        /// Instruction to optimize
        instruction_id: LocalDefId,
        
        /// Equivalent instruction
        equivalent_id: LocalDefId,
        
        /// Description of the optimization
        description: String,
    },
    
    /// Strength reduction
    StrengthReduction {
        /// Instruction to optimize
        instruction_id: LocalDefId,
        
        /// New opcode
        new_opcode: String,
        
        /// Description of the optimization
        description: String,
    },
}

/// Find optimization opportunities in a body
pub fn find_optimizations(ctx: &crate::AnalysisContext) -> Vec<Optimization> {
    let mut optimizations = Vec::new();
    
    if let Some(cfg) = &ctx.control_flow {
        // Find unreachable code
        find_unreachable_code(ctx, cfg, &mut optimizations);
        
        // Find dead stores
        if let Some(data_flow) = &ctx.data_flow {
            find_dead_stores(ctx, cfg, data_flow, &mut optimizations);
        }
    }
    
    optimizations
}

/// Find unreachable code
fn find_unreachable_code(
    ctx: &crate::AnalysisContext,
    cfg: &ControlFlowGraph,
    optimizations: &mut Vec<Optimization>,
) {
    let unreachable_blocks = cfg.unreachable_blocks();
    
    for block_id in unreachable_blocks {
        if let Some(block) = cfg.blocks.get(block_id) {
            for &instr_id in &block.instructions {
                optimizations.push(Optimization::DeadCode {
                    instruction_id: instr_id,
                    description: format!("Instruction is unreachable"),
                });
            }
        }
    }
}

/// Find dead stores
fn find_dead_stores(
    ctx: &crate::AnalysisContext,
    cfg: &ControlFlowGraph,
    data_flow: &DataFlowResults,
    optimizations: &mut Vec<Optimization>,
) {
    for block in &cfg.blocks {
        for &instr_id in &block.instructions {
            let body = ctx.body();
            if let Some(instr) = body.instruction(instr_id) {
                if instr.opcode == "STORE" {
                    // Check if the stored variable is not live after the instruction
                    if let Some(def) = data_flow.def_at.get(&instr_id) {
                        for var in def {
                            if !data_flow.is_live_after(instr_id, var) {
                                optimizations.push(Optimization::DeadCode {
                                    instruction_id: instr_id,
                                    description: format!("Store to {:?} is never used", var),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Query function for finding optimization opportunities in a body
pub(crate) fn optimization_query(db: &dyn crate::AnalysisDatabase, def_id: DefId) -> Arc<Vec<Optimization>> {
    let body = db.body(def_id);
    let mut context = crate::AnalysisContext::new(db, &body);
    
    // Set up the context with control flow and data flow information
    let cfg = db.control_flow_graph(def_id);
    context.set_control_flow_graph(cfg);
    
    let data_flow = db.data_flow_results(def_id);
    context.set_data_flow_results(data_flow);
    
    // Find optimizations
    let optimizations = find_optimizations(&context);
    Arc::new(optimizations)
}
