//! Data flow graph implementation
//!
//! This module provides the implementation of the data flow graph (DFG).
//! The DFG represents the flow of data through a program, where nodes
//! are instructions and edges represent data dependencies.

use std::collections::{HashMap, HashSet};

use hir::ids::LocalDefId;

/// A node in the data flow graph
#[derive(Debug, Clone)]
pub struct DataFlowNode {
    /// The ID of the instruction this node represents
    pub instruction_id: LocalDefId,
    /// Incoming edges to this node
    incoming_edges: Vec<DataFlowEdge>,
    /// Outgoing edges from this node
    outgoing_edges: Vec<DataFlowEdge>,
}

impl DataFlowNode {
    /// Create a new data flow node
    pub fn new(instruction_id: LocalDefId) -> Self {
        Self {
            instruction_id,
            incoming_edges: Vec::new(),
            outgoing_edges: Vec::new(),
        }
    }

    /// Add an incoming edge to this node
    pub fn add_incoming_edge(&mut self, edge: DataFlowEdge) {
        self.incoming_edges.push(edge);
    }

    /// Add an outgoing edge from this node
    pub fn add_outgoing_edge(&mut self, edge: DataFlowEdge) {
        self.outgoing_edges.push(edge);
    }

    /// Get the incoming edges to this node
    pub fn incoming_edges(&self) -> &[DataFlowEdge] {
        &self.incoming_edges
    }

    /// Get the outgoing edges from this node
    pub fn outgoing_edges(&self) -> &[DataFlowEdge] {
        &self.outgoing_edges
    }
}

/// The value flowing through a data flow edge
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataFlowValue {
    /// A value in memory at a specific address
    Memory(i64),
    /// A value in the accumulator
    Accumulator,
}

/// An edge in the data flow graph
#[derive(Debug, Clone, Copy)]
pub struct DataFlowEdge {
    /// The source node of the edge
    pub source: usize,
    /// The target node of the edge
    pub target: usize,
    /// The value flowing through the edge
    pub value: DataFlowValue,
}

impl DataFlowEdge {
    /// Create a new data flow edge
    pub fn new(source: usize, target: usize, value: DataFlowValue) -> Self {
        Self { source, target, value }
    }
}

/// A data flow graph
///
/// The data flow graph represents the flow of data through a program,
/// where nodes are instructions and edges represent data dependencies.
#[derive(Debug, Clone)]
pub struct DataFlowGraph {
    /// The nodes in the graph
    nodes: Vec<DataFlowNode>,
    /// Map from instruction IDs to node IDs
    instr_to_node: HashMap<LocalDefId, usize>,
}

impl DataFlowGraph {
    /// Create a new data flow graph
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            instr_to_node: HashMap::new(),
        }
    }

    /// Add a node to the graph
    pub fn add_node(&mut self, node: DataFlowNode) -> usize {
        let node_id = self.nodes.len();
        self.instr_to_node.insert(node.instruction_id, node_id);
        self.nodes.push(node);
        node_id
    }

    /// Add an edge to the graph
    pub fn add_edge(&mut self, source: usize, target: usize, value: DataFlowValue) {
        let edge = DataFlowEdge::new(source, target, value);
        
        // Add the edge to the source node's outgoing edges
        if let Some(source_node) = self.nodes.get_mut(source) {
            source_node.add_outgoing_edge(edge);
        }
        
        // Add the edge to the target node's incoming edges
        if let Some(target_node) = self.nodes.get_mut(target) {
            target_node.add_incoming_edge(edge);
        }
    }

    /// Get a node by its ID
    pub fn get_node(&self, node_id: usize) -> &DataFlowNode {
        &self.nodes[node_id]
    }

    /// Get a node by its instruction ID
    pub fn get_node_by_instruction(&self, instr_id: LocalDefId) -> Option<&DataFlowNode> {
        self.instr_to_node.get(&instr_id).map(|&node_id| &self.nodes[node_id])
    }

    /// Get the incoming edges to a node
    pub fn get_incoming_edges(&self, node_id: usize) -> Vec<DataFlowEdge> {
        self.nodes[node_id].incoming_edges().to_vec()
    }

    /// Get the outgoing edges from a node
    pub fn get_outgoing_edges(&self, node_id: usize) -> Vec<DataFlowEdge> {
        self.nodes[node_id].outgoing_edges().to_vec()
    }

    /// Find memory addresses that are read before being written
    pub fn find_uninitialized_reads(&self) -> HashSet<i64> {
        let mut uninitialized = HashSet::new();
        let mut initialized = HashSet::new();
        
        // Topologically sort the nodes
        let sorted_nodes = self.topological_sort();
        
        // Analyze each node in topological order
        for &node_id in &sorted_nodes {
            let node = &self.nodes[node_id];
            
            // Check incoming edges for reads
            for edge in node.incoming_edges() {
                if let DataFlowValue::Memory(addr) = edge.value {
                    if !initialized.contains(&addr) {
                        uninitialized.insert(addr);
                    }
                }
            }
            
            // Check outgoing edges for writes
            for edge in node.outgoing_edges() {
                if let DataFlowValue::Memory(addr) = edge.value {
                    initialized.insert(addr);
                }
            }
        }
        
        uninitialized
    }

    /// Find memory addresses that are written but never read
    pub fn find_unused_writes(&self) -> HashSet<i64> {
        let mut written = HashSet::new();
        let mut read = HashSet::new();
        
        // Collect all memory addresses that are written to or read from
        for node in &self.nodes {
            for edge in node.outgoing_edges() {
                if let DataFlowValue::Memory(addr) = edge.value {
                    written.insert(addr);
                }
            }
            
            for edge in node.incoming_edges() {
                if let DataFlowValue::Memory(addr) = edge.value {
                    read.insert(addr);
                }
            }
        }
        
        // Find addresses that are written to but never read from
        written.difference(&read).copied().collect()
    }

    /// Perform a topological sort of the nodes
    fn topological_sort(&self) -> Vec<usize> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut temp = HashSet::new();
        
        // Visit each node
        for node_id in 0..self.nodes.len() {
            if !visited.contains(&node_id) {
                self.visit(node_id, &mut visited, &mut temp, &mut result);
            }
        }
        
        result.reverse();
        result
    }

    /// Helper function for topological sort
    fn visit(
        &self,
        node_id: usize,
        visited: &mut HashSet<usize>,
        temp: &mut HashSet<usize>,
        result: &mut Vec<usize>,
    ) {
        // Check if the node is already being visited
        if temp.contains(&node_id) {
            // Cycle detected, but we'll ignore it for now
            return;
        }
        
        // Check if the node has already been visited
        if visited.contains(&node_id) {
            return;
        }
        
        // Mark the node as being visited
        temp.insert(node_id);
        
        // Visit all successors
        for edge in self.get_outgoing_edges(node_id) {
            self.visit(edge.target, visited, temp, result);
        }
        
        // Mark the node as visited
        visited.insert(node_id);
        temp.remove(&node_id);
        
        // Add the node to the result
        result.push(node_id);
    }
}
