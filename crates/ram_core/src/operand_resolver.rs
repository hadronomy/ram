//! Utilities for resolving operands in the RAM virtual machine

use tracing::debug;

use crate::db::VmState;
use crate::error::VmError;
use crate::operand::{Operand, OperandKind, OperandValue};

/// Trait for resolving operands in the RAM virtual machine
pub trait OperandResolver {
    /// Resolves an operand to a value based on its kind and the VM state
    fn resolve_operand_value(
        &self,
        operand: &Operand,
        vm_state: &mut dyn VmState,
    ) -> Result<i64, VmError>;

    /// Resolves a memory address for storing values
    fn resolve_store_address(
        &self,
        operand: &Operand,
        vm_state: &mut dyn VmState,
    ) -> Result<i64, VmError>;

    /// Resolves a jump target from an operand
    fn resolve_jump_target(
        &self,
        operand: &Operand,
        vm_state: &mut dyn VmState,
    ) -> Result<usize, VmError>;
}

/// Default implementation of the OperandResolver trait
#[derive(Debug, Clone, Default)]
pub struct DefaultOperandResolver;

impl OperandResolver for DefaultOperandResolver {
    fn resolve_operand_value(
        &self,
        operand: &Operand,
        vm_state: &mut dyn VmState,
    ) -> Result<i64, VmError> {
        debug!("Resolving operand value: {:?}", operand);
        let result = match operand.kind {
            OperandKind::Direct => self.resolve_direct_operand(operand, vm_state),
            OperandKind::Indirect => self.resolve_indirect_operand(operand, vm_state),
            OperandKind::Immediate => self.resolve_immediate_operand(operand),
        };
        debug!("Resolved operand value: {:?} -> {:?}", operand, result);
        result
    }

    fn resolve_store_address(
        &self,
        operand: &Operand,
        vm_state: &mut dyn VmState,
    ) -> Result<i64, VmError> {
        debug!("Resolving store address: {:?}", operand);
        let result = match operand.kind {
            OperandKind::Direct => {
                match &operand.value {
                    OperandValue::Number(num) => {
                        // Special case: operand 0 refers to the accumulator
                        // Return a special value that will be recognized by the VM
                        // as a signal to store to the accumulator
                        if *num == 0 {
                            Ok(0) // Special value for accumulator
                        } else {
                            Ok(*num)
                        }
                    }
                    OperandValue::String(s) => {
                        // Try to resolve the string as a label
                        let address = vm_state.resolve_label(s)?;
                        Ok(address as i64)
                    }
                }
            }
            OperandKind::Indirect => {
                match &operand.value {
                    OperandValue::Number(num) => {
                        // Special case: operand 0 refers to the accumulator
                        if *num == 0 {
                            // For indirect store, we need to get the address from the accumulator
                            Ok(vm_state.accumulator())
                        } else {
                            vm_state.get_memory(*num)
                        }
                    }
                    OperandValue::String(s) => {
                        // Try to resolve the string as a label
                        let address = vm_state.resolve_label(s)?;
                        vm_state.get_memory(address as i64)
                    }
                }
            }
            OperandKind::Immediate => {
                Err(VmError::InvalidOperand("Cannot store to an immediate value".to_string()))
            }
        };
        debug!("Resolved store address: {:?} -> {:?}", operand, result);
        result
    }

    fn resolve_jump_target(
        &self,
        operand: &Operand,
        vm_state: &mut dyn VmState,
    ) -> Result<usize, VmError> {
        match operand.kind {
            OperandKind::Direct => match &operand.value {
                OperandValue::Number(num) => Ok(*num as usize),
                OperandValue::String(s) => {
                    let pc = vm_state.resolve_label(s)?;
                    Ok(pc)
                }
            },
            _ => Err(VmError::InvalidOperand(
                "Jump instructions can only use direct addressing".to_string(),
            )),
        }
    }
}

impl DefaultOperandResolver {
    /// Resolves a direct operand (e.g., 5 or label)
    fn resolve_direct_operand(
        &self,
        operand: &Operand,
        vm_state: &mut dyn VmState,
    ) -> Result<i64, VmError> {
        debug!("Resolving direct operand {:?}", operand);
        match &operand.value {
            OperandValue::Number(num) => {
                // Special case: operand 0 refers to the accumulator
                if *num == 0 { Ok(vm_state.accumulator()) } else { vm_state.get_memory(*num) }
            }
            OperandValue::String(s) => {
                // Try to resolve the string as a label
                let address = vm_state.resolve_label(s)?;
                vm_state.get_memory(address as i64)
            }
        }
    }

    /// Resolves an indirect operand (e.g., *5 or *label)
    fn resolve_indirect_operand(
        &self,
        operand: &Operand,
        vm_state: &mut dyn VmState,
    ) -> Result<i64, VmError> {
        match &operand.value {
            OperandValue::Number(num) => {
                // Special case: operand 0 refers to the accumulator
                if *num == 0 {
                    // Use the accumulator value as the address
                    let address = vm_state.accumulator();
                    vm_state.get_memory(address)
                } else {
                    let address = vm_state.get_memory(*num)?;
                    vm_state.get_memory(address)
                }
            }
            OperandValue::String(s) => {
                // Try to resolve the string as a label
                let address = vm_state.resolve_label(s)?;
                let indirect_address = vm_state.get_memory(address as i64)?;
                vm_state.get_memory(indirect_address)
            }
        }
    }

    /// Resolves an immediate operand (e.g., =5 or ="string")
    fn resolve_immediate_operand(&self, operand: &Operand) -> Result<i64, VmError> {
        match &operand.value {
            OperandValue::Number(num) => Ok(*num),
            OperandValue::String(s) => {
                // For immediate addressing with a string, try to parse it as a number
                s.parse::<i64>().map_err(|_| {
                    VmError::InvalidOperand(format!("Cannot use string '{}' as immediate value", s))
                })
            }
        }
    }
}

/// Resolves an operand to a value based on its kind and the VM state
pub fn resolve_operand_value(
    operand: &Operand,
    vm_state: &mut dyn VmState,
) -> Result<i64, VmError> {
    DefaultOperandResolver.resolve_operand_value(operand, vm_state)
}

/// Resolves a memory address for storing values
pub fn resolve_store_address(
    operand: &Operand,
    vm_state: &mut dyn VmState,
) -> Result<i64, VmError> {
    DefaultOperandResolver.resolve_store_address(operand, vm_state)
}

/// Resolves a jump target from an operand
pub fn resolve_jump_target(
    operand: &Operand,
    vm_state: &mut dyn VmState,
) -> Result<usize, VmError> {
    DefaultOperandResolver.resolve_jump_target(operand, vm_state)
}
