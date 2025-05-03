//! Control flow graph implementation using petgraph
//!
//! This module provides the implementation of the control flow graph (CFG).
//! The CFG represents the control flow of a program as a directed graph,
//! where nodes are instructions and edges represent possible control flow paths.

use std::collections::{HashMap, HashSet};

use hir::ids::LocalDefId;
use petgraph::algo::{dominators, has_path_connecting, toposort};
use petgraph::dot::{Config, Dot};
use petgraph::graph::{DiGraph, EdgeIndex, NodeIndex};
use petgraph::visit::{Dfs, EdgeRef};

/// The kind of edge in the control flow graph
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeKind {
    /// An unconditional edge (e.g., fallthrough)
    Unconditional,
    /// A conditional edge taken when the condition is true
    ConditionalTrue,
    /// A conditional edge taken when the condition is false
    ConditionalFalse,
}

/// A node in the control flow graph
#[derive(Debug, Clone)]
pub struct Node {
    /// The ID of the instruction this node represents
    pub instruction_id: Option<LocalDefId>,
}

impl Node {
    /// Create a new node
    pub fn new(instruction_id: Option<LocalDefId>) -> Self {
        Self { instruction_id }
    }
}

/// A basic block in the control flow graph
///
/// A basic block is a sequence of instructions with a single entry point
/// and a single exit point. Control flow enters at the beginning and
/// exits at the end without halting or branching except at the exit.
#[derive(Debug, Clone)]
pub struct BasicBlock {
    /// The nodes in this basic block
    pub nodes: Vec<NodeIndex>,
}

impl BasicBlock {
    /// Create a new basic block
    pub fn new(nodes: Vec<NodeIndex>) -> Self {
        Self { nodes }
    }

    /// Get the entry node of this basic block
    pub fn entry_node(&self) -> Option<NodeIndex> {
        self.nodes.first().copied()
    }

    /// Get the exit node of this basic block
    pub fn exit_node(&self) -> Option<NodeIndex> {
        self.nodes.last().copied()
    }
}

/// A control flow graph
///
/// The control flow graph represents the control flow of a program as a directed graph,
/// where nodes are instructions and edges represent possible control flow paths.
#[derive(Debug, Clone)]
pub struct ControlFlowGraph {
    /// The underlying petgraph directed graph
    graph: DiGraph<Node, EdgeKind>,
    /// The basic blocks in the graph
    basic_blocks: Vec<BasicBlock>,
    /// The entry node of the graph
    entry_node: Option<NodeIndex>,
    /// Map from instruction IDs to node indices
    instr_to_node: HashMap<LocalDefId, NodeIndex>,
}

impl ControlFlowGraph {
    /// Create a new control flow graph
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            basic_blocks: Vec::new(),
            entry_node: None,
            instr_to_node: HashMap::new(),
        }
    }

    /// Add a node to the graph
    pub fn add_node(&mut self, node: Node) -> NodeIndex {
        let node_idx = self.graph.add_node(node);

        // If this is the first node, it's the entry node
        if self.entry_node.is_none() {
            self.entry_node = Some(node_idx);
        }

        // If this node has an instruction ID, add it to the map
        if let Some(instr_id) = self.graph[node_idx].instruction_id {
            self.instr_to_node.insert(instr_id, node_idx);
        }

        node_idx
    }

    /// Add an edge to the graph
    pub fn add_edge(&mut self, source: NodeIndex, target: NodeIndex, kind: EdgeKind) -> EdgeIndex {
        self.graph.add_edge(source, target, kind)
    }

    /// Add a basic block to the graph
    pub fn add_basic_block(&mut self, block: BasicBlock) -> usize {
        let block_id = self.basic_blocks.len();
        self.basic_blocks.push(block);
        block_id
    }

    /// Get a node by its index
    pub fn get_node(&self, node_idx: NodeIndex) -> &Node {
        &self.graph[node_idx]
    }

    /// Get a node by its instruction ID
    pub fn get_node_by_instruction(&self, instr_id: LocalDefId) -> Option<NodeIndex> {
        self.instr_to_node.get(&instr_id).copied()
    }

    /// Get a basic block by its ID
    pub fn get_basic_block(&self, block_id: usize) -> &BasicBlock {
        &self.basic_blocks[block_id]
    }

    /// Get the entry node of the graph
    pub fn entry_node(&self) -> Option<NodeIndex> {
        self.entry_node
    }

    /// Get the incoming edges to a node
    pub fn get_incoming_edges(&self, node_idx: NodeIndex) -> Vec<(NodeIndex, EdgeKind)> {
        self.graph
            .edges_directed(node_idx, petgraph::Direction::Incoming)
            .map(|edge| (edge.source(), *edge.weight()))
            .collect()
    }

    /// Get the outgoing edges from a node
    pub fn get_outgoing_edges(&self, node_idx: NodeIndex) -> Vec<(NodeIndex, EdgeKind)> {
        self.graph
            .edges_directed(node_idx, petgraph::Direction::Outgoing)
            .map(|edge| (edge.target(), *edge.weight()))
            .collect()
    }

    /// Get the successors of a node
    pub fn get_successors(&self, node_idx: NodeIndex) -> Vec<NodeIndex> {
        self.graph.neighbors_directed(node_idx, petgraph::Direction::Outgoing).collect()
    }

    /// Get the predecessors of a node
    pub fn get_predecessors(&self, node_idx: NodeIndex) -> Vec<NodeIndex> {
        self.graph.neighbors_directed(node_idx, petgraph::Direction::Incoming).collect()
    }

    /// Find unreachable nodes in the graph
    pub fn find_unreachable_nodes(&self) -> Vec<NodeIndex> {
        let mut unreachable = Vec::new();

        // If there's no entry node, all nodes are unreachable
        if self.entry_node.is_none() {
            return self.graph.node_indices().collect();
        }

        let entry = self.entry_node.unwrap();

        // Perform a depth-first search from the entry node
        let mut dfs = Dfs::new(&self.graph, entry);
        let mut visited = HashSet::new();

        while let Some(node_idx) = dfs.next(&self.graph) {
            visited.insert(node_idx);
        }

        // Any node not visited is unreachable
        for node_idx in self.graph.node_indices() {
            if !visited.contains(&node_idx) {
                unreachable.push(node_idx);
            }
        }

        unreachable
    }

    /// Find infinite loops in the graph
    pub fn find_infinite_loops(&self) -> Vec<Vec<NodeIndex>> {
        let mut loops = Vec::new();

        // Check if the graph has cycles
        if !is_cyclic_directed(&self.graph) {
            return loops;
        }

        // Find strongly connected components
        let sccs = petgraph::algo::kosaraju_scc(&self.graph);

        // Filter out components with only one node (not loops)
        // and components that have an exit (not infinite loops)
        for scc in sccs {
            if scc.len() > 1 {
                let mut has_exit = false;

                // Check if any node in the SCC has an edge to a node outside the SCC
                for &node_idx in &scc {
                    for succ in self.get_successors(node_idx) {
                        if !scc.contains(&succ) {
                            has_exit = true;
                            break;
                        }
                    }

                    if has_exit {
                        break;
                    }
                }

                if !has_exit {
                    loops.push(scc);
                }
            }
        }

        loops
    }

    /// Compute dominators for each node
    pub fn compute_dominators(&self) -> HashMap<NodeIndex, HashSet<NodeIndex>> {
        let mut result = HashMap::new();

        // If there's no entry node, return an empty map
        if self.entry_node.is_none() {
            return result;
        }

        let entry = self.entry_node.unwrap();

        // Use petgraph's dominators algorithm
        let dom = dominators::simple_fast(&self.graph, entry);

        // Convert the result to a HashMap<NodeIndex, HashSet<NodeIndex>>
        for node_idx in self.graph.node_indices() {
            let mut dom_set = HashSet::new();

            // Add all dominators of this node
            let mut current = Some(node_idx);
            while let Some(idx) = current {
                dom_set.insert(idx);
                current = dom.immediate_dominator(idx);
            }

            result.insert(node_idx, dom_set);
        }

        result
    }

    /// Compute the post-dominators for each node
    pub fn compute_post_dominators(&self) -> HashMap<NodeIndex, HashSet<NodeIndex>> {
        let mut result = HashMap::new();

        // Create a reversed graph
        let mut reversed = self.graph.clone();
        reversed.reverse();

        // Find exit nodes (nodes with no successors in the original graph)
        let exit_nodes: Vec<_> = self
            .graph
            .node_indices()
            .filter(|&node_idx| self.get_successors(node_idx).is_empty())
            .collect();

        // If there are no exit nodes, return an empty map
        if exit_nodes.is_empty() {
            return result;
        }

        // For each exit node, compute dominators in the reversed graph
        for &exit in &exit_nodes {
            let dom = dominators::simple_fast(&reversed, exit);

            // Add the dominators to the result
            for node_idx in reversed.node_indices() {
                let dom_set = result.entry(node_idx).or_insert_with(HashSet::new);

                // Add all dominators of this node
                let mut current = Some(node_idx);
                while let Some(idx) = current {
                    dom_set.insert(idx);
                    current = dom.immediate_dominator(idx);
                }
            }
        }

        result
    }

    /// Get a DOT representation of the graph for visualization
    pub fn to_dot(&self) -> String {
        format!("{:?}", Dot::with_config(&self.graph, &[Config::EdgeNoLabel]))
    }

    /// Get a Mermaid representation of the graph for visualization
    pub fn to_mermaid(&self) -> String {
        let mut result = String::from("graph TD\n");

        // Add nodes
        for node_idx in self.graph.node_indices() {
            let node_id = format!("N{}", node_idx.index());
            let node = &self.graph[node_idx];

            let label = if let Some(instr_id) = node.instruction_id {
                format!("Instr {}", instr_id.0)
            } else {
                "Unknown".to_string()
            };

            result.push_str(&format!("    {}[\"{}\"]\n", node_id, label));
        }

        // Add edges
        for edge in self.graph.edge_indices() {
            let (source, target) = self.graph.edge_endpoints(edge).unwrap();
            let source_id = format!("N{}", source.index());
            let target_id = format!("N{}", target.index());
            let edge_kind = self.graph.edge_weight(edge).unwrap();

            let edge_style = match edge_kind {
                EdgeKind::Unconditional => "-->",
                EdgeKind::ConditionalTrue => "-.->|true|",
                EdgeKind::ConditionalFalse => "-.->|false|",
            };

            result.push_str(&format!("    {} {} {}\n", source_id, edge_style, target_id));
        }

        result
    }

    /// Get a Mermaid representation of the graph with detailed instruction information
    pub fn to_mermaid_with_context(&self, context: &crate::context::AnalysisContext) -> String {
        let mut result = String::from("graph TD\n");
        let body = context.body();

        // Add nodes with detailed instruction information
        for node_idx in self.graph.node_indices() {
            let node_id = format!("N{}", node_idx.index());
            let node = &self.graph[node_idx];

            let label = if let Some(instr_id) = node.instruction_id {
                // Find the instruction in the body
                let instr = body
                    .instructions
                    .iter()
                    .find(|i| i.id == instr_id)
                    .map(|i| {
                        let operand_str = match i.operand {
                            Some(expr_id) => {
                                // Try to find the expression
                                if let Some(expr) = body.exprs.get(expr_id.0 as usize) {
                                    match &expr.kind {
                                        hir::body::ExprKind::Literal(lit) => match lit {
                                            hir::body::Literal::Int(val) => format!("{}", val),
                                            hir::body::Literal::String(s) => format!("\"{}\"", s),
                                            hir::body::Literal::Label(label) => {
                                                format!(":{}", label)
                                            }
                                        },
                                        hir::body::ExprKind::LabelRef(label_ref) => {
                                            // Find the label name from the label_id
                                            // We need to match on the local_id part of the DefId
                                            let label_name = body
                                                .labels
                                                .iter()
                                                .find(|l| l.id.0 == label_ref.label_id.local_id.0)
                                                .map(|l| format!(":{}", l.name))
                                                .unwrap_or_else(|| {
                                                    format!(
                                                        "label_{}",
                                                        label_ref.label_id.local_id.0
                                                    )
                                                });
                                            label_name
                                        }
                                        hir::body::ExprKind::MemoryRef(mem_ref) => {
                                            let mode_prefix = match mem_ref.mode {
                                                hir::body::AddressingMode::Direct => "",
                                                hir::body::AddressingMode::Indirect => "*",
                                                hir::body::AddressingMode::Immediate => "=",
                                            };

                                            if let Some(addr_expr) =
                                                body.exprs.get(mem_ref.address.0 as usize)
                                            {
                                                if let hir::body::ExprKind::Literal(
                                                    hir::body::Literal::Int(val),
                                                ) = &addr_expr.kind
                                                {
                                                    format!("{}{}", mode_prefix, val)
                                                } else {
                                                    format!("{}?", mode_prefix)
                                                }
                                            } else {
                                                format!("{}?", mode_prefix)
                                            }
                                        }
                                        hir::body::ExprKind::InstructionCall(_) => {
                                            "call".to_string()
                                        }
                                    }
                                } else {
                                    "?".to_string()
                                }
                            }
                            None => "".to_string(),
                        };

                        if operand_str.is_empty() {
                            i.opcode.to_string()
                        } else {
                            format!("{} {}", i.opcode, operand_str)
                        }
                    })
                    .unwrap_or_else(|| format!("Instr {}", instr_id.0));

                instr
            } else {
                "Unknown".to_string()
            };

            // Escape quotes for Mermaid
            let escaped_label = label.replace("\"", "\\\"");
            result.push_str(&format!("    {}[\"{}\"] \n", node_id, escaped_label));
        }

        // Add edges
        for edge in self.graph.edge_indices() {
            let (source, target) = self.graph.edge_endpoints(edge).unwrap();
            let source_id = format!("N{}", source.index());
            let target_id = format!("N{}", target.index());
            let edge_kind = self.graph.edge_weight(edge).unwrap();

            let edge_style = match edge_kind {
                EdgeKind::Unconditional => "-->",
                EdgeKind::ConditionalTrue => "-.->|true|",
                EdgeKind::ConditionalFalse => "-.->|false|",
            };

            result.push_str(&format!("    {} {} {}\n", source_id, edge_style, target_id));
        }

        result
    }

    /// Get the underlying petgraph directed graph
    pub fn graph(&self) -> &DiGraph<Node, EdgeKind> {
        &self.graph
    }

    /// Get a mutable reference to the underlying petgraph directed graph
    pub fn graph_mut(&mut self) -> &mut DiGraph<Node, EdgeKind> {
        &mut self.graph
    }

    /// Get all node indices in the graph
    pub fn node_indices(&self) -> Vec<NodeIndex> {
        self.graph.node_indices().collect()
    }

    /// Get the number of nodes in the graph
    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    /// Get the number of edges in the graph
    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    /// Check if the graph is empty
    pub fn is_empty(&self) -> bool {
        self.graph.node_count() == 0
    }

    /// Perform a topological sort of the graph
    pub fn topological_sort(&self) -> Result<Vec<NodeIndex>, petgraph::algo::Cycle<NodeIndex>> {
        toposort(&self.graph, None)
    }

    /// Check if there's a path from source to target
    pub fn has_path(&self, source: NodeIndex, target: NodeIndex) -> bool {
        has_path_connecting(&self.graph, source, target, None)
    }
}

impl Default for ControlFlowGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if a directed graph has cycles
fn is_cyclic_directed<N, E>(graph: &DiGraph<N, E>) -> bool {
    // Try to perform a topological sort
    // If it fails, the graph has cycles
    toposort(graph, None).is_err()
}
