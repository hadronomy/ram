//! Data flow analysis for HIR
//!
//! This module provides data flow analysis for HIR bodies.
//! It analyzes how data flows through the program and detects issues
//! such as uninitialized variables and unused values.

use std::any::TypeId;
use std::collections::{HashMap, HashSet};

use hir::body::{Body, ExprKind, Instruction, Literal};
use hir::expr::ExprId;
use hir::ids::LocalDefId;
use miette::Diagnostic;

use crate::analyzers::control_flow::{ControlFlowAnalysis, ControlFlowGraph};
use crate::context::AnalysisContext;
use crate::pass::AnalysisPass;

mod graph;

pub use graph::{DataFlowGraph, DataFlowNode, DataFlowValue};

/// Data flow analysis pass
///
/// This pass analyzes the data flow of a HIR body and builds a data flow graph.
/// It also detects issues such as uninitialized variables and unused values.
#[derive(Default)]
pub struct DataFlowAnalysis;

impl AnalysisPass for DataFlowAnalysis {
    type Output = DataFlowGraph;

    fn name(&self) -> &'static str {
        "DataFlowAnalysis"
    }

    fn dependencies(&self) -> Vec<TypeId> {
        vec![TypeId::of::<ControlFlowAnalysis>()]
    }

    fn run(&self, ctx: &mut AnalysisContext) -> Result<Self::Output, Box<dyn Diagnostic>> {
        let body = ctx.body();

        // Get the control flow graph
        let cfg = match ctx.get_result::<ControlFlowAnalysis>() {
            Ok(cfg) => cfg.clone(),
            Err(e) => return Err(Box::new(e)),
        };

        let mut dfg_builder = DataFlowGraphBuilder::new(body, &cfg);
        let dfg = dfg_builder.build();

        // Check for uninitialized variables
        let uninit = dfg.find_uninitialized_reads();
        for addr in uninit {
            ctx.warning(
                format!("Uninitialized memory read at address {}", addr),
                "This memory location may not be initialized before it is read".to_string(),
                None,
            );
        }

        // Check for unused values
        let unused = dfg.find_unused_writes();
        for addr in unused {
            ctx.info(
                format!("Unused memory write at address {}", addr),
                "This memory write is never read".to_string(),
                None,
            );
        }

        Ok(dfg)
    }
}

/// Builder for data flow graphs
struct DataFlowGraphBuilder<'a> {
    /// The HIR body being analyzed
    body: &'a Body,
    /// The control flow graph
    cfg: &'a ControlFlowGraph,
    /// The data flow graph being built
    dfg: DataFlowGraph,
    /// Map from instruction IDs to data flow node indices
    instr_to_node: HashMap<LocalDefId, petgraph::graph::NodeIndex>,
    /// Set of memory addresses that have been written to
    written_addrs: HashSet<i64>,
    /// Set of memory addresses that have been read from
    read_addrs: HashSet<i64>,
}

impl<'a> DataFlowGraphBuilder<'a> {
    /// Create a new data flow graph builder
    fn new(body: &'a Body, cfg: &'a ControlFlowGraph) -> Self {
        Self {
            body,
            cfg,
            dfg: DataFlowGraph::new(),
            instr_to_node: HashMap::new(),
            written_addrs: HashSet::new(),
            read_addrs: HashSet::new(),
        }
    }

    /// Build the data flow graph
    fn build(&mut self) -> DataFlowGraph {
        // Create nodes for all instructions
        for instr in &self.body.instructions {
            let node_id = self.dfg.add_node(DataFlowNode::new(instr.id));
            self.instr_to_node.insert(instr.id, node_id);
        }

        // Analyze each instruction to determine data flow
        for instr in &self.body.instructions {
            self.analyze_instruction(instr);
        }

        // Add edges between nodes based on data flow
        self.add_data_flow_edges();

        self.dfg.clone()
    }

    /// Analyze an instruction to determine data flow
    fn analyze_instruction(&mut self, instr: &Instruction) {
        match instr.opcode.to_uppercase().as_str() {
            "LOAD" => {
                // LOAD reads from memory and writes to the accumulator
                if let Some(operand_id) = instr.operand {
                    if let Some(addr) = self.get_memory_address(operand_id) {
                        self.read_addrs.insert(addr);
                    }
                }
            }
            "STORE" => {
                // STORE reads from the accumulator and writes to memory
                if let Some(operand_id) = instr.operand {
                    if let Some(addr) = self.get_memory_address(operand_id) {
                        self.written_addrs.insert(addr);
                    }
                }
            }
            "ADD" | "SUB" | "MUL" | "DIV" => {
                // Arithmetic operations read from memory and write to the accumulator
                if let Some(operand_id) = instr.operand {
                    if let Some(addr) = self.get_memory_address(operand_id) {
                        self.read_addrs.insert(addr);
                    }
                }
            }
            "READ" => {
                // READ writes to memory
                if let Some(operand_id) = instr.operand {
                    if let Some(addr) = self.get_memory_address(operand_id) {
                        self.written_addrs.insert(addr);
                    }
                }
            }
            "WRITE" => {
                // WRITE reads from memory
                if let Some(operand_id) = instr.operand {
                    if let Some(addr) = self.get_memory_address(operand_id) {
                        self.read_addrs.insert(addr);
                    }
                }
            }
            _ => {
                // Other instructions may have different data flow patterns
            }
        }
    }

    /// Get the memory address from an expression ID
    fn get_memory_address(&self, expr_id: ExprId) -> Option<i64> {
        if let Some(expr) = self.body.exprs.get(expr_id.0 as usize) {
            match &expr.kind {
                ExprKind::Literal(Literal::Int(addr)) => Some(*addr),
                ExprKind::MemoryRef(mem_ref) => {
                    if let Some(addr_expr) = self.body.exprs.get(mem_ref.address.0 as usize) {
                        match &addr_expr.kind {
                            ExprKind::Literal(Literal::Int(addr)) => Some(*addr),
                            _ => None,
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            }
        } else {
            None
        }
    }

    /// Add edges between nodes based on data flow
    fn add_data_flow_edges(&mut self) {
        // Create a map from memory addresses to the instructions that write to them
        let mut addr_to_writers: HashMap<i64, Vec<LocalDefId>> = HashMap::new();

        // Create a map from memory addresses to the instructions that read from them
        let mut addr_to_readers: HashMap<i64, Vec<LocalDefId>> = HashMap::new();

        // Analyze each instruction to determine data flow
        for instr in &self.body.instructions {
            match instr.opcode.to_uppercase().as_str() {
                "LOAD" => {
                    // LOAD reads from memory
                    if let Some(operand_id) = instr.operand {
                        if let Some(addr) = self.get_memory_address(operand_id) {
                            addr_to_readers.entry(addr).or_default().push(instr.id);
                        }
                    }
                }
                "STORE" => {
                    // STORE writes to memory
                    if let Some(operand_id) = instr.operand {
                        if let Some(addr) = self.get_memory_address(operand_id) {
                            addr_to_writers.entry(addr).or_default().push(instr.id);
                        }
                    }
                }
                "ADD" | "SUB" | "MUL" | "DIV" => {
                    // Arithmetic operations read from memory
                    if let Some(operand_id) = instr.operand {
                        if let Some(addr) = self.get_memory_address(operand_id) {
                            addr_to_readers.entry(addr).or_default().push(instr.id);
                        }
                    }
                }
                "READ" => {
                    // READ writes to memory
                    if let Some(operand_id) = instr.operand {
                        if let Some(addr) = self.get_memory_address(operand_id) {
                            addr_to_writers.entry(addr).or_default().push(instr.id);
                        }
                    }
                }
                "WRITE" => {
                    // WRITE reads from memory
                    if let Some(operand_id) = instr.operand {
                        if let Some(addr) = self.get_memory_address(operand_id) {
                            addr_to_readers.entry(addr).or_default().push(instr.id);
                        }
                    }
                }
                _ => {
                    // Other instructions may have different data flow patterns
                }
            }
        }

        // Add edges from writers to readers
        for (addr, writers) in &addr_to_writers {
            if let Some(readers) = addr_to_readers.get(addr) {
                for &writer in writers {
                    for &reader in readers {
                        // Check if there's a path from writer to reader in the CFG
                        if self.is_reachable(writer, reader) {
                            let writer_node = self.instr_to_node[&writer];
                            let reader_node = self.instr_to_node[&reader];

                            // Add a data flow edge
                            self.dfg.add_edge(
                                writer_node,
                                reader_node,
                                DataFlowValue::Memory(*addr),
                            );
                        }
                    }
                }
            }
        }
    }

    /// Check if there's a path from source to target in the CFG
    fn is_reachable(&self, source: LocalDefId, target: LocalDefId) -> bool {
        // Get the corresponding nodes in the CFG
        if let (Some(source_idx), Some(target_idx)) =
            (self.cfg.get_node_by_instruction(source), self.cfg.get_node_by_instruction(target))
        {
            // Use petgraph's has_path function
            return self.cfg.has_path(source_idx, target_idx);
        }

        false
    }
}
