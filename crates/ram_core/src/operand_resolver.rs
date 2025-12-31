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

    /// Resolves a target address (Register or Memory) for storing values
    fn resolve_store_address(
        &self,
        operand: &Operand,
        vm_state: &mut dyn VmState,
    ) -> Result<(StoreTarget, i64), VmError>;

    /// Resolves a jump target from an operand
    fn resolve_jump_target(
        &self,
        operand: &Operand,
        vm_state: &mut dyn VmState,
    ) -> Result<usize, VmError>;
}

/// Defines where a value should be stored
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreTarget {
    Register,
    Memory,
    Accumulator, // Special case for address 0 in some contexts, or explicit logic
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
            OperandKind::Indexed => self.resolve_indexed_operand(operand, vm_state),
        };
        debug!("Resolved operand value: {:?} -> {:?}", operand, result);
        result
    }

    fn resolve_store_address(
        &self,
        operand: &Operand,
        vm_state: &mut dyn VmState,
    ) -> Result<(StoreTarget, i64), VmError> {
        debug!("Resolving store address: {:?}", operand);
        match operand.kind {
            OperandKind::Direct => {
                match &operand.value {
                    OperandValue::Number(num) => {
                        // Direct addressing (e.g. STORE 1) targets a Register
                        if *num == 0 {
                            Ok((StoreTarget::Accumulator, 0))
                        } else {
                            Ok((StoreTarget::Register, *num))
                        }
                    }
                    OperandValue::String(s) => {
                        let address = vm_state.resolve_label(s)?;
                        Ok((StoreTarget::Register, address as i64))
                    }
                    OperandValue::Indexed(_, _) => Err(VmError::InvalidOperand(
                        "Invalid direct operand value (indexed) for store".to_string(),
                    )),
                }
            }
            OperandKind::Indirect => {
                // Indirect addressing (e.g. STORE *1) targets Memory at address found in Register 1
                match &operand.value {
                    OperandValue::Number(num) => {
                        let ptr_val = if *num == 0 {
                            vm_state.accumulator()
                        } else {
                            vm_state.get_register(*num)?
                        };
                        Ok((StoreTarget::Memory, ptr_val))
                    }
                    OperandValue::String(s) => {
                        let reg_idx = vm_state.resolve_label(s)?;
                        let ptr_val = vm_state.get_register(reg_idx as i64)?;
                        Ok((StoreTarget::Memory, ptr_val))
                    }
                    OperandValue::Indexed(_, _) => Err(VmError::InvalidOperand(
                        "Invalid indirect operand value (indexed) for store".to_string(),
                    )),
                }
            }
            OperandKind::Indexed => {
                // Indexed addressing (e.g. STORE 3[1]) targets Memory at (3 + Reg[1])
                match &operand.value {
                    OperandValue::Indexed(base, index_reg) => {
                        let index_val = vm_state.get_register(*index_reg)?;
                        Ok((StoreTarget::Memory, base + index_val))
                    }
                    _ => Err(VmError::InvalidOperand(
                        "Invalid indexed operand for store".to_string(),
                    )),
                }
            }
            OperandKind::Immediate => {
                Err(VmError::InvalidOperand("Cannot store to an immediate value".to_string()))
            }
        }
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
                OperandValue::Indexed(_, _) => Err(VmError::InvalidOperand(
                    "Invalid direct operand value (indexed) for jump".to_string(),
                )),
            },
            _ => Err(VmError::InvalidOperand(
                "Jump instructions can only use direct addressing".to_string(),
            )),
        }
    }
}

impl DefaultOperandResolver {
    /// Resolves a direct operand (ACCESS REGISTER)
    fn resolve_direct_operand(
        &self,
        operand: &Operand,
        vm_state: &mut dyn VmState,
    ) -> Result<i64, VmError> {
        match &operand.value {
            OperandValue::Number(num) => {
                if *num == 0 {
                    Ok(vm_state.accumulator())
                } else {
                    vm_state.get_register(*num)
                }
            }
            OperandValue::String(s) => {
                let address = vm_state.resolve_label(s)?;
                vm_state.get_register(address as i64)
            }
            OperandValue::Indexed(_, _) => Err(VmError::InvalidOperand(
                "Unexpected indexed value in direct operand".to_string(),
            )),
        }
    }

    /// Resolves an indirect operand (ACCESS MEMORY via REGISTER POINTER)
    fn resolve_indirect_operand(
        &self,
        operand: &Operand,
        vm_state: &mut dyn VmState,
    ) -> Result<i64, VmError> {
        match &operand.value {
            OperandValue::Number(num) => {
                let address =
                    if *num == 0 { vm_state.accumulator() } else { vm_state.get_register(*num)? };
                vm_state.get_memory(address)
            }
            OperandValue::String(s) => {
                let reg_idx = vm_state.resolve_label(s)?;
                let address = vm_state.get_register(reg_idx as i64)?;
                vm_state.get_memory(address)
            }
            OperandValue::Indexed(_, _) => Err(VmError::InvalidOperand(
                "Unexpected indexed value in indirect operand".to_string(),
            )),
        }
    }

    /// Resolves an immediate operand (LITERAL VALUE)
    fn resolve_immediate_operand(&self, operand: &Operand) -> Result<i64, VmError> {
        match &operand.value {
            OperandValue::Number(num) => Ok(*num),
            OperandValue::String(s) => s.parse::<i64>().map_err(|_| {
                VmError::InvalidOperand(format!("Cannot use string '{}' as immediate value", s))
            }),
            OperandValue::Indexed(_, _) => Err(VmError::InvalidOperand(
                "Unexpected indexed value in immediate operand".to_string(),
            )),
        }
    }

    /// Resolves an indexed operand (ACCESS MEMORY via BASE + REGISTER INDEX)
    fn resolve_indexed_operand(
        &self,
        operand: &Operand,
        vm_state: &mut dyn VmState,
    ) -> Result<i64, VmError> {
        match &operand.value {
            OperandValue::Indexed(base, index_reg) => {
                // index_reg is a register index
                let index_val = vm_state.get_register(*index_reg)?;
                let effective_addr = base + index_val;
                vm_state.get_memory(effective_addr)
            }
            _ => Err(VmError::InvalidOperand("Invalid indexed operand".to_string())),
        }
    }
}

/// Resolves an operand to a value
pub fn resolve_operand_value(
    operand: &Operand,
    vm_state: &mut dyn VmState,
) -> Result<i64, VmError> {
    DefaultOperandResolver.resolve_operand_value(operand, vm_state)
}

/// Resolves a store target
pub fn resolve_store_address(
    operand: &Operand,
    vm_state: &mut dyn VmState,
) -> Result<(StoreTarget, i64), VmError> {
    DefaultOperandResolver.resolve_store_address(operand, vm_state)
}

/// Resolves a jump target
pub fn resolve_jump_target(
    operand: &Operand,
    vm_state: &mut dyn VmState,
) -> Result<usize, VmError> {
    DefaultOperandResolver.resolve_jump_target(operand, vm_state)
}
