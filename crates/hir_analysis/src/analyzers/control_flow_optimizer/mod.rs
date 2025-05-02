//! Control flow optimizer for HIR
//!
//! This module provides control flow optimization for HIR bodies.
//! It uses the results of constant propagation analysis to optimize
//! the control flow graph by removing branches that will never be taken.

use std::any::TypeId;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use hir::body::Body;
use hir::ids::LocalDefId;
use miette::Diagnostic;
use petgraph::visit::EdgeRef;

use crate::analyzers::constant_propagation::{
    BranchTaken, ConstantPropagationAnalysis, ConstantPropagationResult,
};
use crate::analyzers::control_flow::{ControlFlowAnalysis, ControlFlowGraph, EdgeKind};
use crate::context::AnalysisContext;
use crate::pass::AnalysisPass;

/// Control flow optimizer pass
///
/// This pass optimizes the control flow graph by removing branches that will never be taken
/// based on the results of constant propagation analysis.
#[derive(Default)]
pub struct ControlFlowOptimizer;

impl AnalysisPass for ControlFlowOptimizer {
    type Output = OptimizedControlFlowGraph;

    fn name(&self) -> &'static str {
        "ControlFlowOptimizer"
    }

    fn dependencies(&self) -> Vec<TypeId> {
        vec![TypeId::of::<ControlFlowAnalysis>(), TypeId::of::<ConstantPropagationAnalysis>()]
    }

    fn run(&self, ctx: &mut AnalysisContext) -> Result<Self::Output, Box<dyn Diagnostic>> {
        // Get the control flow graph
        let cfg = match ctx.get_result::<ControlFlowAnalysis>() {
            Ok(cfg) => cfg.clone(),
            Err(e) => return Err(Box::new(e)),
        };

        // Get the constant propagation results
        let const_prop = match ctx.get_result::<ConstantPropagationAnalysis>() {
            Ok(const_prop) => const_prop.clone(),
            Err(e) => return Err(Box::new(e)),
        };

        // Create a mutable copy of the CFG for optimization
        let mut cfg_copy = (*cfg).clone();

        // Optimize the control flow graph
        let optimizer = ControlFlowGraphOptimizer::new(ctx.body(), &const_prop);
        optimizer.optimize(&mut cfg_copy);

        // Wrap the optimized CFG in Arc<Mutex<>> for thread-safe access
        let optimized_cfg = Arc::new(Mutex::new(cfg_copy));

        Ok(OptimizedControlFlowGraph {
            cfg: optimized_cfg,
            optimized_edges: const_prop.optimized_edges.clone(),
        })
    }
}

/// The result of control flow optimization
#[derive(Debug, Clone)]
pub struct OptimizedControlFlowGraph {
    /// The optimized control flow graph
    pub cfg: Arc<Mutex<ControlFlowGraph>>,
    /// Map from instruction IDs to branch taken information for conditional jumps
    pub optimized_edges: HashMap<LocalDefId, BranchTaken>,
}

/// Optimizer for control flow graphs
struct ControlFlowGraphOptimizer<'a> {
    /// The constant propagation results
    const_prop: &'a ConstantPropagationResult,
}

impl<'a> ControlFlowGraphOptimizer<'a> {
    /// Create a new control flow graph optimizer
    fn new(_body: &'a Body, const_prop: &'a ConstantPropagationResult) -> Self {
        Self { const_prop }
    }

    /// Optimize the control flow graph
    fn optimize(&self, cfg: &mut ControlFlowGraph) {
        // Use the mutable reference directly

        // Optimize conditional jumps based on constant propagation results
        for (instr_id, branch_taken) in &self.const_prop.optimized_edges {
            // Get the node index for this instruction
            if let Some(node_idx) = cfg.get_node_by_instruction(*instr_id) {
                // Get the outgoing edges
                let edges: Vec<_> =
                    cfg.graph().edges_directed(node_idx, petgraph::Direction::Outgoing).collect();

                // Find the conditional edges
                let mut true_edge = None;
                let mut false_edge = None;

                for edge in edges {
                    match edge.weight() {
                        EdgeKind::ConditionalTrue => true_edge = Some(edge.id()),
                        EdgeKind::ConditionalFalse => false_edge = Some(edge.id()),
                        _ => {}
                    }
                }

                // Remove edges based on the branch taken
                match branch_taken {
                    BranchTaken::Always => {
                        // Condition is always true, remove the false edge
                        if let Some(edge_id) = false_edge {
                            cfg.graph_mut().remove_edge(edge_id);
                        }
                    }
                    BranchTaken::Never => {
                        // Condition is always false, remove the true edge
                        if let Some(edge_id) = true_edge {
                            cfg.graph_mut().remove_edge(edge_id);
                        }
                    }
                }
            }
        }
    }
}
