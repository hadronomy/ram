//! Program representation for the RAM virtual machine

use std::collections::HashMap;

use hir::body::{ExprKind, LabelRef, Literal};
use ram_core::error::VmError;
use ram_core::instruction::Instruction;

/// A program for the RAM virtual machine
#[derive(Debug, Clone)]
pub struct Program {
    /// The instructions in the program
    pub instructions: Vec<Instruction>,
    /// Map of label names to instruction indices
    pub labels: HashMap<String, usize>,
}

impl Program {
    /// Create a new empty program
    pub fn new() -> Self {
        Self { instructions: Vec::new(), labels: HashMap::new() }
    }
}

impl Default for Program {
    fn default() -> Self {
        Self::new()
    }
}

impl Program {
    /// Create a program from a HIR representation
    pub fn from_hir(
        body: &hir::body::Body,
        _db: &dyn crate::db::VmDatabase,
    ) -> Result<Self, VmError> {
        unimplemented!("Program::from_hir");
    }

    /// Get the instruction at the given index
    pub fn get_instruction(&self, index: usize) -> Option<&Instruction> {
        self.instructions.get(index)
    }

    /// Get the number of instructions in the program
    pub fn len(&self) -> usize {
        self.instructions.len()
    }

    /// Check if the program is empty
    pub fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }

    /// Resolve a label to an instruction index
    pub fn resolve_label(&self, label: &str) -> Result<usize, VmError> {
        self.labels
            .get(label)
            .copied()
            .ok_or_else(|| VmError::InvalidInstruction(format!("Unknown label: {}", label)))
    }
}
