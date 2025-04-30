//! Control flow graph implementation
//!
//! This module provides the implementation of the control flow graph (CFG).
//! The CFG represents the control flow of a program as a directed graph,
//! where nodes are instructions and edges represent possible control flow paths.

use std::collections::{HashMap, HashSet};

use hir::ids::LocalDefId;

/// A node in the control flow graph
#[derive(Debug, Clone)]
pub struct Node {
    /// The ID of the instruction this node represents
    pub instruction_id: Option<LocalDefId>,
    /// Incoming edges to this node
    incoming_edges: Vec<Edge>,
    /// Outgoing edges from this node
    outgoing_edges: Vec<Edge>,
}

impl Node {
    /// Create a new node
    pub fn new(instruction_id: Option<LocalDefId>) -> Self {
        Self { instruction_id, incoming_edges: Vec::new(), outgoing_edges: Vec::new() }
    }

    /// Add an incoming edge to this node
    pub fn add_incoming_edge(&mut self, edge: Edge) {
        self.incoming_edges.push(edge);
    }

    /// Add an outgoing edge from this node
    pub fn add_outgoing_edge(&mut self, edge: Edge) {
        self.outgoing_edges.push(edge);
    }

    /// Get the incoming edges to this node
    pub fn incoming_edges(&self) -> &[Edge] {
        &self.incoming_edges
    }

    /// Get the outgoing edges from this node
    pub fn outgoing_edges(&self) -> &[Edge] {
        &self.outgoing_edges
    }
}

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

/// An edge in the control flow graph
#[derive(Debug, Clone, Copy)]
pub struct Edge {
    /// The source node of the edge
    pub source: usize,
    /// The target node of the edge
    pub target: usize,
    /// The kind of edge
    pub kind: EdgeKind,
}

impl Edge {
    /// Create a new edge
    pub fn new(source: usize, target: usize, kind: EdgeKind) -> Self {
        Self { source, target, kind }
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
    pub nodes: Vec<usize>,
}

impl BasicBlock {
    /// Create a new basic block
    pub fn new(nodes: Vec<usize>) -> Self {
        Self { nodes }
    }

    /// Get the entry node of this basic block
    pub fn entry_node(&self) -> Option<usize> {
        self.nodes.first().copied()
    }

    /// Get the exit node of this basic block
    pub fn exit_node(&self) -> Option<usize> {
        self.nodes.last().copied()
    }
}

/// A control flow graph
///
/// The control flow graph represents the control flow of a program as a directed graph,
/// where nodes are instructions and edges represent possible control flow paths.
#[derive(Debug, Clone)]
pub struct ControlFlowGraph {
    /// The nodes in the graph
    nodes: Vec<Node>,
    /// The basic blocks in the graph
    basic_blocks: Vec<BasicBlock>,
    /// The entry node of the graph
    entry_node: Option<usize>,
}

impl ControlFlowGraph {
    /// Create a new control flow graph
    pub fn new() -> Self {
        Self { nodes: Vec::new(), basic_blocks: Vec::new(), entry_node: None }
    }

    /// Add a node to the graph
    pub fn add_node(&mut self, node: Node) -> usize {
        let node_id = self.nodes.len();
        self.nodes.push(node);

        // If this is the first node, it's the entry node
        if self.entry_node.is_none() {
            self.entry_node = Some(node_id);
        }

        node_id
    }

    /// Add an edge to the graph
    pub fn add_edge(&mut self, source: usize, target: usize, kind: EdgeKind) {
        let edge = Edge::new(source, target, kind);

        // Add the edge to the source node's outgoing edges
        if let Some(source_node) = self.nodes.get_mut(source) {
            source_node.add_outgoing_edge(edge);
        }

        // Add the edge to the target node's incoming edges
        if let Some(target_node) = self.nodes.get_mut(target) {
            target_node.add_incoming_edge(edge);
        }
    }

    /// Add a basic block to the graph
    pub fn add_basic_block(&mut self, block: BasicBlock) -> usize {
        let block_id = self.basic_blocks.len();
        self.basic_blocks.push(block);
        block_id
    }

    /// Get a node by its ID
    pub fn get_node(&self, node_id: usize) -> &Node {
        &self.nodes[node_id]
    }

    /// Get a basic block by its ID
    pub fn get_basic_block(&self, block_id: usize) -> &BasicBlock {
        &self.basic_blocks[block_id]
    }

    /// Get the entry node of the graph
    pub fn entry_node(&self) -> Option<usize> {
        self.entry_node
    }

    /// Get the incoming edges to a node
    pub fn get_incoming_edges(&self, node_id: usize) -> Vec<Edge> {
        self.nodes[node_id].incoming_edges().to_vec()
    }

    /// Get the outgoing edges from a node
    pub fn get_outgoing_edges(&self, node_id: usize) -> Vec<Edge> {
        self.nodes[node_id].outgoing_edges().to_vec()
    }

    /// Get the successors of a node
    pub fn get_successors(&self, node_id: usize) -> Vec<usize> {
        self.nodes[node_id].outgoing_edges().iter().map(|edge| edge.target).collect()
    }

    /// Get the predecessors of a node
    pub fn get_predecessors(&self, node_id: usize) -> Vec<usize> {
        self.nodes[node_id].incoming_edges().iter().map(|edge| edge.source).collect()
    }

    /// Find unreachable nodes in the graph
    pub fn find_unreachable_nodes(&self) -> Vec<usize> {
        let mut unreachable = Vec::new();
        let mut visited = HashSet::new();

        // Perform a depth-first search from the entry node
        if let Some(entry) = self.entry_node {
            let mut stack = vec![entry];

            while let Some(node_id) = stack.pop() {
                if visited.insert(node_id) {
                    // Add all successors to the stack
                    for succ in self.get_successors(node_id) {
                        stack.push(succ);
                    }
                }
            }
        }

        // Any node not visited is unreachable
        for node_id in 0..self.nodes.len() {
            if !visited.contains(&node_id) {
                unreachable.push(node_id);
            }
        }

        unreachable
    }

    /// Find infinite loops in the graph
    pub fn find_infinite_loops(&self) -> Vec<Vec<usize>> {
        let mut loops = Vec::new();
        let mut visited = HashSet::new();
        let mut stack = Vec::new();
        let mut on_stack = HashSet::new();

        // Perform a depth-first search to find strongly connected components
        for node_id in 0..self.nodes.len() {
            if !visited.contains(&node_id) {
                self.find_loops_dfs(node_id, &mut visited, &mut stack, &mut on_stack, &mut loops);
            }
        }

        loops
    }

    /// Helper function for finding loops using depth-first search
    fn find_loops_dfs(
        &self,
        node_id: usize,
        visited: &mut HashSet<usize>,
        stack: &mut Vec<usize>,
        on_stack: &mut HashSet<usize>,
        loops: &mut Vec<Vec<usize>>,
    ) {
        visited.insert(node_id);
        stack.push(node_id);
        on_stack.insert(node_id);

        for succ in self.get_successors(node_id) {
            if !visited.contains(&succ) {
                self.find_loops_dfs(succ, visited, stack, on_stack, loops);
            } else if on_stack.contains(&succ) {
                // Found a loop
                let mut loop_nodes = Vec::new();
                let mut i = stack.len() - 1;

                while stack[i] != succ {
                    loop_nodes.push(stack[i]);
                    i -= 1;
                }

                loop_nodes.push(succ);
                loops.push(loop_nodes);
            }
        }

        stack.pop();
        on_stack.remove(&node_id);
    }

    /// Compute dominators for each node
    pub fn compute_dominators(&self) -> HashMap<usize, HashSet<usize>> {
        let mut dominators = HashMap::new();

        // Initialize dominators
        let all_nodes: HashSet<usize> = (0..self.nodes.len()).collect();

        for node_id in 0..self.nodes.len() {
            if Some(node_id) == self.entry_node {
                // The entry node is only dominated by itself
                let mut dom_set = HashSet::new();
                dom_set.insert(node_id);
                dominators.insert(node_id, dom_set);
            } else {
                // All other nodes are dominated by all nodes initially
                dominators.insert(node_id, all_nodes.clone());
            }
        }

        // Iteratively compute dominators
        let mut changed = true;

        while changed {
            changed = false;

            for node_id in 0..self.nodes.len() {
                if Some(node_id) == self.entry_node {
                    continue;
                }

                let preds = self.get_predecessors(node_id);
                if preds.is_empty() {
                    continue;
                }

                // Compute the intersection of all predecessors' dominators
                let mut new_dom = all_nodes.clone();

                for &pred in &preds {
                    if let Some(pred_dom) = dominators.get(&pred) {
                        new_dom = new_dom.intersection(pred_dom).copied().collect();
                    }
                }

                // Add the node itself to its dominators
                new_dom.insert(node_id);

                // Check if the dominators changed
                if let Some(old_dom) = dominators.get(&node_id) {
                    if &new_dom != old_dom {
                        dominators.insert(node_id, new_dom);
                        changed = true;
                    }
                }
            }
        }

        dominators
    }

    /// Compute the post-dominators for each node
    pub fn compute_post_dominators(&self) -> HashMap<usize, HashSet<usize>> {
        let mut post_dominators = HashMap::new();

        // Initialize post-dominators
        let all_nodes: HashSet<usize> = (0..self.nodes.len()).collect();

        // Find exit nodes (nodes with no successors)
        let mut exit_nodes = Vec::new();

        for node_id in 0..self.nodes.len() {
            if self.get_successors(node_id).is_empty() {
                exit_nodes.push(node_id);
            }
        }

        for node_id in 0..self.nodes.len() {
            if exit_nodes.contains(&node_id) {
                // Exit nodes are only post-dominated by themselves
                let mut dom_set = HashSet::new();
                dom_set.insert(node_id);
                post_dominators.insert(node_id, dom_set);
            } else {
                // All other nodes are post-dominated by all nodes initially
                post_dominators.insert(node_id, all_nodes.clone());
            }
        }

        // Iteratively compute post-dominators
        let mut changed = true;

        while changed {
            changed = false;

            for node_id in 0..self.nodes.len() {
                if exit_nodes.contains(&node_id) {
                    continue;
                }

                let succs = self.get_successors(node_id);
                if succs.is_empty() {
                    continue;
                }

                // Compute the intersection of all successors' post-dominators
                let mut new_dom = all_nodes.clone();

                for &succ in &succs {
                    if let Some(succ_dom) = post_dominators.get(&succ) {
                        new_dom = new_dom.intersection(succ_dom).copied().collect();
                    }
                }

                // Add the node itself to its post-dominators
                new_dom.insert(node_id);

                // Check if the post-dominators changed
                if let Some(old_dom) = post_dominators.get(&node_id) {
                    if &new_dom != old_dom {
                        post_dominators.insert(node_id, new_dom);
                        changed = true;
                    }
                }
            }
        }

        post_dominators
    }
}
