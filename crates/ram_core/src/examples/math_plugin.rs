//! Example plugin that adds advanced math instructions to the RAM virtual machine

use std::sync::Arc;

use crate::instruction::InstructionKind;
use crate::operand::OperandKind;
use crate::operand_resolver::{DefaultOperandResolver, OperandResolver};
use crate::plugin::{InstructionBuilder, RamPlugin};
use crate::registry::InstructionRegistry;

/// A plugin that adds advanced math instructions to the RAM virtual machine
pub struct MathPlugin;

impl RamPlugin for MathPlugin {
    fn name(&self) -> &str {
        "MathPlugin"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn description(&self) -> &str {
        "Adds advanced math instructions to the RAM virtual machine"
    }

    fn register(&self, registry: &mut InstructionRegistry) {
        // Register POW instruction (raise accumulator to the power of operand)
        let pow_instruction = InstructionBuilder::new("POW")
            .requires_operand(true)
            .allow_operand_kind(OperandKind::Direct)
            .allow_operand_kind(OperandKind::Indirect)
            .allow_operand_kind(OperandKind::Immediate)
            .execute(|operand, vm_state| {
                let operand = operand.ok_or_else(|| {
                    crate::error::VmError::InvalidOperand("POW requires an operand".to_string())
                })?;

                // Use the operand resolver to get the value
                let resolver = DefaultOperandResolver;
                let value = resolver.resolve_operand_value(operand, vm_state)?;

                // Raise the accumulator to the power of the value
                let acc = vm_state.accumulator();
                let result = acc.pow(value as u32);
                vm_state.set_accumulator(result);

                Ok(())
            })
            .build();

        // Register SQRT instruction (square root of accumulator)
        let sqrt_instruction = InstructionBuilder::new("SQRT")
            .requires_operand(false)
            .execute(|_, vm_state| {
                // Calculate the square root of the accumulator
                let acc = vm_state.accumulator();
                if acc < 0 {
                    return Err(crate::error::VmError::InvalidOperand(
                        "Cannot take square root of negative number".to_string(),
                    ));
                }

                let result = (acc as f64).sqrt() as i64;
                vm_state.set_accumulator(result);

                Ok(())
            })
            .build();

        // Register ABS instruction (absolute value of accumulator)
        let abs_instruction = InstructionBuilder::new("ABS")
            .requires_operand(false)
            .execute(|_, vm_state| {
                // Calculate the absolute value of the accumulator
                let acc = vm_state.accumulator();
                vm_state.set_accumulator(acc.abs());

                Ok(())
            })
            .build();

        // Register the instructions with the registry
        registry.register(InstructionKind::Custom(Arc::from("POW")), pow_instruction);
        registry.register(InstructionKind::Custom(Arc::from("SQRT")), sqrt_instruction);
        registry.register(InstructionKind::Custom(Arc::from("ABS")), abs_instruction);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::VmState;
    use crate::operand::{Operand, OperandKind, OperandValue};

    // A simple VM state implementation for testing
    struct TestVmState {
        accumulator: i64,
        memory: Vec<i64>,
    }

    impl TestVmState {
        fn new() -> Self {
            Self { accumulator: 0, memory: vec![0; 100] }
        }
    }

    impl VmState for TestVmState {
        fn accumulator(&self) -> i64 {
            self.accumulator
        }

        fn set_accumulator(&mut self, value: i64) {
            self.accumulator = value;
        }

        fn get_memory(&self, address: i64) -> Result<i64, crate::error::VmError> {
            if address < 0 || address >= self.memory.len() as i64 {
                return Err(crate::error::VmError::InvalidMemoryAccess(format!(
                    "Address {} out of bounds",
                    address
                )));
            }
            Ok(self.memory[address as usize])
        }

        fn set_memory(&mut self, address: i64, value: i64) -> Result<(), crate::error::VmError> {
            if address < 0 || address >= self.memory.len() as i64 {
                return Err(crate::error::VmError::InvalidMemoryAccess(format!(
                    "Address {} out of bounds",
                    address
                )));
            }
            self.memory[address as usize] = value;
            Ok(())
        }

        fn program_counter(&self) -> usize {
            0
        }

        fn set_program_counter(&mut self, _pc: usize) {
            // Not used in this test
        }

        fn read_input(&mut self) -> Result<i64, crate::error::VmError> {
            Err(crate::error::VmError::IoError("Input not supported in test".to_string()))
        }

        fn write_output(&mut self, _value: i64) -> Result<(), crate::error::VmError> {
            Ok(())
        }

        fn resolve_label(&self, _label: &str) -> Result<usize, crate::error::VmError> {
            Err(crate::error::VmError::InvalidOperand("Labels not supported in test".to_string()))
        }
    }

    #[test]
    fn test_math_plugin() {
        // Create a registry and register the plugin
        let mut registry = InstructionRegistry::new();
        let plugin = MathPlugin;
        plugin.register(&mut registry);

        // Test POW instruction
        let mut vm_state = TestVmState::new();
        vm_state.set_accumulator(2);

        let pow_kind = InstructionKind::Custom(Arc::from("POW"));
        let pow_def = registry.get(&pow_kind).unwrap();

        let operand = Operand { kind: OperandKind::Immediate, value: OperandValue::Number(3) };

        pow_def.execute(Some(&operand), &mut vm_state).unwrap();
        assert_eq!(vm_state.accumulator(), 8); // 2^3 = 8

        // Test SQRT instruction
        vm_state.set_accumulator(16);

        let sqrt_kind = InstructionKind::Custom(Arc::from("SQRT"));
        let sqrt_def = registry.get(&sqrt_kind).unwrap();

        sqrt_def.execute(None, &mut vm_state).unwrap();
        assert_eq!(vm_state.accumulator(), 4); // sqrt(16) = 4

        // Test ABS instruction
        vm_state.set_accumulator(-42);

        let abs_kind = InstructionKind::Custom(Arc::from("ABS"));
        let abs_def = registry.get(&abs_kind).unwrap();

        abs_def.execute(None, &mut vm_state).unwrap();
        assert_eq!(vm_state.accumulator(), 42); // abs(-42) = 42
    }
}
