//! Database interface for instruction definitions

use std::sync::Arc;

use salsa::Database;

use crate::error::VmError;
use crate::instruction::{InstructionDefinition, InstructionKind};
use crate::operand::Operand;

/// Database trait for instruction definitions
#[salsa::db]
pub trait InstructionDb: Database {
    /// Execute an instruction with the given operand
    fn execute_instruction(
        &self,
        instruction: InstructionKind,
        operand: Option<Operand>,
        vm_state: &mut dyn VmState,
    ) -> Result<(), VmError>;

    /// Get the instruction for a given name
    fn get_instruction(&self, name: &str) -> Option<InstructionKind>;

    /// Register a custom instruction
    fn register_instruction(&mut self, name: &str, definition: Arc<dyn InstructionDefinition>);

    /// Get the instruction definition for a given kind
    fn get_instruction_definition(
        &self,
        kind: &InstructionKind,
    ) -> Option<Arc<dyn InstructionDefinition>>;

    /// Validate an instruction with the given operand
    fn validate_instruction(
        &self,
        instruction: InstructionKind,
        operand: Option<&Operand>,
    ) -> Result<(), VmError>;
}

/// Trait for VM state that can be modified by instructions
pub trait VmState {
    /// Get the value of the accumulator
    fn accumulator(&self) -> i64;

    /// Set the value of the accumulator
    fn set_accumulator(&mut self, value: i64);

    /// Get a value from the Register File
    fn get_register(&self, index: i64) -> Result<i64, VmError>;

    /// Set a value in the Register File
    fn set_register(&mut self, index: i64, value: i64) -> Result<(), VmError>;

    /// Get a value from Heap Memory
    fn get_memory(&self, address: i64) -> Result<i64, VmError>;

    /// Set a value in Heap Memory
    fn set_memory(&mut self, address: i64, value: i64) -> Result<(), VmError>;

    /// Get the program counter
    fn program_counter(&self) -> usize;

    /// Set the program counter
    fn set_program_counter(&mut self, pc: usize);

    /// Read a value from input
    fn read_input(&mut self) -> Result<i64, VmError>;

    /// Write a value to output
    fn write_output(&mut self, value: i64) -> Result<(), VmError>;

    /// Resolve a label to a program counter value
    fn resolve_label(&self, label: &str) -> Result<usize, VmError>;
}
