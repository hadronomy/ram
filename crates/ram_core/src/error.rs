//! Error types for the RAM virtual machine

use miette::*;
use thiserror::Error;

/// Errors that can occur during VM execution
#[derive(Debug, Diagnostic, Error)]
pub enum VmError {
    /// Invalid operand for the instruction
    #[error("Invalid operand for instruction: {0}")]
    InvalidOperand(String),

    /// Invalid memory access
    #[error("Invalid memory access: {0}")]
    InvalidMemoryAccess(String),

    /// Division by zero
    #[error("Division by zero")]
    DivisionByZero,

    /// Invalid instruction
    #[error("Invalid instruction: {0}")]
    InvalidInstruction(String),

    #[error("Parse error: {0}")]
    #[diagnostic(code(ram::parse_error))]
    ParseError(#[from] ram_error::Report),

    /// IO error
    #[error("IO error: {0}")]
    IoError(String),

    /// Program terminated
    #[error("Program terminated")]
    ProgramTerminated,

    #[error("Unknown error: {0}")]
    UnknownError(String),
}
