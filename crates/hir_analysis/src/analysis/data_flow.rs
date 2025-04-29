//! Data flow analysis for HIR
//!
//! This module provides data flow analysis for HIR.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use hir::expr::ExprId;
use hir::body::{Body, Instruction};
use hir::ids::{DefId, LocalDefId};

use crate::AnalysisContext;
use crate::analysis::AnalysisPass;
use crate::analysis::control_flow::ControlFlowGraph;

/// A variable in the data flow analysis
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Variable {
    /// A register
    Register(u32),

    /// A memory location
    Memory(u32),
}

/// Data flow information for a basic block
#[derive(Debug, Clone)]
pub struct BlockDataFlow {
    /// Variables defined in this block
    pub def: HashSet<Variable>,

    /// Variables used in this block
    pub use_: HashSet<Variable>,

    /// Variables live at the entry of this block
    pub live_in: HashSet<Variable>,

    /// Variables live at the exit of this block
    pub live_out: HashSet<Variable>,
}

/// Results of data flow analysis
#[derive(Debug, Clone)]
pub struct DataFlowResults {
    /// Data flow information for each block
    pub block_data_flow: HashMap<usize, BlockDataFlow>,

    /// Variables defined at each instruction
    pub def_at: HashMap<LocalDefId, HashSet<Variable>>,

    /// Variables used at each instruction
    pub use_at: HashMap<LocalDefId, HashSet<Variable>>,

    /// Variables live before each instruction
    pub live_before: HashMap<LocalDefId, HashSet<Variable>>,

    /// Variables live after each instruction
    pub live_after: HashMap<LocalDefId, HashSet<Variable>>,
}

impl DataFlowResults {
    /// Create a new empty data flow results
    pub fn new() -> Self {
        Self {
            block_data_flow: HashMap::new(),
            def_at: HashMap::new(),
            use_at: HashMap::new(),
            live_before: HashMap::new(),
            live_after: HashMap::new(),
        }
    }

    /// Check if a variable is defined at an instruction
    pub fn is_defined_at(&self, instruction_id: LocalDefId, var: &Variable) -> bool {
        self.def_at.get(&instruction_id).map_or(false, |vars| vars.contains(var))
    }

    /// Check if a variable is used at an instruction
    pub fn is_used_at(&self, instruction_id: LocalDefId, var: &Variable) -> bool {
        self.use_at.get(&instruction_id).map_or(false, |vars| vars.contains(var))
    }

    /// Check if a variable is live before an instruction
    pub fn is_live_before(&self, instruction_id: LocalDefId, var: &Variable) -> bool {
        self.live_before.get(&instruction_id).map_or(false, |vars| vars.contains(var))
    }

    /// Check if a variable is live after an instruction
    pub fn is_live_after(&self, instruction_id: LocalDefId, var: &Variable) -> bool {
        self.live_after.get(&instruction_id).map_or(false, |vars| vars.contains(var))
    }
}

/// Analyze the data flow of a body
pub fn analyze(ctx: &mut crate::AnalysisContext, cfg: &ControlFlowGraph) -> Arc<DataFlowResults> {
    let mut results = DataFlowResults::new();

    // Initialize block data flow
    for block in &cfg.blocks {
        results.block_data_flow.insert(
            block.id,
            BlockDataFlow {
                def: HashSet::new(),
                use_: HashSet::new(),
                live_in: HashSet::new(),
                live_out: HashSet::new(),
            },
        );
    }

    // Compute def and use sets for each instruction
    for block in &cfg.blocks {
        let mut block_def = HashSet::new();
        let mut block_use = HashSet::new();

        for &instr_id in &block.instructions {
            let body = ctx.body();
            if let Some(instr) = body.instruction(instr_id) {
                let (def, use_) = compute_def_use(body, instr);

                // Add to instruction-level maps
                results.def_at.insert(instr_id, def.clone());
                results.use_at.insert(instr_id, use_.clone());

                // Add to block-level sets
                block_def.extend(def);
                block_use.extend(use_);
            }
        }

        // Update block data flow
        if let Some(block_data_flow) = results.block_data_flow.get_mut(&block.id) {
            block_data_flow.def = block_def;
            block_data_flow.use_ = block_use;
        }
    }

    // Compute live-in and live-out sets for each block
    compute_liveness(cfg, &mut results);

    // Compute live-before and live-after sets for each instruction
    compute_instruction_liveness(ctx.body(), cfg, &mut results);

    Arc::new(results)
}

/// Compute the def and use sets for an instruction
fn compute_def_use(body: &Body, instr: &Instruction) -> (HashSet<Variable>, HashSet<Variable>) {
    let mut def = HashSet::new();
    let mut use_ = HashSet::new();

    match instr.opcode.as_str() {
        "LOAD" => {
            // LOAD r, x: def = {r}, use = {x}
            def.insert(Variable::Register(1)); // Assuming r is register 1

            if let Some(operand) = instr.operand {
                if let Some(expr) = body.expr(operand) {
                    // In a real implementation, we would extract the variable from the expression
                    // For now, we'll just use a placeholder
                    use_.insert(Variable::Memory(1));
                }
            }
        }
        "STORE" => {
            // STORE x, r: def = {x}, use = {r}
            if let Some(operand) = instr.operand {
                if let Some(expr) = body.expr(operand) {
                    // In a real implementation, we would extract the variable from the expression
                    // For now, we'll just use a placeholder
                    def.insert(Variable::Memory(1));
                }
            }

            use_.insert(Variable::Register(1)); // Assuming r is register 1
        }
        "ADD" | "SUB" | "MUL" | "DIV" => {
            // ADD r, x: def = {r}, use = {r, x}
            def.insert(Variable::Register(1)); // Assuming r is register 1
            use_.insert(Variable::Register(1)); // Assuming r is register 1

            if let Some(operand) = instr.operand {
                if let Some(expr) = body.expr(operand) {
                    // In a real implementation, we would extract the variable from the expression
                    // For now, we'll just use a placeholder
                    use_.insert(Variable::Memory(1));
                }
            }
        }
        "READ" => {
            // READ x: def = {x}
            if let Some(operand) = instr.operand {
                if let Some(expr) = body.expr(operand) {
                    // In a real implementation, we would extract the variable from the expression
                    // For now, we'll just use a placeholder
                    def.insert(Variable::Memory(1));
                }
            }
        }
        "WRITE" => {
            // WRITE x: use = {x}
            if let Some(operand) = instr.operand {
                if let Some(expr) = body.expr(operand) {
                    // In a real implementation, we would extract the variable from the expression
                    // For now, we'll just use a placeholder
                    use_.insert(Variable::Memory(1));
                }
            }
        }
        "JUMP" => {
            // JUMP label: no def/use
        }
        "JZERO" | "JGTZ" => {
            // JZERO label, r: use = {r}
            use_.insert(Variable::Register(1)); // Assuming r is register 1
        }
        "HALT" => {
            // HALT: no def/use
        }
        _ => {
            // Unknown instruction
        }
    }

    (def, use_)
}

/// Compute liveness information for each block
fn compute_liveness(cfg: &ControlFlowGraph, results: &mut DataFlowResults) {
    let mut changed = true;

    while changed {
        changed = false;

        for block in &cfg.blocks {
            if let Some(block_data_flow) = results.block_data_flow.get_mut(&block.id) {
                // Compute live-out as the union of live-in of all successors
                let mut new_live_out = HashSet::new();
                for &succ_id in &block.successors {
                    if let Some(succ_data_flow) = results.block_data_flow.get(&succ_id) {
                        new_live_out.extend(succ_data_flow.live_in.iter().cloned());
                    }
                }

                // Check if live-out changed
                if new_live_out != block_data_flow.live_out {
                    changed = true;
                    block_data_flow.live_out = new_live_out;
                }

                // Compute live-in as use ∪ (live-out - def)
                let mut new_live_in = block_data_flow.use_.clone();
                for var in &block_data_flow.live_out {
                    if !block_data_flow.def.contains(var) {
                        new_live_in.insert(var.clone());
                    }
                }

                // Check if live-in changed
                if new_live_in != block_data_flow.live_in {
                    changed = true;
                    block_data_flow.live_in = new_live_in;
                }
            }
        }
    }
}

/// Compute liveness information for each instruction
fn compute_instruction_liveness(
    body: &Body,
    cfg: &ControlFlowGraph,
    results: &mut DataFlowResults,
) {
    for block in &cfg.blocks {
        if let Some(block_data_flow) = results.block_data_flow.get(&block.id) {
            let mut live = block_data_flow.live_out.clone();

            // Process instructions in reverse order
            for &instr_id in block.instructions.iter().rev() {
                // live-after = live
                results.live_after.insert(instr_id, live.clone());

                // live = (live - def) ∪ use
                if let Some(def) = results.def_at.get(&instr_id) {
                    for var in def {
                        live.remove(var);
                    }
                }

                if let Some(use_) = results.use_at.get(&instr_id) {
                    for var in use_ {
                        live.insert(var.clone());
                    }
                }

                // live-before = live
                results.live_before.insert(instr_id, live.clone());
            }
        }
    }
}

/// Query function for getting the data flow results of a body
pub(crate) fn data_flow_query(
    db: &dyn crate::AnalysisDatabase,
    def_id: DefId,
) -> Arc<DataFlowResults> {
    let body = db.body(def_id);
    let mut context = crate::AnalysisContext::new(db, &body);

    let cfg = db.control_flow_graph(def_id);
    analyze(&mut context, &cfg)
}

/// Data flow analysis pass
///
/// This pass analyzes the data flow of a body and builds a data flow graph.
/// It depends on the control flow analysis pass and should run after it.
#[derive(Debug, Default, Clone)]
pub struct DataFlowAnalysis;

impl DataFlowAnalysis {
    /// Create a new data flow analysis pass
    pub fn new() -> Self {
        Self
    }
}

impl AnalysisPass for DataFlowAnalysis {
    type Output = Arc<DataFlowResults>;

    fn run<'db, 'body>(
        &self,
        ctx: &mut AnalysisContext<'db, 'body>,
        dependencies: &crate::AnalysisResults,
    ) -> Self::Output {
        // Get the control flow graph from dependencies
        if let Some(cfg) =
            self.get_dependency::<crate::analysis::control_flow::ControlFlowAnalysis>(dependencies)
        {
            // Run the data flow analysis and return the result
            analyze(ctx, cfg)
        } else {
            // This should never happen if dependencies are properly resolved
            ctx.diagnostics_mut().error(
                "Cannot run data flow analysis without control flow graph".to_string(),
                None,
            );

            // Return an empty result
            Arc::new(DataFlowResults::new())
        }
    }

    fn description(&self) -> &'static str {
        "Analyzes data flow through the program"
    }

    fn priority(&self) -> u32 {
        // Data flow analysis should run after control flow analysis
        20
    }

    fn dependencies(&self) -> Vec<std::any::TypeId> {
        // This pass depends on the control flow analysis
        vec![std::any::TypeId::of::<crate::analysis::control_flow::ControlFlowAnalysis>()]
    }
}
