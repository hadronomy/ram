//! Example of how to use the instruction set API

use std::sync::Arc;

use crate::instruction::InstructionKind;
use crate::instruction_set::InstructionSet;
use crate::operand::OperandKind;
use crate::operand_resolver::{DefaultOperandResolver, OperandResolver};
use crate::plugin::InstructionBuilder;

/// Example of how to use the instruction set API
pub fn instruction_set_example() {
    // Get the standard instruction set
    let standard_set = InstructionSet::standard();
    println!("Standard instruction set: {}", standard_set.name);
    println!("Description: {}", standard_set.description);
    println!("Version: {}", standard_set.get_metadata("version").unwrap_or(&"unknown".to_string()));

    // Print information about all instructions in the standard set
    println!("\nStandard instructions:");
    for info in standard_set.get_all_info() {
        println!("  {}: {}", info.name, info.description);
        println!("    Requires operand: {}", info.requires_operand);
        println!("    Allowed operand kinds: {:?}", info.allowed_operand_kinds);
    }

    // Create a custom instruction set
    let mut custom_set =
        InstructionSet::new("Custom", "A custom instruction set with additional instructions");

    // Add metadata
    custom_set
        .add_metadata("version", "0.1.0")
        .add_metadata("author", "Example Author")
        .add_metadata("license", "MIT");

    // Add a custom instruction
    let custom_instruction = InstructionBuilder::new("POW")
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

    let custom_kind = InstructionKind::Custom(Arc::from("POW"));
    custom_set.add_instruction(custom_kind.clone(), custom_instruction);

    // Print information about the custom instruction
    println!("\nCustom instruction:");
    let custom_info = custom_set.get_info(&custom_kind).unwrap();
    println!("  {}: {}", custom_info.name, custom_info.description);
    println!("    Requires operand: {}", custom_info.requires_operand);
    println!("    Allowed operand kinds: {:?}", custom_info.allowed_operand_kinds);

    // Create a merged set with both standard and custom instructions
    let mut merged_set = InstructionSet::standard();
    merged_set.merge(&custom_set);

    // Print information about all instructions in the merged set
    println!("\nMerged instruction set:");
    println!("  Total instructions: {}", merged_set.get_all_info().len());
    println!("  Instruction names: {:?}", merged_set.names().collect::<Vec<_>>());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instruction_set_example() {
        // Just make sure the example runs without panicking
        instruction_set_example();
    }
}
