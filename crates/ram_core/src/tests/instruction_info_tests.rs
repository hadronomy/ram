//! Tests for the instruction info API

use crate::instruction::InstructionKind;
use crate::operand::OperandKind;
use crate::registry::InstructionRegistry;

#[test]
fn test_instruction_info() {
    // Get info about a standard instruction
    let load_info = InstructionKind::Load.info();

    assert_eq!(load_info.name, "LOAD");
    assert!(load_info.requires_operand);
    assert!(load_info.allowed_operand_kinds.contains(&OperandKind::Direct));
    assert!(load_info.allowed_operand_kinds.contains(&OperandKind::Indirect));
    assert!(load_info.allowed_operand_kinds.contains(&OperandKind::Immediate));
    assert_eq!(load_info.description, "Load a value into the accumulator");

    // Get info about a custom instruction
    let custom_kind = InstructionKind::Custom(std::sync::Arc::from("CUSTOM"));
    let custom_info = custom_kind.info();

    assert_eq!(custom_info.name, "CUSTOM");
    assert!(custom_info.requires_operand);
    assert!(custom_info.allowed_operand_kinds.contains(&OperandKind::Direct));
    assert_eq!(custom_info.description, "Custom instruction");

    // Get info about all standard instructions
    let standard_infos = InstructionKind::standard_instructions_info();
    assert_eq!(standard_infos.len(), 12); // 12 standard instructions

    // Check that HALT doesn't require an operand
    let halt_info =
        standard_infos.iter().find(|info| info.name == "HALT").expect("HALT instruction not found");

    assert!(!halt_info.requires_operand);
    assert!(halt_info.allowed_operand_kinds.is_empty());
}

#[test]
fn test_registry_info_methods() {
    // Create a registry
    let registry = InstructionRegistry::new();

    // We don't need to register instructions to get info about them
    // The info is available directly from the InstructionKind

    // Get info about a standard instruction by kind
    let load_kind = InstructionKind::Load;
    let load_info = registry.get_info(&load_kind).expect("Failed to get LOAD info");

    assert_eq!(load_info.name, "LOAD");
    assert!(load_info.requires_operand);

    // Get info about all standard instructions directly
    let all_standard_infos = InstructionKind::standard_instructions_info();
    assert!(!all_standard_infos.is_empty());

    // Check that we can get info about a custom instruction
    let custom_kind = InstructionKind::Custom(std::sync::Arc::from("CUSTOM"));
    let custom_info = registry.get_info(&custom_kind).expect("Failed to get CUSTOM info");
    assert_eq!(custom_info.name, "CUSTOM");
}
