//! Module for running RAM programs

use std::path::Path;
use std::sync::Arc;

use miette::{IntoDiagnostic, Result, miette};
use ram_vm::io::{StdinInput, StdoutOutput};
use ram_vm::{VirtualMachine, VmDatabase, VmDatabaseImpl};

/// Run a RAM program from a file path
pub fn run_program(
    program_path: &Path,
    _input_path: Option<&Path>,
    _memory_path: Option<&Path>,
) -> Result<()> {
    // Create a new database
    let db = Arc::new(VmDatabaseImpl::new());

    // Read the program file
    let program_text = std::fs::read_to_string(program_path).into_diagnostic()?;

    // Parse the program
    let program = db
        .parse_to_vm_program(&program_text)
        .map_err(|e| miette!("Failed to parse program: {}", e))?;

    // Create input and output
    let input = StdinInput::new();
    let output = StdoutOutput::new();

    // Create a virtual machine
    let mut vm = VirtualMachine::new(program, input, output, db);

    // Run the program
    vm.run().map_err(|e| miette!("Failed to run program: {}", e))?;

    Ok(())
}
