//! Program representation for the RAM virtual machine

use std::collections::HashMap;

use hir::body;
use hir::ids::DefId;
use ram_core::error::VmError;
use ram_core::instruction::Instruction;
use ram_core::operand::{Operand, OperandValue};
use tracing::debug;

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
    /// Find a label by its DefId in the HIR body
    fn find_label_by_id(body: &body::Body, label_id: DefId) -> Result<&body::Label, VmError> {
        body.labels
            .iter()
            .find(|l| {
                let def_id = DefId { file_id: body.owner.file_id, local_id: l.id };
                def_id == label_id
            })
            .ok_or_else(|| {
                VmError::InvalidInstruction(format!("Could not find label with ID: {:?}", label_id))
            })
    }

    /// Create a program from a HIR representation
    pub fn from_hir(body: &body::Body, _db: &dyn crate::db::VmDatabase) -> Result<Self, VmError> {
        let mut program = Program::new();

        // First pass: collect instruction ID mapping
        let mut instruction_indices: HashMap<u32, usize> = HashMap::new();
        for (idx, instr) in body.instructions.iter().enumerate() {
            debug!("HIR Instruction: {:?}", instr);
            instruction_indices.insert(instr.id.0, idx);
        }

        // Second pass: process all labels to build an accurate label map
        debug!("HIR Labels: {:?}", body.labels);
        for label in &body.labels {
            if let Some(instr_id) = label.instruction_id {
                if let Some(&idx) = instruction_indices.get(&instr_id.0) {
                    program.labels.insert(label.name.clone(), idx);
                }
            }
        }

        // Third pass: process all instructions
        for instr in &body.instructions {
            // Get the instruction kind from the opcode
            let kind = ram_core::instruction::InstructionKind::from_name(&instr.opcode);

            // Get the operand if any
            let operand = if let Some(expr_id) = instr.operand {
                // Find the expression
                let expr = body.exprs.iter().find(|e| e.id == expr_id).ok_or_else(|| {
                    VmError::InvalidInstruction(format!(
                        "Could not find expression with ID: {:?}",
                        expr_id
                    ))
                })?;

                // Convert the expression to an operand
                match &expr.kind {
                    body::ExprKind::Literal(body::Literal::Int(value)) => {
                        Some(Operand::direct(*value))
                    }
                    body::ExprKind::Literal(body::Literal::String(value)) => {
                        // For string literals, always use the string value
                        Some(Operand::direct_str(value.clone()))
                    }
                    body::ExprKind::Literal(body::Literal::Label(label_name)) => {
                        // For label literals, always use the label name as a string
                        // The VM will resolve it at runtime
                        Some(Operand::direct_str(label_name.clone()))
                    }
                    body::ExprKind::LabelRef(label_ref) => {
                        // Find the label by its ID
                        let label = Self::find_label_by_id(body, label_ref.label_id)?;

                        // Always use the label name as a string
                        // The VM will resolve it at runtime
                        Some(Operand::direct_str(label.name.clone()))
                    }
                    body::ExprKind::MemoryRef(mem_ref) => {
                        // Get the address expression
                        let addr_expr =
                            body.exprs.iter().find(|e| e.id == mem_ref.address).ok_or_else(
                                || {
                                    VmError::InvalidInstruction(format!(
                                        "Could not find address expression with ID: {:?}",
                                        mem_ref.address
                                    ))
                                },
                            )?;

                        // Get the address value
                        let operand_value = match &addr_expr.kind {
                            body::ExprKind::Literal(body::Literal::Int(value)) => {
                                OperandValue::Number(*value)
                            }
                            body::ExprKind::Literal(body::Literal::String(value)) => {
                                OperandValue::String(value.clone())
                            }
                            body::ExprKind::Literal(body::Literal::Label(label_name)) => {
                                OperandValue::String(label_name.clone())
                            }
                            body::ExprKind::LabelRef(label_ref) => {
                                // Find the label by its ID
                                let label = Self::find_label_by_id(body, label_ref.label_id)?;

                                // Use the label name as a string
                                OperandValue::String(label.name.clone())
                            }
                            _ => {
                                return Err(VmError::InvalidInstruction(format!(
                                    "Unsupported address expression: {:?}",
                                    addr_expr.kind
                                )));
                            }
                        };

                        // Create the operand with the appropriate addressing mode
                        match mem_ref.mode {
                            body::AddressingMode::Direct => match operand_value {
                                OperandValue::Number(n) => Some(Operand::direct(n)),
                                OperandValue::String(s) => Some(Operand::direct_str(s)),
                            },
                            body::AddressingMode::Indirect => match operand_value {
                                OperandValue::Number(n) => Some(Operand::indirect(n)),
                                OperandValue::String(s) => Some(Operand::indirect_str(s)),
                            },
                            body::AddressingMode::Immediate => match operand_value {
                                OperandValue::Number(n) => Some(Operand::immediate(n)),
                                OperandValue::String(s) => Some(Operand::immediate_str(s)),
                            },
                        }
                    }
                    _ => {
                        return Err(VmError::InvalidInstruction(format!(
                            "Unsupported operand expression: {:?}",
                            expr.kind
                        )));
                    }
                }
            } else {
                None
            };

            // Create the instruction
            let instruction = ram_core::instruction::Instruction::new(kind, operand);

            // Validate the instruction
            instruction.validate()?;

            // Add the instruction to the program
            program.instructions.push(instruction);
        }

        Ok(program)
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
