//! Constant propagation analysis for HIR
//!
//! This module provides constant propagation analysis for HIR bodies.
//! It analyzes the program to determine which values are constant and
//! can be determined at compile time.

use std::any::TypeId;
use std::collections::HashMap;

use hir::body::{AddressingMode, Body, ExprKind, Instruction, Literal};
use hir::ids::LocalDefId;
use miette::Diagnostic;

use crate::analyzers::control_flow::{ControlFlowAnalysis, ControlFlowGraph, EdgeKind};
use crate::analyzers::data_flow::{DataFlowAnalysis, DataFlowGraph};
use crate::context::AnalysisContext;
use crate::pass::AnalysisPass;

/// Constant propagation analysis pass
///
/// This pass analyzes the program to determine which values are constant
/// and can be determined at compile time. It uses this information to
/// optimize the control flow graph by removing branches that will never
/// be taken.
#[derive(Default)]
pub struct ConstantPropagationAnalysis;

impl AnalysisPass for ConstantPropagationAnalysis {
    type Output = ConstantPropagationResult;

    fn name(&self) -> &'static str {
        "ConstantPropagationAnalysis"
    }

    fn dependencies(&self) -> Vec<TypeId> {
        vec![TypeId::of::<ControlFlowAnalysis>(), TypeId::of::<DataFlowAnalysis>()]
    }

    fn run(&self, ctx: &mut AnalysisContext) -> Result<Self::Output, Box<dyn Diagnostic>> {
        // Get the control flow graph
        let cfg = match ctx.get_result::<ControlFlowAnalysis>() {
            Ok(cfg) => cfg.clone(),
            Err(e) => return Err(Box::new(e)),
        };

        // Get the data flow graph
        let dfg = match ctx.get_result::<DataFlowAnalysis>() {
            Ok(dfg) => dfg.clone(),
            Err(e) => return Err(Box::new(e)),
        };

        // Clone the body to avoid borrowing issues
        let body = ctx.body().clone();

        // Analyze constant values
        let mut analyzer = ConstantPropagationAnalyzer::new(&body, &cfg, &dfg);
        let result = analyzer.analyze();

        // Analyze the control flow graph to find branches that can be optimized
        let optimized_edges = analyzer.analyze_conditional_branches();

        // Report optimizations only for branches that can be statically determined
        let mut diagnostics = Vec::new();
        for (instr_id, branch_taken) in &optimized_edges {
            if let Some(instr) = body.instructions.iter().find(|i| i.id == *instr_id) {
                let branch_str = match branch_taken {
                    BranchTaken::Always => "always",
                    BranchTaken::Never => "never",
                };

                diagnostics.push((
                    *instr_id,
                    format!("Conditional jump {} taken", branch_str),
                    format!(
                        "The condition for this {} instruction is statically known",
                        instr.opcode
                    ),
                ));
            }
        }

        // Add diagnostics after we're done with all the borrowing
        for (instr_id, message, details) in diagnostics {
            ctx.info_at_instruction(message, details, instr_id);
        }

        Ok(ConstantPropagationResult { constant_values: result, optimized_edges })
    }
}

/// The result of constant propagation analysis
#[derive(Debug, Clone)]
pub struct ConstantPropagationResult {
    /// Map from instruction IDs to constant accumulator values after the instruction
    pub constant_values: HashMap<LocalDefId, Option<i64>>,
    /// Map from instruction IDs to branch taken information for conditional jumps
    pub optimized_edges: HashMap<LocalDefId, BranchTaken>,
}

/// Indicates whether a branch is always taken or never taken
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BranchTaken {
    /// The branch is always taken
    Always,
    /// The branch is never taken
    Never,
}

/// Analyzer for constant propagation
struct ConstantPropagationAnalyzer<'a> {
    /// The HIR body being analyzed
    body: &'a Body,
    /// The control flow graph
    cfg: &'a ControlFlowGraph,

    /// Map from instruction IDs to constant accumulator values after the instruction
    constant_values: HashMap<LocalDefId, Option<i64>>,
}

impl<'a> ConstantPropagationAnalyzer<'a> {
    /// Create a new constant propagation analyzer
    fn new(body: &'a Body, cfg: &'a ControlFlowGraph, _dfg: &'a DataFlowGraph) -> Self {
        Self { body, cfg, constant_values: HashMap::new() }
    }

    /// Analyze the program to determine constant values
    fn analyze(&mut self) -> HashMap<LocalDefId, Option<i64>> {
        // Initialize all instructions with unknown values
        for instr in &self.body.instructions {
            self.constant_values.insert(instr.id, None);
        }

        // Perform a topological sort of the CFG to process instructions in order
        if let Ok(sorted_nodes) = self.cfg.topological_sort() {
            // Process instructions in topological order
            for node_idx in sorted_nodes {
                if let Some(instr_id) = self.cfg.get_node(node_idx).instruction_id {
                    if let Some(instr) = self.body.instructions.iter().find(|i| i.id == instr_id) {
                        self.process_instruction(instr);
                    }
                }
            }
        } else {
            // If there's a cycle, process instructions in any order
            // This is less precise but still safe
            for instr in &self.body.instructions {
                self.process_instruction(instr);
            }
        }

        self.constant_values.clone()
    }

    /// Process an instruction to determine its effect on constant values
    fn process_instruction(&mut self, instr: &Instruction) {
        // Get the current accumulator value (if known)
        let acc_value = self.get_accumulator_value_before(instr.id);

        // Update the accumulator value based on the instruction
        let new_acc_value = match instr.opcode.to_uppercase().as_str() {
            "LOAD" => {
                // LOAD sets the accumulator to the value at the memory address
                if let Some(operand_id) = instr.operand {
                    // Only consider immediate values (like =10) as constants
                    // All other memory references (including array accesses) are not statically known
                    self.get_constant_operand_value(operand_id)
                } else {
                    None
                }
            }
            "STORE" => {
                // STORE doesn't change the accumulator
                acc_value
            }
            "ADD" => {
                // ADD adds the operand to the accumulator
                if let (Some(acc), Some(operand_id)) = (acc_value, instr.operand) {
                    self.get_constant_operand_value(operand_id)
                        .map(|operand_value| acc + operand_value)
                } else {
                    None
                }
            }
            "SUB" => {
                // SUB subtracts the operand from the accumulator
                if let (Some(acc), Some(operand_id)) = (acc_value, instr.operand) {
                    self.get_constant_operand_value(operand_id)
                        .map(|operand_value| acc - operand_value)
                } else {
                    None
                }
            }
            "MUL" => {
                // MUL multiplies the accumulator by the operand
                if let (Some(acc), Some(operand_id)) = (acc_value, instr.operand) {
                    self.get_constant_operand_value(operand_id)
                        .map(|operand_value| acc * operand_value)
                } else {
                    None
                }
            }
            "DIV" => {
                // DIV divides the accumulator by the operand
                if let (Some(acc), Some(operand_id)) = (acc_value, instr.operand) {
                    if let Some(operand_value) = self.get_constant_operand_value(operand_id) {
                        if operand_value != 0 {
                            Some(acc / operand_value)
                        } else {
                            // Division by zero is undefined
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            "JUMP" | "JMP" | "JGTZ" | "JZERO" => {
                // Jump instructions don't change the accumulator
                acc_value
            }
            "HALT" => {
                // HALT doesn't change the accumulator
                acc_value
            }
            "READ" => {
                // READ gets input from the user, so the value is unknown
                None
            }
            "WRITE" => {
                // WRITE doesn't change the accumulator
                acc_value
            }
            _ => {
                // Unknown instruction, assume it makes the accumulator unknown
                None
            }
        };

        // Update the constant value map
        self.constant_values.insert(instr.id, new_acc_value);
    }

    /// Get the constant value of an operand, if known
    fn get_constant_operand_value(&self, operand_id: hir::expr::ExprId) -> Option<i64> {
        if let Some(expr) = self.body.exprs.get(operand_id.0 as usize) {
            match &expr.kind {
                ExprKind::Literal(Literal::Int(value)) => Some(*value),
                ExprKind::MemoryRef(mem_ref) => {
                    // Memory references (including array accesses) are not statically known
                    // unless they are direct literals with a constant address and immediate mode
                    if let AddressingMode::Immediate = mem_ref.mode {
                        // For immediate addressing (e.g., =5), we can use the literal value
                        if let Some(addr_expr) = self.body.exprs.get(mem_ref.address.0 as usize) {
                            if let ExprKind::Literal(Literal::Int(value)) = &addr_expr.kind {
                                return Some(*value);
                            }
                        }
                    }
                    // For all other memory references, the value is not statically known
                    None
                }
                ExprKind::ArrayAccess(_) => {
                    // Array accesses are not statically known
                    // They require runtime evaluation
                    None
                }
                _ => None,
            }
        } else {
            None
        }
    }

    /// Get the accumulator value before an instruction, if known
    fn get_accumulator_value_before(&self, instr_id: LocalDefId) -> Option<i64> {
        // Find the instruction's node in the CFG
        let node_idx = self.cfg.get_node_by_instruction(instr_id)?;

        // Get the predecessors of this node
        let predecessors = self.cfg.get_predecessors(node_idx);
        if predecessors.is_empty() {
            // If there are no predecessors, this is the entry point
            // The accumulator starts at 0
            return Some(0);
        }

        // Check if all predecessors have the same constant accumulator value
        let mut pred_values = Vec::new();
        for pred_idx in predecessors {
            if let Some(pred_instr_id) = self.cfg.get_node(pred_idx).instruction_id {
                if let Some(value) = self.constant_values.get(&pred_instr_id).copied().flatten() {
                    pred_values.push(value);
                } else {
                    // If any predecessor has an unknown value, the result is unknown
                    return None;
                }
            }
        }

        // If all predecessors have the same value, return that value
        if !pred_values.is_empty() && pred_values.iter().all(|&v| v == pred_values[0]) {
            Some(pred_values[0])
        } else {
            // If predecessors have different values, the result is unknown
            None
        }
    }

    /// Analyze the control flow graph to find branches that can be optimized
    /// Returns a map of instruction IDs to branch taken information
    fn analyze_conditional_branches(&self) -> HashMap<LocalDefId, BranchTaken> {
        let mut optimized_edges = HashMap::new();

        // Find conditional jumps with constant accumulator values
        for instr in &self.body.instructions {
            let opcode = instr.opcode.to_uppercase();

            // Check if this is a conditional jump
            if opcode == "JGTZ" || opcode == "JZERO" {
                // Get the accumulator value before this instruction
                if let Some(acc_value) = self.get_accumulator_value_before(instr.id) {
                    // Determine if the condition is always true or always false
                    let condition_true = match opcode.as_str() {
                        "JGTZ" => acc_value > 0,
                        "JZERO" => acc_value == 0,
                        _ => false,
                    };

                    // Get the node index for this instruction
                    if let Some(node_idx) = self.cfg.get_node_by_instruction(instr.id) {
                        // Get the outgoing edges
                        let edges: Vec<_> = self
                            .cfg
                            .graph()
                            .edges_directed(node_idx, petgraph::Direction::Outgoing)
                            .collect();

                        // Find the conditional edges
                        let mut has_true_edge = false;
                        let mut has_false_edge = false;

                        for edge in edges {
                            match edge.weight() {
                                EdgeKind::ConditionalTrue => has_true_edge = true,
                                EdgeKind::ConditionalFalse => has_false_edge = true,
                                _ => {}
                            }
                        }

                        // Record which branches will always be taken or never taken
                        if condition_true && has_false_edge {
                            // Condition is always true, false branch will never be taken
                            optimized_edges.insert(instr.id, BranchTaken::Always);
                        } else if !condition_true && has_true_edge {
                            // Condition is always false, true branch will never be taken
                            optimized_edges.insert(instr.id, BranchTaken::Never);
                        }
                    }
                }
                // If the accumulator value is not statically known, we can't optimize this branch
                // So we don't add anything to optimized_edges
            }
        }

        optimized_edges
    }
}
