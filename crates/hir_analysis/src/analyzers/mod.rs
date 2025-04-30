//! Analyzers for HIR
//!
//! This module provides various analyzers for the HIR, including:
//!
//! - Control flow analysis
//! - Data flow analysis
//! - Instruction validation

pub mod control_flow;
pub mod data_flow;
pub mod instruction_validation;

// Re-export main components
pub use control_flow::{ControlFlowAnalysis, ControlFlowGraph};
pub use data_flow::{DataFlowAnalysis, DataFlowGraph};
pub use instruction_validation::InstructionValidationAnalysis;
