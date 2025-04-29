//! Optimization analysis for HIR
//!
//! This module provides optimization analysis for HIR.

use std::sync::Arc;

use hir::ids::{DefId, LocalDefId};
use miette::Severity;

use crate::AnalysisContext;
use crate::analysis::AnalysisPass;
use crate::analysis::control_flow::ControlFlowGraph;
use crate::analysis::data_flow::DataFlowResults;

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

    // Get the control flow graph from the context
    let cfg = ctx.db().control_flow_graph(ctx.body().owner);

    // Find unreachable code
    find_unreachable_code(ctx, &cfg, &mut optimizations);

    // Find dead stores
    let data_flow = ctx.db().data_flow_results(ctx.body().owner);
    find_dead_stores(ctx, &cfg, &data_flow, &mut optimizations);

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
            // In a real implementation, we would check if the variable is live after this instruction
            // For now, we'll just add a placeholder optimization for some instructions
            if instr_id.0 % 3 == 0 {
                // Just a simple heuristic for demonstration
                optimizations.push(Optimization::DeadStore {
                    instr_id,
                    description: format!("Dead store at instruction {}", instr_id.0),
                });
            }
        }
    }
}

/// Query function for finding optimization opportunities in a body
pub(crate) fn optimization_query(
    db: &dyn crate::AnalysisDatabase,
    def_id: DefId,
) -> Arc<Vec<Optimization>> {
    let body = db.body(def_id);
    let mut context = crate::AnalysisContext::new(db, &body);

    // Set up the context with control flow and data flow information
    let cfg = db.control_flow_graph(def_id);
    let data_flow = db.data_flow_results(def_id);

    // Store the results in the type info
    context.type_info_mut().set_type::<Arc<ControlFlowGraph>>(cfg.clone());
    context.type_info_mut().set_type::<Arc<DataFlowResults>>(data_flow.clone());

    // Find optimizations
    let optimizations = find_optimizations(&context);
    Arc::new(optimizations)
}

/// Optimization analysis pass
///
/// This pass identifies optimization opportunities in the code.
/// It depends on the control flow analysis and data flow analysis passes
/// and should run after them.
#[derive(Debug, Default, Clone)]
pub struct OptimizationAnalysis;

impl OptimizationAnalysis {
    /// Create a new optimization analysis pass
    pub fn new() -> Self {
        Self
    }
}

impl AnalysisPass for OptimizationAnalysis {
    type Output = Vec<Optimization>;

    fn run<'db, 'body>(
        &self,
        ctx: &mut AnalysisContext<'db, 'body>,
        dependencies: &crate::AnalysisResults,
    ) -> Self::Output {
        // Get the control flow graph and data flow results from dependencies
        let cfg =
            self.get_dependency::<crate::analysis::control_flow::ControlFlowAnalysis>(dependencies);
        let data_flow =
            self.get_dependency::<crate::analysis::data_flow::DataFlowAnalysis>(dependencies);

        // Make sure we have the required dependencies
        if cfg.is_none() {
            ctx.diagnostics_mut().error(
                "Cannot run optimization analysis without control flow graph".to_string(),
                None,
                Severity::Error,
            );
        }

        if data_flow.is_none() {
            ctx.diagnostics_mut().error(
                "Cannot run optimization analysis without data flow results".to_string(),
                None,
                Severity::Error,
            );
        }

        // Run the optimization analysis and return the result
        find_optimizations(ctx)
    }

    fn description(&self) -> &'static str {
        "Identifies optimization opportunities in the code"
    }

    fn priority(&self) -> u32 {
        // Optimization analysis should run after data flow analysis
        30
    }

    fn dependencies(&self) -> Vec<std::any::TypeId> {
        // This pass depends on the control flow analysis and data flow analysis
        vec![
            std::any::TypeId::of::<crate::analysis::control_flow::ControlFlowAnalysis>(),
            std::any::TypeId::of::<crate::analysis::data_flow::DataFlowAnalysis>(),
        ]
    }
}
