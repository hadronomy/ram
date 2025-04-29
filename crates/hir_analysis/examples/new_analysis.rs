//! Example of using the new analysis system
//!
//! This example demonstrates how to use the new analysis system to analyze a body.

use std::sync::Arc;

use hir::body::Body;
use hir::ids::DefId;
use hir_analysis::analysis::control_flow::ControlFlowAnalysis;
use hir_analysis::analysis::data_flow::DataFlowAnalysis;
use hir_analysis::analysis::manager::AnalysisManager;
use hir_analysis::analysis::optimization::OptimizationAnalysis;
use hir_analysis::analysis::AnalysisPass;
use hir_analysis::AnalysisContext;
use hir_analysis::AnalysisDatabase;

/// A custom analysis pass that counts the number of instructions in a body
#[derive(Debug, Default, Clone)]
struct InstructionCountPass;

impl AnalysisPass for InstructionCountPass {
    type Output = usize;

    fn run<'db, 'body>(
        &self,
        ctx: &mut AnalysisContext<'db, 'body>,
        _dependencies: &hir_analysis::analysis::results::AnalysisResultsCache,
    ) -> Self::Output {
        // Count the number of instructions in the body
        ctx.body().instructions.len()
    }

    fn description(&self) -> &'static str {
        "Counts the number of instructions in a body"
    }

    fn priority(&self) -> u32 {
        // This pass can run at any time
        100
    }

    fn dependencies(&self) -> Vec<std::any::TypeId> {
        // This pass has no dependencies
        Vec::new()
    }
}

/// A custom analysis pass that calculates the average number of instructions per basic block
#[derive(Debug, Default, Clone)]
struct AverageInstructionsPerBlockPass;

impl AnalysisPass for AverageInstructionsPerBlockPass {
    type Output = f64;

    fn run<'db, 'body>(
        &self,
        ctx: &mut AnalysisContext<'db, 'body>,
        dependencies: &hir_analysis::analysis::results::AnalysisResultsCache,
    ) -> Self::Output {
        // Get the control flow graph from dependencies
        if let Some(cfg) = self.get_dependency::<ControlFlowAnalysis>(dependencies) {
            // Get the number of instructions from dependencies
            if let Some(instr_count) = self.get_dependency::<InstructionCountPass>(dependencies) {
                // Calculate the average
                let block_count = cfg.blocks.len();
                if block_count > 0 {
                    *instr_count as f64 / block_count as f64
                } else {
                    0.0
                }
            } else {
                // If we don't have the instruction count, calculate it directly
                let instr_count = ctx.body().instructions.len();
                let block_count = cfg.blocks.len();
                if block_count > 0 {
                    instr_count as f64 / block_count as f64
                } else {
                    0.0
                }
            }
        } else {
            // If we don't have the control flow graph, we can't calculate the average
            ctx.diagnostics_mut().error(
                "Cannot calculate average instructions per block without control flow graph"
                    .to_string(),
                None,
            );
            0.0
        }
    }

    fn description(&self) -> &'static str {
        "Calculates the average number of instructions per basic block"
    }

    fn priority(&self) -> u32 {
        // This pass should run after the control flow analysis and instruction count pass
        50
    }

    fn dependencies(&self) -> Vec<std::any::TypeId> {
        // This pass depends on the control flow analysis and instruction count pass
        vec![
            std::any::TypeId::of::<ControlFlowAnalysis>(),
            std::any::TypeId::of::<InstructionCountPass>(),
        ]
    }
}

/// Run the analysis on a body
fn analyze_body(db: &dyn AnalysisDatabase, def_id: DefId) {
    // Get the body from the database
    let body = db.body(def_id);

    // Create a context for the analysis
    let mut context = AnalysisContext::new(db, &body);

    // Create a manager for the analysis
    let mut manager = AnalysisManager::new();

    // Register the passes
    manager.register_pass::<ControlFlowAnalysis>();
    manager.register_pass::<DataFlowAnalysis>();
    manager.register_pass::<OptimizationAnalysis>();
    manager.register_pass::<InstructionCountPass>();
    manager.register_pass::<AverageInstructionsPerBlockPass>();

    // Run the analysis
    if let Err(err) = manager.run_all_passes(&mut context) {
        println!("Error running analysis: {}", err);
        return;
    }

    // Get the results
    let instr_count = manager.get_result::<usize>().unwrap_or(&0);
    let avg_instr_per_block = manager.get_result::<f64>().unwrap_or(&0.0);
    let cfg = manager.get_result::<Arc<hir_analysis::analysis::control_flow::ControlFlowGraph>>();
    let data_flow = manager.get_result::<Arc<hir_analysis::analysis::data_flow::DataFlowResults>>();
    let optimizations = manager.get_result::<Vec<hir_analysis::analysis::optimization::Optimization>>();

    // Print the results
    println!("Analysis results:");
    println!("  Instruction count: {}", instr_count);
    println!("  Average instructions per block: {:.2}", avg_instr_per_block);
    println!("  Control flow graph: {}", if cfg.is_some() { "available" } else { "not available" });
    println!("  Data flow results: {}", if data_flow.is_some() { "available" } else { "not available" });
    println!("  Optimization opportunities: {}", optimizations.map_or(0, |o| o.len()));

    // Print any diagnostics
    if context.diagnostics().has_errors() {
        println!("Errors:");
        for error in context.diagnostics().errors() {
            println!("  {}", error.message);
        }
    }

    if context.diagnostics().has_warnings() {
        println!("Warnings:");
        for warning in context.diagnostics().warnings() {
            println!("  {}", warning.message);
        }
    }
}

fn main() {
    println!("This is an example of using the new analysis system.");
    println!("It doesn't actually run any analysis, but demonstrates how to use the API.");
}
