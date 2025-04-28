//! AST to HIR lowering
//!
//! This module handles the conversion from AST to HIR.
//! It extracts semantic information from the AST and builds
//! the HIR representation.

use std::collections::HashMap;

use base_db::input::FileId;
use cstree::text::TextRange;
use hir_def::item_tree::ItemTree;
use ram_core::instruction::InstructionKind;
use ram_syntax::{AstNode, SyntaxKind, ast};
use tracing::{debug, error, warn};

use crate::body::{
    AddressingMode, Body, Expr, ExprKind, Instruction, InstructionCall, Label, LabelRef, Literal,
    MemoryRef,
};
// Assume HirDatabase trait exists or will be added if needed for context lookups
// use crate::db::HirDatabase;
use crate::expr::ExprId;
use crate::ids::{DefId, LocalDefId};

/// Errors that can occur during HIR lowering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirError {
    MissingOpcode(TextRange),
    UnknownOperand(TextRange),
    MissingDirectOperandValue(TextRange),
    InvalidDirectOperandValue(TextRange),
    MissingIndirectOperandValue(TextRange),
    InvalidIndirectOperandValue(TextRange),
    MissingImmediateOperandValue(TextRange),
    InvalidImmediateOperandValue(TextRange),
    LabelNotFoundInItemTree(String, TextRange),
    LabelNotFoundInBody(String, LocalDefId),
    // Consider adding: UnknownIdentifier(String, TextRange),
}

impl std::fmt::Display for HirError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HirError::MissingOpcode(range) => write!(f, "Missing opcode at {:?}", range),
            HirError::UnknownOperand(range) => write!(f, "Unknown operand type at {:?}", range),
            HirError::MissingDirectOperandValue(range) => {
                write!(f, "Missing value for direct operand at {:?}", range)
            }
            HirError::InvalidDirectOperandValue(range) => {
                write!(f, "Invalid value for direct operand at {:?}", range)
            }
            HirError::MissingIndirectOperandValue(range) => {
                write!(f, "Missing value for indirect operand at {:?}", range)
            }
            HirError::InvalidIndirectOperandValue(range) => {
                write!(f, "Invalid value for indirect operand at {:?}", range)
            }
            HirError::MissingImmediateOperandValue(range) => {
                write!(f, "Missing value for immediate operand at {:?}", range)
            }
            HirError::InvalidImmediateOperandValue(range) => {
                write!(f, "Invalid value for immediate operand at {:?}", range)
            }
            HirError::LabelNotFoundInItemTree(name, range) => {
                write!(f, "Label '{}' defined at {:?} not found in ItemTree", name, range)
            }
            HirError::LabelNotFoundInBody(name, id) => {
                write!(f, "Label '{}' (ID {:?}) not found in HIR body labels", name, id)
            }
        }
    }
}

impl std::error::Error for HirError {}

/// A collector for lowering AST to HIR.
/// It builds the HIR Body by processing an ItemTree and an AST Program.
pub struct HirCollector {
    /// The body being built.
    body: Body,

    /// Map of label names to their definition IDs (global). Populated from ItemTree.
    label_defs: HashMap<String, DefId>,

    /// Map of label names to their local IDs within the current body. Populated from ItemTree.
    label_name_to_local_id: HashMap<String, LocalDefId>,

    /// Next available expression ID.
    next_expr_id: u32,

    /// Next available local definition ID (used for instructions).
    next_local_id: u32,
}

impl HirCollector {
    /// Create a new HIR collector, initializing labels from the ItemTree.
    pub fn new(owner: DefId, file_id: FileId, item_tree: &ItemTree) -> Self {
        let mut label_defs = HashMap::new();
        let mut label_name_to_local_id = HashMap::new();
        let mut labels = Vec::new();

        // Pre-populate labels from ItemTree
        for label_def in &item_tree.labels {
            // Assume ItemTreeId corresponds to the LocalDefId for labels within this file.
            let local_id = LocalDefId(label_def.id.0);
            let def_id = DefId { file_id, local_id };

            label_defs.insert(label_def.name.clone(), def_id);
            label_name_to_local_id.insert(label_def.name.clone(), local_id);

            labels.push(Label {
                id: local_id,
                name: label_def.name.clone(),
                instruction_id: None, // To be filled during AST lowering
            });
        }

        Self {
            body: Body { owner, exprs: Vec::new(), instructions: Vec::new(), labels },
            label_defs,
            label_name_to_local_id,
            next_expr_id: 0,
            // Start local IDs for instructions after the highest ID used by ItemTree items?
            // Or just start from 0? Let's start from 0 for simplicity, assuming no overlap needed.
            next_local_id: 0,
        }
    }

    /// Generate a new unique expression ID.
    fn next_expr_id(&mut self) -> ExprId {
        let id = ExprId(self.next_expr_id);
        self.next_expr_id += 1;
        id
    }

    /// Generate a new unique local definition ID for instructions.
    fn next_instruction_local_id(&mut self) -> LocalDefId {
        let id = LocalDefId(self.next_local_id);
        self.next_local_id += 1;
        id
    }

    /// Lower the body of an AST Program, processing statements and linking labels.
    pub fn lower_program_body(&mut self, program: &ast::Program) -> Result<(), HirError> {
        let mut current_label_name: Option<String> = None;
        let mut last_instruction_id: Option<LocalDefId> = None;

        for stmt in program.statements() {
            // Check if this statement has an instruction
            let has_instruction = stmt.instruction().is_some();

            // Process label if present
            if let Some(label_def) = stmt.label_def() {
                let Some(name) = label_def.name() else {
                    // Skip labels without names (should ideally be caught by parser/ItemTree)
                    continue;
                };
                // Check if this label was defined in ItemTree (and thus pre-loaded).
                if !self.label_name_to_local_id.contains_key(&name) {
                    // This indicates an inconsistency between AST and ItemTree.
                    error!("Label '{}' found in AST but not in ItemTree", name);
                    return Err(HirError::LabelNotFoundInItemTree(
                        name,
                        label_def.syntax().text_range(),
                    ));
                }
                // Store the name of the label
                current_label_name = Some(name);

                // If this statement doesn't also have an instruction, continue to the next statement
                if !has_instruction {
                    continue;
                }
                // Otherwise, fall through to process the instruction in the same statement
            }

            // Process instruction if present
            if let Some(instruction) = stmt.instruction() {
                let instr_local_id = self.next_instruction_local_id(); // Get ID for this instruction.

                // If there's a label for this instruction, pass it to the lower_instruction method
                let label_for_instruction = current_label_name.clone();
                let mut hir_instruction = self.lower_instruction(&instruction, instr_local_id)?;

                // Set the label name on the instruction if there is one
                if let Some(label_name) = &label_for_instruction {
                    hir_instruction.label_name = Some(label_name.clone());
                }

                // If there was a label just before this instruction, link it.
                if let Some(label_name) = current_label_name.take() {
                    self.link_label_to_instruction(&label_name, instr_local_id)?;
                }

                self.body.instructions.push(hir_instruction);
                last_instruction_id = Some(instr_local_id);
            }
        }

        // Handle any remaining label that might be at the end of the file
        // Link it to the last instruction if available
        if let Some(label_name) = current_label_name {
            if let Some(last_id) = last_instruction_id {
                self.link_label_to_instruction(&label_name, last_id)?;
            } else {
                // This is a rare case where there's a label but no instructions in the file
                warn!("Label '{}' found at the end of the file with no instructions to link to", label_name);
            }
        }

        Ok(())
    }

    /// Links a label (identified by name) to the given instruction ID.
    fn link_label_to_instruction(
        &mut self,
        label_name: &str,
        instr_local_id: LocalDefId,
    ) -> Result<(), HirError> {
        let Some(label_local_id) = self.label_name_to_local_id.get(label_name).copied() else {
            // This case should be prevented by the check in lower_program_body,
            // but handle defensively.
            error!(
                "Internal error: Label name '{}' present but not found in local ID map.",
                label_name
            );
            // Or return a more specific internal error?
            return Err(HirError::LabelNotFoundInBody(
                label_name.to_string(),
                LocalDefId(u32::MAX),
            )); // Placeholder ID
        };

        let Some(label_in_body) = self.body.labels.iter_mut().find(|l| l.id == label_local_id)
        else {
            // This indicates an internal inconsistency.
            error!(
                "Label '{}' (ID {:?}) not found in body.labels during linking",
                label_name, label_local_id
            );
            return Err(HirError::LabelNotFoundInBody(label_name.to_string(), label_local_id));
        };

        // Link the label to this instruction's ID.
        label_in_body.instruction_id = Some(instr_local_id);
        Ok(())
    }

    /// Lower an AST Instruction to a HIR Instruction.
    fn lower_instruction(
        &mut self,
        instruction: &ast::Instruction,
        instr_local_id: LocalDefId,
    ) -> Result<Instruction, HirError> {
        // Extract the opcode (identifier token).
        let opcode_token = instruction
            .syntax()
            .children_with_tokens()
            .filter_map(|node_or_token| node_or_token.into_token())
            .find(|token| token.kind() == SyntaxKind::IDENTIFIER)
            .ok_or_else(|| HirError::MissingOpcode(instruction.syntax().text_range()))?;

        let opcode = opcode_token.text().to_string().to_uppercase();

        // Validate opcode if needed (optional).
        let _kind = InstructionKind::from_name(&opcode);

        // Lower the operand, if present.
        let mut operand_exprs = Vec::new();
        if let Some(operand) = instruction.operand() {
            let operand_expr_id = self.lower_operand(&operand)?;
            operand_exprs.push(operand_expr_id);
        }

        // The HIR Instruction currently only stores the first operand's ExprId.
        let first_operand_expr_id = operand_exprs.first().copied();

        // Create the associated InstructionCall expression.
        let call_expr_id = self.next_expr_id();
        let call_expr = Expr {
            id: call_expr_id,
            kind: ExprKind::InstructionCall(InstructionCall {
                opcode: opcode.clone(),
                operands: operand_exprs, // Store all lowered operands here.
            }),
        };
        self.body.exprs.push(call_expr);

        // Create the HIR Instruction.
        let hir_instruction = Instruction {
            id: instr_local_id,
            opcode,
            operand: first_operand_expr_id, // Link to the first operand expression.
            label_name: None, // Will be set by the caller if needed
        };

        Ok(hir_instruction)
    }

    /// Lower an AST Operand to a HIR Expression, returning its ExprId.
    fn lower_operand(&mut self, operand: &ast::Operand) -> Result<ExprId, HirError> {
        let expr_id = self.next_expr_id();

        let kind = if let Some(direct) = operand.as_direct() {
            self.lower_direct_operand(direct)?
        } else if let Some(indirect) = operand.as_indirect() {
            self.lower_indirect_operand(indirect)?
        } else if let Some(immediate) = operand.as_immediate() {
            self.lower_immediate_operand(immediate)?
        } else {
            // Should be unreachable if the grammar is correct, but handle defensively.
            error!("Unknown operand type encountered: {}", operand.syntax());
            return Err(HirError::UnknownOperand(operand.syntax().text_range()));
        };

        let expr = Expr { id: expr_id, kind };
        self.body.exprs.push(expr);

        Ok(expr_id)
    }

    /// Lower a direct operand (e.g., `LOAD 100`, `LOAD label`).
    fn lower_direct_operand(&mut self, operand: ast::DirectOperand) -> Result<ExprKind, HirError> {
        let value_node = operand
            .value()
            .ok_or_else(|| HirError::MissingDirectOperandValue(operand.syntax().text_range()))?;

        if let Some(num) = value_node.as_number() {
            // Direct numeric address: `LOAD 100` -> MemoryRef(Direct, Literal(100))
            let literal_expr_id = self.create_literal_expr(Literal::Int(num))?;
            return Ok(ExprKind::MemoryRef(MemoryRef {
                mode: AddressingMode::Direct,
                address: literal_expr_id,
            }));
        }

        if let Some(ident) = value_node.as_identifier() {
            // Direct label address: `LOAD my_label`
            return self.lower_identifier_operand(&ident, AddressingMode::Direct);
        }

        Err(HirError::InvalidDirectOperandValue(value_node.syntax().text_range()))
    }

    /// Lower an indirect operand (e.g., `LOAD *100`, `LOAD *label`).
    fn lower_indirect_operand(
        &mut self,
        operand: ast::IndirectOperand,
    ) -> Result<ExprKind, HirError> {
        let value_node = operand
            .value()
            .ok_or_else(|| HirError::MissingIndirectOperandValue(operand.syntax().text_range()))?;

        if let Some(num) = value_node.as_number() {
            // Indirect numeric address: `LOAD *100` -> MemoryRef(Indirect, Literal(100))
            let literal_expr_id = self.create_literal_expr(Literal::Int(num))?;
            return Ok(ExprKind::MemoryRef(MemoryRef {
                mode: AddressingMode::Indirect,
                address: literal_expr_id,
            }));
        }

        if let Some(ident) = value_node.as_identifier() {
            // Indirect label address: `LOAD *my_label`
            return self.lower_identifier_operand(&ident, AddressingMode::Indirect);
        }

        Err(HirError::InvalidIndirectOperandValue(value_node.syntax().text_range()))
    }

    /// Lower an immediate operand (e.g., `LOAD #100`, `LOAD #label`).
    fn lower_immediate_operand(
        &mut self,
        operand: ast::ImmediateOperand,
    ) -> Result<ExprKind, HirError> {
        let value_node = operand
            .value()
            .ok_or_else(|| HirError::MissingImmediateOperandValue(operand.syntax().text_range()))?;

        if let Some(num) = value_node.as_number() {
            // Immediate numeric value: `LOAD #100` -> Literal(100)
            // The value *is* the operand, not an address.
            return Ok(ExprKind::Literal(Literal::Int(num)));
        }

        if let Some(ident) = value_node.as_identifier() {
            // Immediate label value: `LOAD #my_label`
            return self.lower_identifier_operand(&ident, AddressingMode::Immediate);
        }

        Err(HirError::InvalidImmediateOperandValue(value_node.syntax().text_range()))
    }

    /// Helper to lower an identifier used as an operand value, handling different addressing modes.
    fn lower_identifier_operand(
        &mut self,
        ident: &str,
        mode: AddressingMode,
    ) -> Result<ExprKind, HirError> {
        match self.label_defs.get(ident).copied() {
            Some(def_id) => {
                // Known label
                match mode {
                    AddressingMode::Direct | AddressingMode::Immediate => {
                        // `LOAD label` or `LOAD #label` -> LabelRef(def_id)
                        Ok(ExprKind::LabelRef(LabelRef { label_id: def_id }))
                    }
                    AddressingMode::Indirect => {
                        // `LOAD *label` -> MemoryRef(Indirect, LabelRef(def_id))
                        let label_ref_id = self.create_label_ref_expr(def_id)?;
                        Ok(ExprKind::MemoryRef(MemoryRef {
                            mode: AddressingMode::Indirect,
                            address: label_ref_id,
                        }))
                    }
                }
            }
            None => {
                // Unknown identifier
                warn!("{:?} operand identifier '{}' not found in known labels.", mode, ident);
                match mode {
                    AddressingMode::Direct | AddressingMode::Indirect => {
                        // Treat as a label literal for now (might be resolved later or error).
                        // `LOAD label` -> Literal(Label(ident))
                        // `LOAD *label` -> Literal(Label(ident)) - Address is the unknown label name
                        Ok(ExprKind::Literal(Literal::Label(ident.to_string())))
                    }
                    AddressingMode::Immediate => {
                        // Treat as a string literal.
                        // `LOAD #label` -> Literal(String(ident))
                        Ok(ExprKind::Literal(Literal::String(ident.to_string())))
                    }
                }
            }
        }
    }

    /// Helper to create a literal expression and add it to the body.
    fn create_literal_expr(&mut self, literal: Literal) -> Result<ExprId, HirError> {
        let expr_id = self.next_expr_id();
        let expr = Expr { id: expr_id, kind: ExprKind::Literal(literal) };
        self.body.exprs.push(expr);
        Ok(expr_id)
    }

    /// Helper to create a label reference expression and add it to the body.
    fn create_label_ref_expr(&mut self, label_id: DefId) -> Result<ExprId, HirError> {
        let expr_id = self.next_expr_id();
        let expr = Expr { id: expr_id, kind: ExprKind::LabelRef(LabelRef { label_id }) };
        self.body.exprs.push(expr);
        Ok(expr_id)
    }

    /// Finish building the body and return it.
    pub fn finish(self) -> Body {
        // Potentially perform final checks or optimizations on self.body here.
        self.body
    }
}

/// Lower an AST Program to a HIR Body using information from the ItemTree.
///
/// This is the main entry point for HIR lowering.
pub fn lower_program(
    // db: &dyn HirDatabase, // Pass database if context lookups are needed
    program: &ast::Program,
    owner: DefId, // ID of the item this body belongs to (e.g., function, module)
    file_id: FileId,
    item_tree: &ItemTree, // Pre-parsed item information
) -> Result<Body, HirError> {
    // 1. Initialize collector with ItemTree info (labels, etc.)
    let mut collector = HirCollector::new(owner, file_id, item_tree);

    // 2. Lower the program body from the AST, linking labels to instructions
    collector.lower_program_body(program)?;

    // 3. Finalize and return the HIR Body
    Ok(collector.finish())
}
