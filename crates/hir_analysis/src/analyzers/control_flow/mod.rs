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
        let unreachable_nodes = cfg.find_unreachable_nodes();

        // Collect instruction indices for unreachable nodes
        let mut unreachable_indices: Vec<usize> = unreachable_nodes
            .iter()
            .filter_map(|&node_idx| {
                let node = cfg.get_node(node_idx);
                // Map the instruction ID back to its index in the body instructions vector
                // This assumes instructions are stored sequentially
                node.instruction_id.and_then(|id| body.instructions.iter().position(|i| i.id == id))
            })
            .collect();

        // Sort indices to group them by program order
        unreachable_indices.sort_unstable();

        // Group contiguous indices into ranges
        let mut unreachable_ranges: Vec<(usize, usize)> = Vec::new();

        for &idx in &unreachable_indices {
            match unreachable_ranges.last_mut() {
                // If the current index follows immediately after the end of the last range, extend it
                Some((_, end)) if *end + 1 == idx => {
                    *end = idx;
                }
                // Otherwise, start a new range
                _ => {
                    unreachable_ranges.push((idx, idx));
                }
            }
        }

        // Report one warning per contiguous block
        for (start_idx, end_idx) in unreachable_ranges {
            let start_instr = &body.instructions[start_idx];
            let end_instr = &body.instructions[end_idx];

            // Create a span that covers the entire block
            let full_span = start_instr.span.start..end_instr.span.end;

            ctx.warning(
                "Unreachable code",
                "This block of instructions will never be executed",
                Some(full_span),
            );
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

    /// Helper method to find a label by its DefId
    fn find_label_by_id(&self, label_id: hir::ids::DefId) -> Option<&hir::body::Label> {
        self.body.labels.iter().find(|l| {
            // We need to compare the file_id and local_id parts of DefId
            // since we can't directly compare DefId with LocalDefId
            label_id.file_id == self.body.owner.file_id && label_id.local_id.0 == l.id.0
        })
    }

    /// Helper method to add appropriate edges for a jump instruction
    fn add_jump_edges(
        &mut self,
        node_id: petgraph::graph::NodeIndex,
        target_node_id: petgraph::graph::NodeIndex,
        opcode: &str,
        instr_index: usize,
    ) {
        // Add a jump edge
        let edge_kind = match opcode.to_uppercase().as_str() {
            "JUMP" | "JMP" => EdgeKind::Unconditional,
            "JGTZ" => EdgeKind::ConditionalTrue,
            "JZERO" => EdgeKind::ConditionalTrue,
            _ => EdgeKind::Unconditional,
        };

        self.cfg.add_edge(node_id, target_node_id, edge_kind);

        // For conditional jumps, also add a fallthrough edge
        if (opcode.to_uppercase() == "JGTZ" || opcode.to_uppercase() == "JZERO")
            && instr_index + 1 < self.body.instructions.len()
        {
            let next_instr_id = self.body.instructions[instr_index + 1].id;
            let next_node_id = self.instr_to_node[&next_instr_id];
            self.cfg.add_edge(node_id, next_node_id, EdgeKind::ConditionalFalse);
        }
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
            let is_jump =
                matches!(instr.opcode.to_uppercase().as_str(), "JUMP" | "JMP" | "JGTZ" | "JZERO");

            // Check if this is a halt instruction
            let is_halt = instr.opcode.to_uppercase() == "HALT";

            if is_jump {
                // Add a conditional edge to the jump target
                if let Some(operand_id) = instr.operand
                    && let Some(expr) = self.body.exprs.get(operand_id.0 as usize)
                {
                    match &expr.kind {
                        // Handle literal label references (string literals representing labels)
                        hir::body::ExprKind::Literal(hir::body::Literal::Label(label_name)) => {
                            if let Some(&target_instr_id) = self.label_to_instr.get(label_name)
                                && let Some(&target_node_id) =
                                    self.instr_to_node.get(&target_instr_id)
                            {
                                // Add appropriate edges for this jump instruction
                                self.add_jump_edges(node_id, target_node_id, &instr.opcode, i);
                            }
                        }
                        // Handle LabelRef type - this is what we were missing
                        hir::body::ExprKind::LabelRef(label_ref) => {
                            // Find the label by its ID
                            if let Some(label) = self.find_label_by_id(label_ref.label_id)
                                && let Some(&target_instr_id) = self.label_to_instr.get(&label.name)
                                && let Some(&target_node_id) =
                                    self.instr_to_node.get(&target_instr_id)
                            {
                                // Add appropriate edges for this jump instruction
                                self.add_jump_edges(node_id, target_node_id, &instr.opcode, i);
                            }
                        }
                        _ => {
                            // Non-label operand, can't determine jump target statically
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
