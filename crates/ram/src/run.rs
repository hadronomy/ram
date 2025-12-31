//! Module for running RAM programs

use std::io::Write;
use std::path::Path;
use std::sync::Arc;

use miette::{IntoDiagnostic, Result, miette};
use ram_vm::{VecInput, VecOutput, VirtualMachine, VmDatabaseImpl};

use crate::language;

/// Run a RAM program from a file path
pub fn run_program(
    program_path: &Path,
    input_values: Option<Vec<i64>>,
    _memory_path: Option<&Path>,
) -> Result<()> {
    // Read the program file
    let program_text = std::fs::read_to_string(program_path).into_diagnostic()?;

    // Parse and Validate using the full language pipeline
    // This runs lexer -> parser -> hir lowering -> analysis pipeline
    let (_ast, body, _pipeline, _context, errors) = language::parse_program(&program_text);

    // Check for validation errors
    if !errors.is_empty() {
        // Print all errors
        for error in &errors {
            eprintln!("{:?}", error);
        }
        // Fail the run command
        return Err(miette!("Program validation failed with {} errors", errors.len()));
    }

    // Determine input values: use provided CLI args or prompt interactively
    let values = if let Some(vals) = input_values {
        vals
    } else {
        print!("Input: ");
        std::io::stdout().flush().into_diagnostic()?;
        let mut buffer = String::new();
        std::io::stdin().read_line(&mut buffer).into_diagnostic()?;

        // Replace commas with spaces to allow comma-separated input (e.g. "1, 2, 3")
        buffer
            .replace(',', " ")
            .split_whitespace()
            .map(|token| {
                token.parse::<i64>().map_err(|e| miette!("Invalid number '{}': {}", token, e))
            })
            .collect::<Result<Vec<i64>>>()?
    };

    let input = VecInput::new(values);
    let output = VecOutput::new();

    // Create a new database for VM execution
    let db = Arc::new(VmDatabaseImpl::new());

    // Convert the validated HIR Body to a VM Program
    let program = ram_vm::Program::from_hir(&body, &*db)
        .map_err(|e| miette!("Failed to compile to VM program: {}", e))?;

    // Create a virtual machine
    let mut vm = VirtualMachine::new(program, input, output, db);

    // Run the program
    vm.run().map_err(|e| miette!("Failed to run program: {}", e))?;

    println!("Output: {:?}", vm.output.values);

    Ok(())
}
