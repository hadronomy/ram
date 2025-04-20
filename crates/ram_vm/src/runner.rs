//! Convenience functions for running RAM programs
//!
//! This module provides convenience functions for creating and running RAM programs.

use std::sync::Arc;

use ram_core::db::VmState;
use ram_core::error::VmError;

use crate::db::{VmDatabase, VmDatabaseImpl};
use crate::io::{VecInput, VecOutput};
use crate::vm::VirtualMachine;

/// Result of running a program
#[derive(Debug)]
pub struct RunResult {
    /// The final value of the accumulator
    pub accumulator: i64,
    /// The output values
    pub output: Vec<i64>,
    /// The number of steps executed
    pub steps: usize,
}

/// Run a program with the given source code and input values
pub fn run_program(source: &str, input: Vec<i64>) -> Result<RunResult, VmError> {
    // Create a database
    let db = Arc::new(VmDatabaseImpl::new());

    // Parse the source code to a program
    let program = db.parse_to_vm_program(source)?;

    // Create input and output
    let input = VecInput::new(input);
    let output = VecOutput::new();

    // Create and run the virtual machine
    let mut vm = VirtualMachine::new(program, input, output, db);
    vm.run()?;

    // Create the result
    let result = RunResult {
        accumulator: vm.get_accumulator(),
        output: vm.output.values.clone(),
        steps: vm.get_pc(),
    };

    Ok(result)
}

/// Run a program with the given source code, input values, and initial memory values
pub fn run_program_with_memory(
    source: &str,
    input: Vec<i64>,
    memory: Vec<(i64, i64)>,
) -> Result<RunResult, VmError> {
    // Create a database
    let db = Arc::new(VmDatabaseImpl::new());

    // Parse the source code to a program
    let program = db.parse_to_vm_program(source)?;

    // Create input and output
    let input = VecInput::new(input);
    let output = VecOutput::new();

    // Create and run the virtual machine
    let mut vm = VirtualMachine::new(program, input, output, db);

    // Set initial memory values
    for (address, value) in memory {
        vm.set_memory(address, value)?;
    }

    vm.run()?;

    // Create the result
    let result = RunResult {
        accumulator: vm.get_accumulator(),
        output: vm.output.values.clone(),
        steps: vm.get_pc(),
    };

    Ok(result)
}

/// Run a program with the given source code, input values, and maximum number of iterations
pub fn run_program_with_max_iterations(
    source: &str,
    input: Vec<i64>,
    max_iterations: usize,
) -> Result<RunResult, VmError> {
    // Create a database
    let db = Arc::new(VmDatabaseImpl::new());

    // Parse the source code to a program
    let program = db.parse_to_vm_program(source)?;

    // Create input and output
    let input = VecInput::new(input);
    let output = VecOutput::new();

    // Create and run the virtual machine
    let mut vm = VirtualMachine::new(program, input, output, db);

    // Run with max iterations
    vm.run_with_max_iterations(max_iterations)?;

    // Create the result
    let result = RunResult {
        accumulator: vm.get_accumulator(),
        output: vm.output.values.clone(),
        steps: vm.get_pc(),
    };

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_program() {
        // A simple program that adds two numbers
        let source = r#"
            LOAD =5
            ADD =10
            STORE 0
            WRITE 0
            HALT
        "#;

        let result = run_program(source, vec![]).unwrap();

        assert_eq!(result.output, vec![15]);
        assert_eq!(result.accumulator, 15);
    }

    #[test]
    fn test_run_program_with_memory() {
        // A program that adds two numbers from memory
        let source = r#"
            LOAD 0
            ADD 1
            WRITE 0
            STORE 0
            HALT
        "#;

        let result = run_program_with_memory(source, vec![], vec![(0, 10), (1, 20)]).unwrap();

        assert_eq!(result.output, vec![30]);
        assert_eq!(result.accumulator, 30);
    }

    #[test]
    fn test_run_program_with_input() {
        // A program that reads a number and outputs its square
        let source = r#"
            READ 0
            LOAD 0
            MUL 0
            WRITE 0
            STORE 0
            HALT
        "#;

        let result = run_program(source, vec![5]).unwrap();

        assert_eq!(result.output, vec![25]);
        assert_eq!(result.accumulator, 25);
    }
}
