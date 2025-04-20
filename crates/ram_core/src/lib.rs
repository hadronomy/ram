//! Core data structures and traits for the RAM virtual machine
//!
//! This crate defines the core data structures and traits for the RAM virtual machine,
//! including instruction definitions and the InstructionDb trait for salsa.
//!
//! The crate also provides a plugin system for extending the VM with custom instructions.

pub mod db;
pub mod error;
pub mod instruction;
pub mod instruction_set;
pub mod instructions;
pub mod operand;
pub mod operand_resolver;
pub mod plugin;
pub mod registry;

#[cfg(feature = "examples")]
pub mod examples;

pub use crate::db::InstructionDb;
pub use crate::error::VmError;
pub use crate::instruction::{
    Instruction, InstructionDefinition, InstructionInfo, InstructionKind,
};
pub use crate::instruction_set::{
    INSTRUCTION_SET_REGISTRY, InstructionSet, InstructionSetRegistry, STANDARD_INSTRUCTION_SET,
};
pub use crate::instructions::standard_instructions;
pub use crate::operand::{Operand, OperandKind, OperandValue};
pub use crate::operand_resolver::{
    DefaultOperandResolver, OperandResolver, resolve_jump_target, resolve_operand_value,
    resolve_store_address,
};
pub use crate::plugin::{InstructionBuilder, PluginManager, RamPlugin};
pub use crate::registry::InstructionRegistry;

#[cfg(test)]
mod tests {
    pub mod instruction_info_tests;
    pub mod instruction_set_tests;
}
