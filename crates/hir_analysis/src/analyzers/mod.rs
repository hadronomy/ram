//! Analyzers for HIR
//!
//! This module provides various analyzers for the HIR, including:
//!
//! - Control flow analysis
//! - Data flow analysis
//! - Constant propagation analysis
//! - Control flow optimization
//! - Instruction validation

pub mod constant_propagation;
pub mod control_flow;
pub mod control_flow_optimizer;
pub mod data_flow;
pub mod instruction_validation;

// Re-export main components
pub use constant_propagation::{
    BranchTaken, ConstantPropagationAnalysis, ConstantPropagationResult,
};
pub use control_flow::{ControlFlowAnalysis, ControlFlowGraph};
pub use control_flow_optimizer::{ControlFlowOptimizer, OptimizedControlFlowGraph};
pub use data_flow::{DataFlowAnalysis, DataFlowGraph};
pub use instruction_validation::InstructionValidationAnalysis;
