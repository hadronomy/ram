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
    /// Flag indicating if the VM is running
    running: bool,
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
        Self {
            program,
            memory: Memory::new(),
            accumulator: 0,
            pc: 0,
            running: true,
            input,
            output,
            db,
        }
    }

    /// Create a new virtual machine with a builder pattern
    pub fn builder(
        program: Program,
        input: I,
        output: O,
        db: Arc<VmDatabaseImpl>,
    ) -> VirtualMachineBuilder<I, O> {
        VirtualMachineBuilder::new(program, input, output, db)
    }

    /// Reset the virtual machine
    pub fn reset(&mut self) {
        self.memory.clear();
        self.accumulator = 0;
        self.pc = 0;
        self.running = true;
    }

    /// Execute the program until it halts
    pub fn run(&mut self) -> Result<(), VmError> {
        while self.running && self.pc < self.program.len() {
            self.step()?;
        }
        Ok(())
    }

    /// Execute the program until it halts or reaches the maximum number of iterations
    pub fn run_with_max_iterations(&mut self, max_iterations: usize) -> Result<(), VmError> {
        let mut iterations = 0;
        while self.running && self.pc < self.program.len() && iterations < max_iterations {
            self.step()?;
            iterations += 1;
        }

        if iterations >= max_iterations && self.running && self.pc < self.program.len() {
            return Err(VmError::InvalidInstruction(format!(
                "Program exceeded maximum iterations ({})",
                max_iterations
            )));
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

        // Get the instruction definition
        let definition = self
            .db
            .get_instruction_definition(&kind)
            .ok_or_else(|| VmError::InvalidInstruction(format!("Unknown instruction: {}", kind)))?;

        // Execute the instruction
        match definition.execute(operand.as_ref(), self) {
            Ok(()) => {
                debug!("After execution: accumulator={}, PC={}", self.accumulator, self.pc);
                Ok(())
            }
            Err(VmError::ProgramTerminated) => {
                // Special case for HALT instruction
                debug!("Program terminated with HALT instruction");
                self.running = false; // Set running flag to false
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

    /// Check if the VM is running
    pub fn is_running(&self) -> bool {
        self.running
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
        debug!("Getting memory at address {}", address);
        // Special case: address 0 refers to the accumulator
        if address == 0 {
            debug!("Reading from accumulator: {}", self.accumulator);
            Ok(self.accumulator)
        } else {
            self.memory.get(address)
        }
    }

    fn set_memory(&mut self, address: i64, value: i64) -> Result<(), VmError> {
        // Special case: address 0 refers to the accumulator
        if address == 0 {
            debug!("Setting accumulator to {}", value);
            self.accumulator = value;
            Ok(())
        } else {
            self.memory.set(address, value)
        }
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

/// Builder for creating and configuring a virtual machine
pub struct VirtualMachineBuilder<I: Input, O: Output> {
    /// The program to execute
    program: Program,
    /// The input source
    input: I,
    /// The output sink
    output: O,
    /// The database for instruction definitions
    db: Arc<VmDatabaseImpl>,
    /// Initial memory values
    initial_memory: HashMap<i64, i64>,
    /// Initial accumulator value
    initial_accumulator: i64,
    /// Maximum number of iterations
    max_iterations: Option<usize>,
}

impl<I: Input, O: Output> VirtualMachineBuilder<I, O> {
    /// Create a new virtual machine builder
    pub fn new(program: Program, input: I, output: O, db: Arc<VmDatabaseImpl>) -> Self {
        Self {
            program,
            input,
            output,
            db,
            initial_memory: HashMap::new(),
            initial_accumulator: 0,
            max_iterations: None,
        }
    }

    /// Set the initial value of the accumulator
    pub fn with_accumulator(mut self, value: i64) -> Self {
        self.initial_accumulator = value;
        self
    }

    /// Set the initial value of a memory location
    pub fn with_memory(mut self, address: i64, value: i64) -> Self {
        self.initial_memory.insert(address, value);
        self
    }

    /// Set the initial values of multiple memory locations
    pub fn with_memory_values(mut self, values: impl IntoIterator<Item = (i64, i64)>) -> Self {
        for (address, value) in values {
            self.initial_memory.insert(address, value);
        }
        self
    }

    /// Set the maximum number of iterations
    pub fn with_max_iterations(mut self, max_iterations: usize) -> Self {
        self.max_iterations = Some(max_iterations);
        self
    }

    /// Build the virtual machine
    pub fn build(self) -> VirtualMachine<I, O> {
        let mut vm = VirtualMachine::new(self.program, self.input, self.output, self.db);

        // Set the initial accumulator value
        vm.accumulator = self.initial_accumulator;

        // Set the initial memory values
        for (address, value) in self.initial_memory {
            // Ignore errors when setting initial memory values
            let _ = vm.memory.set(address, value);
        }

        vm
    }

    /// Build and run the virtual machine
    pub fn run(self) -> Result<VirtualMachine<I, O>, VmError> {
        // Store the max_iterations before self is moved
        let max_iterations = self.max_iterations;

        let mut vm = self.build();

        if let Some(max_iterations) = max_iterations {
            vm.run_with_max_iterations(max_iterations)?
        } else {
            vm.run()?
        }

        Ok(vm)
    }
}
