//! Module for running RAM programs

use std::io::Write;
use std::path::Path;
use std::sync::Arc;

use miette::{IntoDiagnostic, Result, miette};
use ram_vm::{VecInput, VecOutput, VirtualMachine, VmDatabase, VmDatabaseImpl};

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

    // TODO: Improve io API
    print!("Input: ");
    std::io::stdout().flush().into_diagnostic()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).into_diagnostic()?;

    let input =
        VecInput::new(input.split_whitespace().map(|number| number.parse().unwrap()).collect());
    let output = VecOutput::new();

    // Create a virtual machine
    let mut vm = VirtualMachine::new(program, input, output, db);

    // Run the program
    vm.run().map_err(|e| miette!("Failed to run program: {}", e))?;

    println!("Output: {:?}", vm.output.values);

    Ok(())
}
