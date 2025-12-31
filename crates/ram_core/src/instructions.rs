//! Standard instruction implementations for the RAM virtual machine

use std::sync::Arc;

use tracing::debug;

use crate::db::VmState;
use crate::error::VmError;
use crate::instruction::{InstructionDefinition, InstructionKind};
use crate::operand::{Operand, OperandKind};
use crate::operand_resolver::{DefaultOperandResolver, OperandResolver, StoreTarget};
use crate::registry::InstructionRegistry;

/// LOAD instruction implementation
#[derive(Debug, Clone)]
pub struct LoadInstruction;

impl InstructionDefinition for LoadInstruction {
    fn name(&self) -> &str {
        "LOAD"
    }

    fn requires_operand(&self) -> bool {
        true
    }

    fn allowed_operand_kinds(&self) -> &[OperandKind] {
        &[OperandKind::Direct, OperandKind::Indirect, OperandKind::Immediate, OperandKind::Indexed]
    }

    fn execute(
        &self,
        operand: Option<&Operand>,
        vm_state: &mut dyn VmState,
    ) -> Result<(), VmError> {
        let operand = operand
            .ok_or_else(|| VmError::InvalidOperand("LOAD requires an operand".to_string()))?;

        // Use the operand resolver to get the value
        let resolver = DefaultOperandResolver;
        let value = resolver.resolve_operand_value(operand, vm_state)?;

        vm_state.set_accumulator(value);
        Ok(())
    }
}

/// STORE instruction implementation
#[derive(Debug, Clone)]
pub struct StoreInstruction;

impl InstructionDefinition for StoreInstruction {
    fn name(&self) -> &str {
        "STORE"
    }

    fn requires_operand(&self) -> bool {
        true
    }

    fn allowed_operand_kinds(&self) -> &[OperandKind] {
        &[OperandKind::Direct, OperandKind::Indirect, OperandKind::Indexed]
    }

    fn execute(
        &self,
        operand: Option<&Operand>,
        vm_state: &mut dyn VmState,
    ) -> Result<(), VmError> {
        let operand = operand
            .ok_or_else(|| VmError::InvalidOperand("STORE requires an operand".to_string()))?;

        // Get the accumulator value
        let acc = vm_state.accumulator();

        // Use the operand resolver to get the target (Register vs Memory) and address
        let resolver = DefaultOperandResolver;
        let (target_type, address) = resolver.resolve_store_address(operand, vm_state)?;

        match target_type {
            StoreTarget::Register => vm_state.set_register(address, acc)?,
            StoreTarget::Memory => vm_state.set_memory(address, acc)?,
            StoreTarget::Accumulator => vm_state.set_accumulator(acc),
        }

        Ok(())
    }
}

/// ADD instruction implementation
#[derive(Debug, Clone)]
pub struct AddInstruction;

impl InstructionDefinition for AddInstruction {
    fn name(&self) -> &str {
        "ADD"
    }

    fn requires_operand(&self) -> bool {
        true
    }

    fn allowed_operand_kinds(&self) -> &[OperandKind] {
        &[OperandKind::Direct, OperandKind::Indirect, OperandKind::Immediate, OperandKind::Indexed]
    }

    fn execute(
        &self,
        operand: Option<&Operand>,
        vm_state: &mut dyn VmState,
    ) -> Result<(), VmError> {
        let operand = operand
            .ok_or_else(|| VmError::InvalidOperand("ADD requires an operand".to_string()))?;

        // Use the operand resolver to get the value
        let resolver = DefaultOperandResolver;
        let value = resolver.resolve_operand_value(operand, vm_state)?;

        // Add the value to the accumulator
        let acc = vm_state.accumulator();
        vm_state.set_accumulator(acc + value);

        Ok(())
    }
}

/// SUB instruction implementation
#[derive(Debug, Clone)]
pub struct SubInstruction;

impl InstructionDefinition for SubInstruction {
    fn name(&self) -> &str {
        "SUB"
    }

    fn requires_operand(&self) -> bool {
        true
    }

    fn allowed_operand_kinds(&self) -> &[OperandKind] {
        &[OperandKind::Direct, OperandKind::Indirect, OperandKind::Immediate, OperandKind::Indexed]
    }

    fn execute(
        &self,
        operand: Option<&Operand>,
        vm_state: &mut dyn VmState,
    ) -> Result<(), VmError> {
        let operand = operand
            .ok_or_else(|| VmError::InvalidOperand("SUB requires an operand".to_string()))?;

        // Use the operand resolver to get the value
        let resolver = DefaultOperandResolver;
        let value = resolver.resolve_operand_value(operand, vm_state)?;

        // Subtract the value from the accumulator
        let acc = vm_state.accumulator();
        vm_state.set_accumulator(acc - value);

        Ok(())
    }
}

/// MUL instruction implementation
#[derive(Debug, Clone)]
pub struct MulInstruction;

impl InstructionDefinition for MulInstruction {
    fn name(&self) -> &str {
        "MUL"
    }

    fn requires_operand(&self) -> bool {
        true
    }

    fn allowed_operand_kinds(&self) -> &[OperandKind] {
        &[OperandKind::Direct, OperandKind::Indirect, OperandKind::Immediate, OperandKind::Indexed]
    }

    fn execute(
        &self,
        operand: Option<&Operand>,
        vm_state: &mut dyn VmState,
    ) -> Result<(), VmError> {
        let operand = operand
            .ok_or_else(|| VmError::InvalidOperand("MUL requires an operand".to_string()))?;

        // Use the operand resolver to get the value
        let resolver = DefaultOperandResolver;
        let value = resolver.resolve_operand_value(operand, vm_state)?;

        // Multiply the accumulator by the value
        let acc = vm_state.accumulator();
        vm_state.set_accumulator(acc * value);

        Ok(())
    }
}

/// DIV instruction implementation
#[derive(Debug, Clone)]
pub struct DivInstruction;

impl InstructionDefinition for DivInstruction {
    fn name(&self) -> &str {
        "DIV"
    }

    fn requires_operand(&self) -> bool {
        true
    }

    fn allowed_operand_kinds(&self) -> &[OperandKind] {
        &[OperandKind::Direct, OperandKind::Indirect, OperandKind::Immediate, OperandKind::Indexed]
    }

    fn execute(
        &self,
        operand: Option<&Operand>,
        vm_state: &mut dyn VmState,
    ) -> Result<(), VmError> {
        let operand = operand
            .ok_or_else(|| VmError::InvalidOperand("DIV requires an operand".to_string()))?;

        // Use the operand resolver to get the value
        let resolver = DefaultOperandResolver;
        let value = resolver.resolve_operand_value(operand, vm_state)?;

        // Check for division by zero
        if value == 0 {
            return Err(VmError::DivisionByZero);
        }

        // Divide the accumulator by the value
        let acc = vm_state.accumulator();
        vm_state.set_accumulator(acc / value);

        Ok(())
    }
}

/// JUMP instruction implementation
#[derive(Debug, Clone)]
pub struct JumpInstruction;

impl InstructionDefinition for JumpInstruction {
    fn name(&self) -> &str {
        "JUMP"
    }

    fn requires_operand(&self) -> bool {
        true
    }

    fn allowed_operand_kinds(&self) -> &[OperandKind] {
        &[OperandKind::Direct]
    }

    fn execute(
        &self,
        operand: Option<&Operand>,
        vm_state: &mut dyn VmState,
    ) -> Result<(), VmError> {
        let operand = operand
            .ok_or_else(|| VmError::InvalidOperand("JUMP requires an operand".to_string()))?;

        // Use the operand resolver to get the jump target
        let resolver = DefaultOperandResolver;
        let target = resolver.resolve_jump_target(operand, vm_state)?;

        // Set the program counter to the jump target
        debug!("JUMP: Jumping to target {}", target);
        vm_state.set_program_counter(target);

        Ok(())
    }
}

/// JGTZ instruction implementation
#[derive(Debug, Clone)]
pub struct JumpGtzInstruction;

impl InstructionDefinition for JumpGtzInstruction {
    fn name(&self) -> &str {
        "JGTZ"
    }

    fn requires_operand(&self) -> bool {
        true
    }

    fn allowed_operand_kinds(&self) -> &[OperandKind] {
        &[OperandKind::Direct]
    }

    fn execute(
        &self,
        operand: Option<&Operand>,
        vm_state: &mut dyn VmState,
    ) -> Result<(), VmError> {
        let operand = operand
            .ok_or_else(|| VmError::InvalidOperand("JGTZ requires an operand".to_string()))?;

        // Only jump if the accumulator is greater than zero
        if vm_state.accumulator() > 0 {
            // Use the operand resolver to get the jump target
            let resolver = DefaultOperandResolver;
            let target = resolver.resolve_jump_target(operand, vm_state)?;

            // Set the program counter to the jump target
            debug!("JGTZ: Jumping to target {}", target);
            vm_state.set_program_counter(target);
        } else {
            debug!("JGTZ: Accumulator <= 0, not jumping");
        }

        Ok(())
    }
}

/// JZERO instruction implementation
#[derive(Debug, Clone)]
pub struct JumpZeroInstruction;

impl InstructionDefinition for JumpZeroInstruction {
    fn name(&self) -> &str {
        "JZERO"
    }

    fn requires_operand(&self) -> bool {
        true
    }

    fn allowed_operand_kinds(&self) -> &[OperandKind] {
        &[OperandKind::Direct]
    }

    fn execute(
        &self,
        operand: Option<&Operand>,
        vm_state: &mut dyn VmState,
    ) -> Result<(), VmError> {
        let operand = operand
            .ok_or_else(|| VmError::InvalidOperand("JZERO requires an operand".to_string()))?;

        // Only jump if the accumulator is zero
        if vm_state.accumulator() == 0 {
            // Use the operand resolver to get the jump target
            let resolver = DefaultOperandResolver;
            let target = resolver.resolve_jump_target(operand, vm_state)?;

            // Set the program counter to the jump target
            debug!("JZERO: Jumping to target {}", target);
            vm_state.set_program_counter(target);
        } else {
            debug!("JZERO: Accumulator != 0, not jumping");
        }

        Ok(())
    }
}

/// READ instruction implementation
#[derive(Debug, Clone)]
pub struct ReadInstruction;

impl InstructionDefinition for ReadInstruction {
    fn name(&self) -> &str {
        "READ"
    }

    fn requires_operand(&self) -> bool {
        true
    }

    fn allowed_operand_kinds(&self) -> &[OperandKind] {
        &[OperandKind::Direct, OperandKind::Indirect, OperandKind::Indexed]
    }

    fn execute(
        &self,
        operand: Option<&Operand>,
        vm_state: &mut dyn VmState,
    ) -> Result<(), VmError> {
        let operand = operand
            .ok_or_else(|| VmError::InvalidOperand("READ requires an operand".to_string()))?;

        // Read input from the VM state
        let value = vm_state.read_input()?;

        // Use the operand resolver to get the target
        let resolver = DefaultOperandResolver;
        let (target_type, address) = resolver.resolve_store_address(operand, vm_state)?;

        match target_type {
            StoreTarget::Register => vm_state.set_register(address, value)?,
            StoreTarget::Memory => vm_state.set_memory(address, value)?,
            StoreTarget::Accumulator => vm_state.set_accumulator(value),
        }

        Ok(())
    }
}

/// WRITE instruction implementation
#[derive(Debug, Clone)]
pub struct WriteInstruction;

impl InstructionDefinition for WriteInstruction {
    fn name(&self) -> &str {
        "WRITE"
    }

    fn requires_operand(&self) -> bool {
        true
    }

    fn allowed_operand_kinds(&self) -> &[OperandKind] {
        &[OperandKind::Direct, OperandKind::Indirect, OperandKind::Immediate, OperandKind::Indexed]
    }

    fn execute(
        &self,
        operand: Option<&Operand>,
        vm_state: &mut dyn VmState,
    ) -> Result<(), VmError> {
        let operand = operand
            .ok_or_else(|| VmError::InvalidOperand("WRITE requires an operand".to_string()))?;

        // Use the operand resolver to get the value
        let resolver = DefaultOperandResolver;
        let value = resolver.resolve_operand_value(operand, vm_state)?;
        debug!("WRITE: Writing value {}", value);

        // Write the value to the output
        vm_state.write_output(value)?;

        Ok(())
    }
}

/// HALT instruction implementation
#[derive(Debug, Clone)]
pub struct HaltInstruction;

impl InstructionDefinition for HaltInstruction {
    fn name(&self) -> &str {
        "HALT"
    }

    fn requires_operand(&self) -> bool {
        false
    }

    fn allowed_operand_kinds(&self) -> &[OperandKind] {
        &[]
    }

    fn execute(
        &self,
        _operand: Option<&Operand>,
        _vm_state: &mut dyn VmState,
    ) -> Result<(), VmError> {
        Err(VmError::ProgramTerminated)
    }
}

/// Create a registry with all standard instructions
pub fn standard_instructions() -> InstructionRegistry {
    let mut registry = InstructionRegistry::new();

    // Register standard instructions
    registry.register(InstructionKind::Load, Arc::new(LoadInstruction));
    registry.register(InstructionKind::Store, Arc::new(StoreInstruction));
    registry.register(InstructionKind::Add, Arc::new(AddInstruction));
    registry.register(InstructionKind::Sub, Arc::new(SubInstruction));
    registry.register(InstructionKind::Mul, Arc::new(MulInstruction));
    registry.register(InstructionKind::Div, Arc::new(DivInstruction));
    registry.register(InstructionKind::Jump, Arc::new(JumpInstruction));
    registry.register(InstructionKind::JumpGtz, Arc::new(JumpGtzInstruction));
    registry.register(InstructionKind::JumpZero, Arc::new(JumpZeroInstruction));
    registry.register(InstructionKind::Read, Arc::new(ReadInstruction));
    registry.register(InstructionKind::Write, Arc::new(WriteInstruction));
    registry.register(InstructionKind::Halt, Arc::new(HaltInstruction));

    registry
}
