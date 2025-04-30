//! Control flow analysis for HIR
//!
//! This module provides control flow analysis for HIR bodies.
//! It builds a control flow graph (CFG) from a HIR body and provides
//! methods for analyzing the control flow.

use std::any::TypeId;
use std::collections::{HashMap, HashSet};

use hir::body::Body;
use hir::ids::LocalDefId;
use miette::Diagnostic;

use crate::context::AnalysisContext;
use crate::pass::AnalysisPass;

mod graph;

pub use graph::{BasicBlock, ControlFlowGraph, EdgeKind, Node};

/// Control flow analysis pass
///
/// This pass analyzes the control flow of a HIR body and builds a control flow graph.
/// It also detects unreachable code and other control flow issues.
#[derive(Default)]
pub struct ControlFlowAnalysis;

impl AnalysisPass for ControlFlowAnalysis {
    type Output = ControlFlowGraph;

    fn name(&self) -> &'static str {
        "ControlFlowAnalysis"
    }

    fn dependencies(&self) -> Vec<TypeId> {
        vec![]
    }

    fn run(&self, ctx: &mut AnalysisContext) -> Result<Self::Output, Box<dyn Diagnostic>> {
        // Clone the body to avoid borrowing issues
        let body = ctx.body().clone();
        let mut cfg_builder = ControlFlowGraphBuilder::new(&body);
        let cfg = cfg_builder.build();

        // Check for unreachable code
        let unreachable = cfg.find_unreachable_nodes();
        for node_id in unreachable {
            if let Some(instr_id) = cfg.get_node(node_id).instruction_id {
                if let Some(instr) = body.instructions.iter().find(|i| i.id == instr_id) {
                    ctx.warning_at_instruction(
                        format!("Unreachable instruction: {}", instr.opcode),
                        "This instruction will never be executed".to_string(),
                        instr_id,
                    );
                }
            }
        }

        // Check for infinite loops
        let infinite_loops = cfg.find_infinite_loops();
        for loop_nodes in infinite_loops {
            let loop_instrs: Vec<_> = loop_nodes
                .iter()
                .filter_map(|&node_id| {
                    let node = cfg.get_node(node_id);
                    node.instruction_id
                })
                .collect();

            if !loop_instrs.is_empty() {
                // Use the first instruction in the loop for the warning
                let first_instr_id = loop_instrs[0];
                ctx.warning_at_instruction(
                    "Potential infinite loop detected".to_string(),
                    "This loop may not terminate".to_string(),
                    first_instr_id,
                );
            }
        }

        Ok(cfg)
    }
}

/// Builder for control flow graphs
struct ControlFlowGraphBuilder<'a> {
    /// The HIR body being analyzed
    body: &'a Body,
    /// The control flow graph being built
    cfg: ControlFlowGraph,
    /// Map from instruction IDs to node indices
    instr_to_node: HashMap<LocalDefId, petgraph::graph::NodeIndex>,
    /// Map from label names to instruction IDs
    label_to_instr: HashMap<String, LocalDefId>,
}

impl<'a> ControlFlowGraphBuilder<'a> {
    /// Create a new control flow graph builder
    fn new(body: &'a Body) -> Self {
        let mut label_to_instr = HashMap::new();

        // Build a map from label names to instruction IDs
        for label in &body.labels {
            if let Some(instr_id) = label.instruction_id {
                label_to_instr.insert(label.name.clone(), instr_id);
            }
        }

        Self { body, cfg: ControlFlowGraph::new(), instr_to_node: HashMap::new(), label_to_instr }
    }

    /// Build the control flow graph
    fn build(&mut self) -> ControlFlowGraph {
        // Create nodes for all instructions
        for instr in &self.body.instructions {
            let node_id = self.cfg.add_node(Node::new(Some(instr.id)));
            self.instr_to_node.insert(instr.id, node_id);
        }

        // Create edges between nodes
        for (i, instr) in self.body.instructions.iter().enumerate() {
            let node_id = self.instr_to_node[&instr.id];

            // Check if this is a jump instruction
            let is_jump = match instr.opcode.to_uppercase().as_str() {
                "JUMP" | "JMP" | "JGTZ" | "JZERO" => true,
                _ => false,
            };

            // Check if this is a halt instruction
            let is_halt = instr.opcode.to_uppercase() == "HALT";

            if is_jump {
                // Add a conditional edge to the jump target
                if let Some(operand_id) = instr.operand {
                    if let Some(expr) = self.body.exprs.get(operand_id.0 as usize) {
                        match &expr.kind {
                            hir::body::ExprKind::Literal(hir::body::Literal::Label(label_name)) => {
                                if let Some(&target_instr_id) = self.label_to_instr.get(label_name)
                                {
                                    if let Some(&target_node_id) =
                                        self.instr_to_node.get(&target_instr_id)
                                    {
                                        // Add a jump edge
                                        let edge_kind = match instr.opcode.to_uppercase().as_str() {
                                            "JUMP" | "JMP" => EdgeKind::Unconditional,
                                            "JGTZ" => EdgeKind::ConditionalTrue,
                                            "JZERO" => EdgeKind::ConditionalTrue,
                                            _ => EdgeKind::Unconditional,
                                        };

                                        self.cfg.add_edge(node_id, target_node_id, edge_kind);

                                        // For conditional jumps, also add a fallthrough edge
                                        if instr.opcode.to_uppercase() == "JGTZ"
                                            || instr.opcode.to_uppercase() == "JZERO"
                                        {
                                            if i + 1 < self.body.instructions.len() {
                                                let next_instr_id =
                                                    self.body.instructions[i + 1].id;
                                                let next_node_id =
                                                    self.instr_to_node[&next_instr_id];
                                                self.cfg.add_edge(
                                                    node_id,
                                                    next_node_id,
                                                    EdgeKind::ConditionalFalse,
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {
                                // Non-label operand, can't determine jump target statically
                            }
                        }
                    }
                }
            } else if !is_halt {
                // Add a fallthrough edge to the next instruction
                if i + 1 < self.body.instructions.len() {
                    let next_instr_id = self.body.instructions[i + 1].id;
                    let next_node_id = self.instr_to_node[&next_instr_id];
                    self.cfg.add_edge(node_id, next_node_id, EdgeKind::Unconditional);
                }
            }
        }

        // Identify basic blocks
        self.identify_basic_blocks();

        self.cfg.clone()
    }

    /// Identify basic blocks in the control flow graph
    fn identify_basic_blocks(&mut self) {
        let mut current_block = Vec::new();
        let mut leaders: HashSet<LocalDefId> = HashSet::new();

        // The first instruction is always a leader
        if let Some(first_instr) = self.body.instructions.first() {
            leaders.insert(first_instr.id);
        }

        // Instructions that are targets of jumps are leaders
        for instr in &self.body.instructions {
            let node_id = self.instr_to_node[&instr.id];

            // If this node has incoming edges, it's a leader
            if !self.cfg.get_incoming_edges(node_id).is_empty() {
                leaders.insert(instr.id);
            }

            // If this node has multiple outgoing edges, the next instruction is a leader
            if self.cfg.get_outgoing_edges(node_id).len() > 1 {
                // Find the next instruction
                for (i, instr2) in self.body.instructions.iter().enumerate() {
                    if instr2.id == instr.id && i + 1 < self.body.instructions.len() {
                        leaders.insert(self.body.instructions[i + 1].id);
                        break;
                    }
                }
            }
        }

        // Create basic blocks
        for instr in &self.body.instructions {
            let node_id = self.instr_to_node[&instr.id];

            if leaders.contains(&instr.id) {
                // Start a new basic block
                if !current_block.is_empty() {
                    // Finish the current block
                    let block = BasicBlock::new(current_block.clone());
                    self.cfg.add_basic_block(block);

                    // Clear the current block
                    current_block.clear();
                }

                // Add the leader to the new block
                current_block.push(node_id);
            } else {
                // Add the instruction to the current block
                current_block.push(node_id);
            }
        }

        // Finish the last block
        if !current_block.is_empty() {
            let block = BasicBlock::new(current_block);
            self.cfg.add_basic_block(block);
        }
    }
}
