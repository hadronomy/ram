//! Standard instruction implementations for the RAM virtual machine

use tracing::debug;

use crate::db::VmState;
use crate::error::VmError;
use crate::instruction::InstructionDefinition;
use crate::operand::{self, Operand, OperandKind};

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
        &[OperandKind::Direct, OperandKind::Indirect, OperandKind::Immediate]
    }

    fn execute(
        &self,
        operand: Option<&Operand>,
        vm_state: &mut dyn VmState,
    ) -> Result<(), VmError> {
        let operand = operand
            .ok_or_else(|| VmError::InvalidOperand("LOAD requires an operand".to_string()))?;
        let value = match operand.kind {
            OperandKind::Direct => {
                match &operand.value {
                    operand::OperandValue::Number(num) => vm_state.get_memory(*num)?,
                    operand::OperandValue::String(s) => {
                        // Try to resolve the string as a label
                        let address = vm_state.resolve_label(s)?;
                        vm_state.get_memory(address as i64)?
                    }
                }
            }
            OperandKind::Indirect => {
                match &operand.value {
                    operand::OperandValue::Number(num) => {
                        let address = vm_state.get_memory(*num)?;
                        vm_state.get_memory(address)?
                    }
                    operand::OperandValue::String(s) => {
                        // Try to resolve the string as a label
                        let address = vm_state.resolve_label(s)?;
                        let indirect_address = vm_state.get_memory(address as i64)?;
                        vm_state.get_memory(indirect_address)?
                    }
                }
            }
            OperandKind::Immediate => {
                match &operand.value {
                    operand::OperandValue::Number(num) => *num,
                    operand::OperandValue::String(s) => {
                        // For immediate addressing with a string, try to parse it as a number
                        s.parse::<i64>().map_err(|_| {
                            VmError::InvalidOperand(format!(
                                "Cannot use string '{}' as immediate value",
                                s
                            ))
                        })?
                    }
                }
            }
        };
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
        &[OperandKind::Direct, OperandKind::Indirect]
    }

    fn execute(
        &self,
        operand: Option<&Operand>,
        vm_state: &mut dyn VmState,
    ) -> Result<(), VmError> {
        let operand = operand
            .ok_or_else(|| VmError::InvalidOperand("STORE requires an operand".to_string()))?;
        let acc = vm_state.accumulator();
        match operand.kind {
            OperandKind::Direct => {
                match &operand.value {
                    operand::OperandValue::Number(num) => vm_state.set_memory(*num, acc)?,
                    operand::OperandValue::String(s) => {
                        // Try to resolve the string as a label
                        let address = vm_state.resolve_label(s)?;
                        vm_state.set_memory(address as i64, acc)?
                    }
                }
            }
            OperandKind::Indirect => {
                match &operand.value {
                    operand::OperandValue::Number(num) => {
                        let address = vm_state.get_memory(*num)?;
                        vm_state.set_memory(address, acc)?
                    }
                    operand::OperandValue::String(s) => {
                        // Try to resolve the string as a label
                        let address = vm_state.resolve_label(s)?;
                        let indirect_address = vm_state.get_memory(address as i64)?;
                        vm_state.set_memory(indirect_address, acc)?
                    }
                }
            }
            OperandKind::Immediate => {
                return Err(VmError::InvalidOperand(
                    "STORE cannot use immediate addressing".to_string(),
                ));
            }
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
        &[OperandKind::Direct, OperandKind::Indirect, OperandKind::Immediate]
    }

    fn execute(
        &self,
        operand: Option<&Operand>,
        vm_state: &mut dyn VmState,
    ) -> Result<(), VmError> {
        let operand = operand
            .ok_or_else(|| VmError::InvalidOperand("ADD requires an operand".to_string()))?;
        let value = match operand.kind {
            OperandKind::Direct => {
                match &operand.value {
                    operand::OperandValue::Number(num) => vm_state.get_memory(*num)?,
                    operand::OperandValue::String(s) => {
                        // Try to resolve the string as a label
                        let address = vm_state.resolve_label(s)?;
                        vm_state.get_memory(address as i64)?
                    }
                }
            }
            OperandKind::Indirect => {
                match &operand.value {
                    operand::OperandValue::Number(num) => {
                        let address = vm_state.get_memory(*num)?;
                        vm_state.get_memory(address)?
                    }
                    operand::OperandValue::String(s) => {
                        // Try to resolve the string as a label
                        let address = vm_state.resolve_label(s)?;
                        let indirect_address = vm_state.get_memory(address as i64)?;
                        vm_state.get_memory(indirect_address)?
                    }
                }
            }
            OperandKind::Immediate => {
                match &operand.value {
                    operand::OperandValue::Number(num) => *num,
                    operand::OperandValue::String(s) => {
                        // For immediate addressing with a string, try to parse it as a number
                        s.parse::<i64>().map_err(|_| {
                            VmError::InvalidOperand(format!(
                                "Cannot use string '{}' as immediate value",
                                s
                            ))
                        })?
                    }
                }
            }
        };
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
        &[OperandKind::Direct, OperandKind::Indirect, OperandKind::Immediate]
    }

    fn execute(
        &self,
        operand: Option<&Operand>,
        vm_state: &mut dyn VmState,
    ) -> Result<(), VmError> {
        let operand = operand
            .ok_or_else(|| VmError::InvalidOperand("SUB requires an operand".to_string()))?;
        let value = match operand.kind {
            OperandKind::Direct => {
                match &operand.value {
                    operand::OperandValue::Number(num) => vm_state.get_memory(*num)?,
                    operand::OperandValue::String(s) => {
                        // Try to resolve the string as a label
                        let address = vm_state.resolve_label(s)?;
                        vm_state.get_memory(address as i64)?
                    }
                }
            }
            OperandKind::Indirect => {
                match &operand.value {
                    operand::OperandValue::Number(num) => {
                        let address = vm_state.get_memory(*num)?;
                        vm_state.get_memory(address)?
                    }
                    operand::OperandValue::String(s) => {
                        // Try to resolve the string as a label
                        let address = vm_state.resolve_label(s)?;
                        let indirect_address = vm_state.get_memory(address as i64)?;
                        vm_state.get_memory(indirect_address)?
                    }
                }
            }
            OperandKind::Immediate => {
                match &operand.value {
                    operand::OperandValue::Number(num) => *num,
                    operand::OperandValue::String(s) => {
                        // For immediate addressing with a string, try to parse it as a number
                        s.parse::<i64>().map_err(|_| {
                            VmError::InvalidOperand(format!(
                                "Cannot use string '{}' as immediate value",
                                s
                            ))
                        })?
                    }
                }
            }
        };
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
        &[OperandKind::Direct, OperandKind::Indirect, OperandKind::Immediate]
    }

    fn execute(
        &self,
        operand: Option<&Operand>,
        vm_state: &mut dyn VmState,
    ) -> Result<(), VmError> {
        let operand = operand
            .ok_or_else(|| VmError::InvalidOperand("MUL requires an operand".to_string()))?;
        let value = match operand.kind {
            OperandKind::Direct => {
                match &operand.value {
                    operand::OperandValue::Number(num) => vm_state.get_memory(*num)?,
                    operand::OperandValue::String(s) => {
                        // Try to resolve the string as a label
                        let address = vm_state.resolve_label(s)?;
                        vm_state.get_memory(address as i64)?
                    }
                }
            }
            OperandKind::Indirect => {
                match &operand.value {
                    operand::OperandValue::Number(num) => {
                        let address = vm_state.get_memory(*num)?;
                        vm_state.get_memory(address)?
                    }
                    operand::OperandValue::String(s) => {
                        // Try to resolve the string as a label
                        let address = vm_state.resolve_label(s)?;
                        let indirect_address = vm_state.get_memory(address as i64)?;
                        vm_state.get_memory(indirect_address)?
                    }
                }
            }
            OperandKind::Immediate => {
                match &operand.value {
                    operand::OperandValue::Number(num) => *num,
                    operand::OperandValue::String(s) => {
                        // For immediate addressing with a string, try to parse it as a number
                        s.parse::<i64>().map_err(|_| {
                            VmError::InvalidOperand(format!(
                                "Cannot use string '{}' as immediate value",
                                s
                            ))
                        })?
                    }
                }
            }
        };
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
        &[OperandKind::Direct, OperandKind::Indirect, OperandKind::Immediate]
    }

    fn execute(
        &self,
        operand: Option<&Operand>,
        vm_state: &mut dyn VmState,
    ) -> Result<(), VmError> {
        let operand = operand
            .ok_or_else(|| VmError::InvalidOperand("DIV requires an operand".to_string()))?;
        let value = match operand.kind {
            OperandKind::Direct => {
                match &operand.value {
                    operand::OperandValue::Number(num) => vm_state.get_memory(*num)?,
                    operand::OperandValue::String(s) => {
                        // Try to resolve the string as a label
                        let address = vm_state.resolve_label(s)?;
                        vm_state.get_memory(address as i64)?
                    }
                }
            }
            OperandKind::Indirect => {
                match &operand.value {
                    operand::OperandValue::Number(num) => {
                        let address = vm_state.get_memory(*num)?;
                        vm_state.get_memory(address)?
                    }
                    operand::OperandValue::String(s) => {
                        // Try to resolve the string as a label
                        let address = vm_state.resolve_label(s)?;
                        let indirect_address = vm_state.get_memory(address as i64)?;
                        vm_state.get_memory(indirect_address)?
                    }
                }
            }
            OperandKind::Immediate => {
                match &operand.value {
                    operand::OperandValue::Number(num) => *num,
                    operand::OperandValue::String(s) => {
                        // For immediate addressing with a string, try to parse it as a number
                        s.parse::<i64>().map_err(|_| {
                            VmError::InvalidOperand(format!(
                                "Cannot use string '{}' as immediate value",
                                s
                            ))
                        })?
                    }
                }
            }
        };
        if value == 0 {
            return Err(VmError::DivisionByZero);
        }
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
        match operand.kind {
            OperandKind::Direct => {
                // For JUMP, we always treat the operand as a label name
                let label = match &operand.value {
                    operand::OperandValue::Number(num) => num.to_string(),
                    operand::OperandValue::String(s) => s.clone(),
                };
                debug!("JUMP: Attempting to jump to label '{}'", label);
                let pc = vm_state.resolve_label(&label)?;
                debug!("JUMP: Resolved label '{}' to PC={}", label, pc);
                vm_state.set_program_counter(pc);
            }
            _ => {
                return Err(VmError::InvalidOperand(
                    "JUMP can only use direct addressing".to_string(),
                ));
            }
        }
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
        if vm_state.accumulator() > 0 {
            match operand.kind {
                OperandKind::Direct => {
                    // For JGTZ, we always treat the operand as a label name
                    let label = match &operand.value {
                        operand::OperandValue::Number(num) => num.to_string(),
                        operand::OperandValue::String(s) => s.clone(),
                    };
                    debug!("JGTZ: Attempting to jump to label '{}'", label);
                    let pc = vm_state.resolve_label(&label)?;
                    debug!("JGTZ: Resolved label '{}' to PC={}", label, pc);
                    vm_state.set_program_counter(pc);
                }
                _ => {
                    return Err(VmError::InvalidOperand(
                        "JGTZ can only use direct addressing".to_string(),
                    ));
                }
            }
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
        if vm_state.accumulator() == 0 {
            match operand.kind {
                OperandKind::Direct => {
                    // For JZERO, we always treat the operand as a label name
                    let label = match &operand.value {
                        operand::OperandValue::Number(num) => num.to_string(),
                        operand::OperandValue::String(s) => s.clone(),
                    };
                    debug!("JZERO: Attempting to jump to label '{}'", label);
                    let pc = vm_state.resolve_label(&label)?;
                    debug!("JZERO: Resolved label '{}' to PC={}", label, pc);
                    vm_state.set_program_counter(pc);
                }
                _ => {
                    return Err(VmError::InvalidOperand(
                        "JZERO can only use direct addressing".to_string(),
                    ));
                }
            }
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
        &[OperandKind::Direct, OperandKind::Indirect]
    }

    fn execute(
        &self,
        operand: Option<&Operand>,
        vm_state: &mut dyn VmState,
    ) -> Result<(), VmError> {
        let operand = operand
            .ok_or_else(|| VmError::InvalidOperand("READ requires an operand".to_string()))?;
        let value = vm_state.read_input()?;
        match operand.kind {
            OperandKind::Direct => {
                match &operand.value {
                    operand::OperandValue::Number(num) => vm_state.set_memory(*num, value)?,
                    operand::OperandValue::String(s) => {
                        // Try to resolve the string as a label
                        let address = vm_state.resolve_label(s)?;
                        vm_state.set_memory(address as i64, value)?
                    }
                }
            }
            OperandKind::Indirect => {
                match &operand.value {
                    operand::OperandValue::Number(num) => {
                        let address = vm_state.get_memory(*num)?;
                        vm_state.set_memory(address, value)?
                    }
                    operand::OperandValue::String(s) => {
                        // Try to resolve the string as a label
                        let address = vm_state.resolve_label(s)?;
                        let indirect_address = vm_state.get_memory(address as i64)?;
                        vm_state.set_memory(indirect_address, value)?
                    }
                }
            }
            OperandKind::Immediate => {
                return Err(VmError::InvalidOperand(
                    "READ cannot use immediate addressing".to_string(),
                ));
            }
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
        &[OperandKind::Direct, OperandKind::Indirect, OperandKind::Immediate]
    }

    fn execute(
        &self,
        operand: Option<&Operand>,
        vm_state: &mut dyn VmState,
    ) -> Result<(), VmError> {
        let operand = operand
            .ok_or_else(|| VmError::InvalidOperand("WRITE requires an operand".to_string()))?;
        let value = match operand.kind {
            OperandKind::Direct => {
                match &operand.value {
                    operand::OperandValue::Number(num) => vm_state.get_memory(*num)?,
                    operand::OperandValue::String(s) => {
                        // Try to resolve the string as a label
                        let address = vm_state.resolve_label(s)?;
                        vm_state.get_memory(address as i64)?
                    }
                }
            }
            OperandKind::Indirect => {
                match &operand.value {
                    operand::OperandValue::Number(num) => {
                        let address = vm_state.get_memory(*num)?;
                        vm_state.get_memory(address)?
                    }
                    operand::OperandValue::String(s) => {
                        // Try to resolve the string as a label
                        let address = vm_state.resolve_label(s)?;
                        let indirect_address = vm_state.get_memory(address as i64)?;
                        vm_state.get_memory(indirect_address)?
                    }
                }
            }
            OperandKind::Immediate => {
                match &operand.value {
                    operand::OperandValue::Number(num) => *num,
                    operand::OperandValue::String(s) => {
                        // For immediate addressing with a string, try to parse it as a number
                        s.parse::<i64>().map_err(|_| {
                            VmError::InvalidOperand(format!(
                                "Cannot use string '{}' as immediate value",
                                s
                            ))
                        })?
                    }
                }
            }
        };
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
pub fn standard_instructions() -> crate::registry::InstructionRegistry {
    use std::sync::Arc;

    use crate::instruction::InstructionKind;
    use crate::registry::InstructionRegistry;

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
