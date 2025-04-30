//! Data flow graph implementation using petgraph
//!
//! This module provides the implementation of the data flow graph (DFG).
//! The DFG represents the flow of data through a program, where nodes
//! are instructions and edges represent data dependencies.

use std::collections::{HashMap, HashSet};

use hir::ids::LocalDefId;
use petgraph::algo::toposort;
use petgraph::dot::{Config, Dot};
use petgraph::graph::{DiGraph, EdgeIndex, NodeIndex};
use petgraph::visit::EdgeRef;

/// The value flowing through a data flow edge
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataFlowValue {
    /// A value in memory at a specific address
    Memory(i64),
    /// A value in the accumulator
    Accumulator,
}

/// A node in the data flow graph
#[derive(Debug, Clone)]
pub struct DataFlowNode {
    /// The ID of the instruction this node represents
    pub instruction_id: LocalDefId,
}

impl DataFlowNode {
    /// Create a new data flow node
    pub fn new(instruction_id: LocalDefId) -> Self {
        Self { instruction_id }
    }
}

/// A data flow graph
///
/// The data flow graph represents the flow of data through a program,
/// where nodes are instructions and edges represent data dependencies.
#[derive(Debug, Clone)]
pub struct DataFlowGraph {
    /// The underlying petgraph directed graph
    graph: DiGraph<DataFlowNode, DataFlowValue>,
    /// Map from instruction IDs to node indices
    instr_to_node: HashMap<LocalDefId, NodeIndex>,
}

impl DataFlowGraph {
    /// Create a new data flow graph
    pub fn new() -> Self {
        Self { graph: DiGraph::new(), instr_to_node: HashMap::new() }
    }

    /// Add a node to the graph
    pub fn add_node(&mut self, node: DataFlowNode) -> NodeIndex {
        let node_idx = self.graph.add_node(node);
        self.instr_to_node.insert(self.graph[node_idx].instruction_id, node_idx);
        node_idx
    }

    /// Add an edge to the graph
    pub fn add_edge(
        &mut self,
        source: NodeIndex,
        target: NodeIndex,
        value: DataFlowValue,
    ) -> EdgeIndex {
        self.graph.add_edge(source, target, value)
    }

    /// Get a node by its index
    pub fn get_node(&self, node_idx: NodeIndex) -> &DataFlowNode {
        &self.graph[node_idx]
    }

    /// Get a node by its instruction ID
    pub fn get_node_by_instruction(&self, instr_id: LocalDefId) -> Option<&DataFlowNode> {
        self.instr_to_node.get(&instr_id).map(|&node_idx| &self.graph[node_idx])
    }

    /// Get a node index by its instruction ID
    pub fn get_node_idx_by_instruction(&self, instr_id: LocalDefId) -> Option<NodeIndex> {
        self.instr_to_node.get(&instr_id).copied()
    }

    /// Get the incoming edges to a node
    pub fn get_incoming_edges(&self, node_idx: NodeIndex) -> Vec<(NodeIndex, DataFlowValue)> {
        self.graph
            .edges_directed(node_idx, petgraph::Direction::Incoming)
            .map(|edge| (edge.source(), *edge.weight()))
            .collect()
    }

    /// Get the outgoing edges from a node
    pub fn get_outgoing_edges(&self, node_idx: NodeIndex) -> Vec<(NodeIndex, DataFlowValue)> {
        self.graph
            .edges_directed(node_idx, petgraph::Direction::Outgoing)
            .map(|edge| (edge.target(), *edge.weight()))
            .collect()
    }

    /// Find memory addresses that are read before being written
    ///
    /// Returns a set of (address, instruction_id) pairs
    pub fn find_uninitialized_reads(&self) -> HashSet<(i64, LocalDefId)> {
        let mut uninitialized = HashSet::new();
        let mut initialized = HashSet::new();

        // Topologically sort the nodes
        let sorted_nodes = match self.topological_sort() {
            Ok(nodes) => nodes,
            Err(_) => {
                // If there's a cycle, just use all nodes in any order
                self.graph.node_indices().collect()
            }
        };

        // Analyze each node in topological order
        for node_idx in sorted_nodes {
            let node = &self.graph[node_idx];
            let instr_id = node.instruction_id;

            // Check incoming edges for reads
            for (_, value) in self.get_incoming_edges(node_idx) {
                if let DataFlowValue::Memory(addr) = value {
                    if !initialized.contains(&addr) {
                        uninitialized.insert((addr, instr_id));
                    }
                }
            }

            // Check outgoing edges for writes
            for (_, value) in self.get_outgoing_edges(node_idx) {
                if let DataFlowValue::Memory(addr) = value {
                    initialized.insert(addr);
                }
            }
        }

        uninitialized
    }

    /// Find memory addresses that are written but never read
    ///
    /// Returns a set of (address, instruction_id) pairs
    pub fn find_unused_writes(&self) -> HashSet<(i64, LocalDefId)> {
        let mut written_addrs = HashSet::new();
        let mut read_addrs = HashSet::new();
        let mut written_with_instr = HashMap::new();

        // Collect all memory addresses that are written to or read from
        for node_idx in self.graph.node_indices() {
            let node = &self.graph[node_idx];
            let instr_id = node.instruction_id;

            for (_, value) in self.get_outgoing_edges(node_idx) {
                if let DataFlowValue::Memory(addr) = value {
                    written_addrs.insert(addr);
                    written_with_instr.insert(addr, instr_id);
                }
            }

            for (_, value) in self.get_incoming_edges(node_idx) {
                if let DataFlowValue::Memory(addr) = value {
                    read_addrs.insert(addr);
                }
            }
        }

        // Find addresses that are written to but never read from
        written_addrs
            .difference(&read_addrs)
            .filter_map(|addr| written_with_instr.get(addr).map(|instr_id| (*addr, *instr_id)))
            .collect()
    }

    /// Perform a topological sort of the nodes
    fn topological_sort(&self) -> Result<Vec<NodeIndex>, petgraph::algo::Cycle<NodeIndex>> {
        toposort(&self.graph, None)
    }

    /// Get a DOT representation of the graph for visualization
    pub fn to_dot(&self) -> String {
        format!("{:?}", Dot::with_config(&self.graph, &[Config::EdgeNoLabel]))
    }

    /// Get the underlying petgraph directed graph
    pub fn graph(&self) -> &DiGraph<DataFlowNode, DataFlowValue> {
        &self.graph
    }

    /// Get a mutable reference to the underlying petgraph directed graph
    pub fn graph_mut(&mut self) -> &mut DiGraph<DataFlowNode, DataFlowValue> {
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
}

impl Default for DataFlowGraph {
    fn default() -> Self {
        Self::new()
    }
}
