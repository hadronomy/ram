//! Core data structures and traits for the RAM virtual machine
//!
//! This crate defines the core data structures and traits for the RAM virtual machine,
//! including instruction definitions and the InstructionDb trait for salsa.

pub mod db;
pub mod error;
pub mod instruction;
pub mod instructions;
pub mod operand;
pub mod registry;

pub use crate::db::InstructionDb;
pub use crate::error::VmError;
pub use crate::instruction::{Instruction, InstructionDefinition, InstructionKind};
pub use crate::instructions::standard_instructions;
pub use crate::operand::{Operand, OperandKind};
pub use crate::registry::InstructionRegistry;
