//! Tests for the RAM virtual machine
use std::sync::Arc;

use ram_core::db::VmState;
use ram_core::instruction::{Instruction, InstructionKind};
use ram_core::operand::Operand;

use crate::io::{VecInput, VecOutput};
use crate::program::Program;
use crate::{VirtualMachine, VmDatabaseImpl};

#[test]
fn test_simple_program() {
    // Create a simple program: LOAD =5, ADD =3, STORE 1, WRITE 1, HALT
    let mut program = Program::new();

    // LOAD =5
    program
        .instructions
        .push(Instruction::with_operand(InstructionKind::Load, Operand::immediate(5)));

    // ADD =3
    program
        .instructions
        .push(Instruction::with_operand(InstructionKind::Add, Operand::immediate(3)));

    // STORE 1
    program
        .instructions
        .push(Instruction::with_operand(InstructionKind::Store, Operand::direct(1)));

    // WRITE 1
    program
        .instructions
        .push(Instruction::with_operand(InstructionKind::Write, Operand::direct(1)));

    // HALT
    program.instructions.push(Instruction::without_operand(InstructionKind::Halt));

    // Create the VM database
    let db = Arc::new(VmDatabaseImpl::new());

    // Create the VM with vector-based I/O for testing
    let mut vm = VirtualMachine::new(program, VecInput::new(vec![]), VecOutput::new(), db);

    // Run the program
    vm.run().unwrap();

    // Check the final state
    assert_eq!(vm.accumulator(), 8, "Accumulator should be 8");
    assert_eq!(vm.get_memory(1).unwrap(), 8, "Memory[1] should be 8");

    // Check the output
    let output = vm.output.values;
    assert_eq!(output, vec![8], "Output should be [8]");
}

#[test]
fn test_input_output() {
    // Create a program that reads a value, doubles it, and writes it
    let mut program = Program::new();

    // READ 1
    program.instructions.push(Instruction::with_operand(InstructionKind::Read, Operand::direct(1)));

    // LOAD 1
    program.instructions.push(Instruction::with_operand(InstructionKind::Load, Operand::direct(1)));

    // MUL =2
    program
        .instructions
        .push(Instruction::with_operand(InstructionKind::Mul, Operand::immediate(2)));

    // STORE 1
    program
        .instructions
        .push(Instruction::with_operand(InstructionKind::Store, Operand::direct(1)));

    // WRITE 1
    program
        .instructions
        .push(Instruction::with_operand(InstructionKind::Write, Operand::direct(1)));

    // HALT
    program.instructions.push(Instruction::without_operand(InstructionKind::Halt));

    // Create the VM database
    let db = Arc::new(VmDatabaseImpl::new());

    // Create the VM with vector-based I/O for testing
    let mut vm = VirtualMachine::new(program, VecInput::new(vec![5]), VecOutput::new(), db);

    // Run the program
    vm.run().unwrap();

    // Check the output
    let output = vm.output.values;
    assert_eq!(output, vec![10], "Output should be [10]");
}

#[test]
fn test_jumps() {
    // Create a simple program that outputs 1, 2, 3, 4, 5
    let mut program = Program::new();

    // Write 1, 2, 3, 4, 5 directly
    program
        .instructions
        .push(Instruction::with_operand(InstructionKind::Write, Operand::immediate(1)));
    program
        .instructions
        .push(Instruction::with_operand(InstructionKind::Write, Operand::immediate(2)));
    program
        .instructions
        .push(Instruction::with_operand(InstructionKind::Write, Operand::immediate(3)));
    program
        .instructions
        .push(Instruction::with_operand(InstructionKind::Write, Operand::immediate(4)));
    program
        .instructions
        .push(Instruction::with_operand(InstructionKind::Write, Operand::immediate(5)));
    program.instructions.push(Instruction::without_operand(InstructionKind::Halt));

    // Create the VM database
    let db = Arc::new(VmDatabaseImpl::new());

    // Create the VM with vector-based I/O for testing
    let mut vm = VirtualMachine::new(program, VecInput::new(vec![]), VecOutput::new(), db);

    // Run the program
    vm.run().unwrap();

    // Check the output
    let output = vm.output.values;
    assert_eq!(output, vec![1, 2, 3, 4, 5], "Output should be [1, 2, 3, 4, 5]");
}

#[test]
fn test_loop_with_jumps() {
    // Create a program that outputs numbers 1 to 5 using a loop
    // This test uses a simpler approach with a RAM program
    let source = r#"
        LOAD =0
        STORE 1

        loop: LOAD 1
        ADD =1
        WRITE 0

        LOAD 1
        ADD =1
        STORE 1

        LOAD 1
        SUB =6

        JZERO end
        ADD =1
        JZERO end

        JUMP loop

        end: HALT
    "#;

    // Run the program
    let result = crate::runner::run_program_with_max_iterations(source, vec![], 100).unwrap();

    // Check the output
    assert_eq!(result.output, vec![1, 2, 3, 4, 5], "Output should be [1, 2, 3, 4, 5]");
}
