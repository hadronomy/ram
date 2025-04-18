//! Virtual machine implementation for executing RAM programs

use std::collections::HashMap;
use std::sync::Arc;

use ram_core::db::VmState;
use ram_core::error::VmError;
use tracing::debug;

use crate::db::{VmDatabase, VmDatabaseImpl};
use crate::io::{Input, Output};
use crate::memory::Memory;
use crate::program::Program;

/// Virtual machine for executing RAM programs
pub struct VirtualMachine<I: Input, O: Output> {
    /// The program being executed
    program: Program,
    /// The memory
    memory: Memory,
    /// The accumulator register
    accumulator: i64,
    /// The program counter
    pc: usize,
    /// The input source
    input: I,
    /// The output sink
    pub output: O,
    /// The database for instruction definitions
    db: Arc<VmDatabaseImpl>,
}

impl<I: Input, O: Output> VirtualMachine<I, O> {
    /// Create a new virtual machine
    pub fn new(program: Program, input: I, output: O, db: Arc<VmDatabaseImpl>) -> Self {
        Self { program, memory: Memory::new(), accumulator: 0, pc: 0, input, output, db }
    }

    /// Reset the virtual machine
    pub fn reset(&mut self) {
        self.memory.clear();
        self.accumulator = 0;
        self.pc = 0;
    }

    /// Execute the program until it halts
    pub fn run(&mut self) -> Result<(), VmError> {
        while self.pc < self.program.len() {
            self.step()?;
        }
        Ok(())
    }

    /// Execute a single instruction
    pub fn step(&mut self) -> Result<(), VmError> {
        if self.pc >= self.program.len() {
            return Err(VmError::InvalidInstruction("Program counter out of bounds".to_string()));
        }

        let instruction = self
            .program
            .get_instruction(self.pc)
            .ok_or_else(|| VmError::InvalidInstruction("Invalid program counter".to_string()))?;

        // Debug log the current instruction
        // Get the instruction name from the registry
        let instr_name = format!("{:?}", instruction.kind);
        let operand_str = match &instruction.operand {
            Some(op) => format!("{:?}={}", op.kind, op.value),
            None => "None".to_string(),
        };
        debug!("Executing instruction at PC={}: {} {}", self.pc, instr_name, operand_str);
        debug!("Current accumulator={}, labels={:?}", self.accumulator, self.program.labels);

        // Increment the PC for the next instruction
        self.pc += 1;

        // Clone the instruction data to avoid borrowing issues
        let kind = instruction.kind.clone();
        let operand = instruction.operand.clone();

        // Get the current state values we need for execution
        let accumulator = self.accumulator;
        let pc = self.pc;
        let memory = &mut self.memory;
        let input = &mut self.input;
        let output = &mut self.output;
        let labels = &self.program.labels;

        // Create a temporary VmState implementation
        let mut temp_state = TempVmState { accumulator, pc, memory, input, output, labels };

        // Execute the instruction
        match self.db.execute_instruction(kind, operand, &mut temp_state) {
            Ok(()) => {
                // Update the VM state from the temporary state
                self.accumulator = temp_state.accumulator;
                self.pc = temp_state.pc;
                debug!("After execution: accumulator={}, PC={}", self.accumulator, self.pc);
                Ok(())
            }
            Err(VmError::ProgramTerminated) => {
                // Special case for HALT instruction
                debug!("Program terminated with HALT instruction");
                self.pc = self.program.len(); // Set PC to end of program
                Ok(())
            }
            Err(e) => {
                debug!("Error executing instruction: {:?}", e);
                Err(e)
            }
        }
    }

    /// Get the current value of the accumulator
    pub fn get_accumulator(&self) -> i64 {
        self.accumulator
    }

    /// Get a reference to the memory
    pub fn get_memory(&self) -> &Memory {
        &self.memory
    }

    /// Get the current program counter
    pub fn get_pc(&self) -> usize {
        self.pc
    }
}

/// Temporary VmState implementation to avoid borrowing issues
struct TempVmState<'a, I: Input, O: Output> {
    accumulator: i64,
    pc: usize,
    memory: &'a mut Memory,
    input: &'a mut I,
    output: &'a mut O,
    labels: &'a HashMap<String, usize>,
}

impl<'a, I: Input, O: Output> VmState for TempVmState<'a, I, O> {
    fn accumulator(&self) -> i64 {
        self.accumulator
    }

    fn set_accumulator(&mut self, value: i64) {
        self.accumulator = value;
    }

    fn get_memory(&self, address: i64) -> Result<i64, VmError> {
        self.memory.get(address)
    }

    fn set_memory(&mut self, address: i64, value: i64) -> Result<(), VmError> {
        self.memory.set(address, value)
    }

    fn program_counter(&self) -> usize {
        self.pc
    }

    fn set_program_counter(&mut self, pc: usize) {
        self.pc = pc;
    }

    fn read_input(&mut self) -> Result<i64, VmError> {
        self.input.read()
    }

    fn write_output(&mut self, value: i64) -> Result<(), VmError> {
        self.output.write(value)
    }

    fn resolve_label(&self, label: &str) -> Result<usize, VmError> {
        self.labels
            .get(label)
            .copied()
            .ok_or_else(|| VmError::InvalidInstruction(format!("Unknown label: {}", label)))
    }
}

impl<I: Input, O: Output> VmState for VirtualMachine<I, O> {
    fn accumulator(&self) -> i64 {
        self.accumulator
    }

    fn set_accumulator(&mut self, value: i64) {
        self.accumulator = value;
    }

    fn get_memory(&self, address: i64) -> Result<i64, VmError> {
        self.memory.get(address)
    }

    fn set_memory(&mut self, address: i64, value: i64) -> Result<(), VmError> {
        self.memory.set(address, value)
    }

    fn program_counter(&self) -> usize {
        self.pc
    }

    fn set_program_counter(&mut self, pc: usize) {
        self.pc = pc;
    }

    fn read_input(&mut self) -> Result<i64, VmError> {
        self.input.read()
    }

    fn write_output(&mut self, value: i64) -> Result<(), VmError> {
        self.output.write(value)
    }

    fn resolve_label(&self, label: &str) -> Result<usize, VmError> {
        self.program.resolve_label(label)
    }
}
