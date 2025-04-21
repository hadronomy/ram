//! Examples for the RAM virtual machine
//!
//! This module contains examples of how to use the RAM virtual machine,
//! including how to create and use plugins and instruction sets.

pub mod instruction_set_example;
pub mod math_plugin;

/// Example of how to use the plugin system
#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::math_plugin::MathPlugin;
    use crate::instruction::{Instruction, InstructionKind};
    use crate::operand::{Operand, OperandKind, OperandValue};
    use crate::plugin::PluginManager;
    use crate::registry::InstructionRegistry;

    #[test]
    fn test_plugin_system() {
        // Create a plugin manager and register the math plugin
        let mut plugin_manager = PluginManager::new();
        plugin_manager.register_plugin(Arc::new(MathPlugin));

        // Create an instruction registry
        let mut registry = InstructionRegistry::new();

        // Register all plugins with the registry
        plugin_manager.register_all(&mut registry);

        // Now we can use the custom instructions
        let pow_kind = InstructionKind::Custom(Arc::from("POW"));
        let sqrt_kind = InstructionKind::Custom(Arc::from("SQRT"));
        let abs_kind = InstructionKind::Custom(Arc::from("ABS"));

        // Check that the instructions are registered
        assert!(registry.contains(&pow_kind));
        assert!(registry.contains(&sqrt_kind));
        assert!(registry.contains(&abs_kind));

        // We can also look up instructions by name
        assert!(registry.get_by_name("POW").is_some());
        assert!(registry.get_by_name("SQRT").is_some());
        assert!(registry.get_by_name("ABS").is_some());

        // And we can create instructions using the registry
        // These would be used in a real program, but we're just demonstrating
        // that they can be created
        let _pow_instruction = Instruction::with_operand(
            pow_kind,
            Operand { kind: OperandKind::Immediate, value: OperandValue::Number(2) },
        );

        let _sqrt_instruction = Instruction::without_operand(sqrt_kind);
        let _abs_instruction = Instruction::without_operand(abs_kind);

        // These instructions can now be executed by a VM
        // (see the math_plugin.rs file for an example of how to execute them)
    }
}
