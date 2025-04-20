//! Tests for the instruction set API

use std::sync::Arc;

use crate::instruction::InstructionKind;
use crate::instruction_set::{InstructionSet, STANDARD_INSTRUCTION_SET};
use crate::plugin::InstructionBuilder;

#[test]
fn test_standard_instruction_set() {
    // Get the standard instruction set
    let standard_set = InstructionSet::standard();

    // Check that it has the expected name and description
    assert_eq!(standard_set.name, "Standard");
    assert_eq!(
        standard_set.description,
        "The standard instruction set for the RAM virtual machine"
    );

    // Check that it has metadata
    assert_eq!(standard_set.get_metadata("version"), Some(&"1.0.0".to_string()));
    assert_eq!(standard_set.get_metadata("author"), Some(&"RAM VM Team".to_string()));
    assert_eq!(standard_set.get_metadata("license"), Some(&"MIT".to_string()));

    // Check that it contains all standard instructions
    for kind in InstructionKind::standard_kinds() {
        assert!(standard_set.contains(&kind), "Standard set should contain {:?}", kind);
    }

    // Check that we can get information about instructions
    let load_info = standard_set.get_info(&InstructionKind::Load).expect("Failed to get LOAD info");
    assert_eq!(load_info.name, "LOAD");
    assert!(load_info.requires_operand);

    // Check that we can get information by name
    let store_info = standard_set.get_info_by_name("STORE").expect("Failed to get STORE info");
    assert_eq!(store_info.name, "STORE");

    // Check that we can get information by name (case-insensitive)
    let add_info =
        standard_set.get_info_by_name_case_insensitive("add").expect("Failed to get ADD info");
    assert_eq!(add_info.name, "ADD");
}

#[test]
fn test_custom_instruction_set() {
    // Create a custom instruction set
    let mut custom_set = InstructionSet::new("Custom", "A custom instruction set for testing");

    // Add metadata
    custom_set
        .add_metadata("version", "0.1.0")
        .add_metadata("author", "Test Author")
        .add_metadata("license", "MIT");

    // Add a custom instruction
    let custom_instruction = InstructionBuilder::new("CUSTOM")
        .requires_operand(true)
        .allow_operand_kind(crate::operand::OperandKind::Direct)
        .execute(|_, _| Ok(()))
        .build();

    let custom_kind = InstructionKind::Custom(Arc::from("CUSTOM"));
    custom_set.add_instruction(custom_kind.clone(), custom_instruction);

    // Check that the set contains the custom instruction
    assert!(custom_set.contains(&custom_kind));
    assert!(custom_set.contains_name("CUSTOM"));
    assert!(custom_set.contains_name_case_insensitive("custom"));

    // Check that we can get information about the custom instruction
    let custom_info = custom_set.get_info(&custom_kind).expect("Failed to get CUSTOM info");
    assert_eq!(custom_info.name, "CUSTOM");
    assert!(custom_info.requires_operand);

    // Check that we can get the custom instruction by name
    let custom_def = custom_set.get_by_name("CUSTOM").expect("Failed to get CUSTOM definition");
    assert_eq!(custom_def.name(), "CUSTOM");

    // Check that we can get the custom instruction by name (case-insensitive)
    let custom_def =
        custom_set.get_by_name_case_insensitive("custom").expect("Failed to get CUSTOM definition");
    assert_eq!(custom_def.name(), "CUSTOM");
}

#[test]
fn test_merging_instruction_sets() {
    // Create a custom instruction set
    let mut custom_set = InstructionSet::new("Custom", "A custom instruction set for testing");

    // Add a custom instruction
    let custom_instruction = InstructionBuilder::new("CUSTOM")
        .requires_operand(true)
        .allow_operand_kind(crate::operand::OperandKind::Direct)
        .execute(|_, _| Ok(()))
        .build();

    let custom_kind = InstructionKind::Custom(Arc::from("CUSTOM"));
    custom_set.add_instruction(custom_kind.clone(), custom_instruction);

    // Create a merged set
    let mut merged_set = InstructionSet::standard();
    merged_set.merge(&custom_set);

    // Check that the merged set contains both standard and custom instructions
    for kind in InstructionKind::standard_kinds() {
        assert!(merged_set.contains(&kind), "Merged set should contain {:?}", kind);
    }
    assert!(merged_set.contains(&custom_kind));

    // Check that we can get information about both standard and custom instructions
    let load_info = merged_set.get_info(&InstructionKind::Load).expect("Failed to get LOAD info");
    assert_eq!(load_info.name, "LOAD");

    let custom_info = merged_set.get_info(&custom_kind).expect("Failed to get CUSTOM info");
    assert_eq!(custom_info.name, "CUSTOM");
}

#[test]
fn test_static_standard_instruction_set() {
    // Check that the static standard instruction set is initialized
    assert_eq!(STANDARD_INSTRUCTION_SET.name, "Standard");
    assert_eq!(
        STANDARD_INSTRUCTION_SET.description,
        "The standard instruction set for the RAM virtual machine"
    );

    // Check that it contains all standard instructions
    for kind in InstructionKind::standard_kinds() {
        assert!(STANDARD_INSTRUCTION_SET.contains(&kind), "Standard set should contain {:?}", kind);
    }
}
