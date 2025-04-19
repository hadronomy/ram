//! AST to HIR lowering
//!
//! This module handles the conversion from AST to HIR.
//! It extracts semantic information from the AST and builds
//! the HIR representation.

use std::collections::HashMap;

use base_db::input::FileId;
use hir_def::item_tree::ItemTree;
use ram_core::instruction::InstructionKind;
use ram_syntax::{AstNode, SyntaxKind, ast};
use tracing::warn;

use crate::body::{
    AddressingMode, Body, Expr, ExprKind, Instruction, InstructionCall, Label, LabelRef, Literal,
    MemoryRef,
};
use crate::expr::ExprId;
use crate::ids::{DefId, LocalDefId};

/// A collector for lowering AST to HIR
pub struct HirCollector {
    /// The body being built
    body: Body,

    /// The file ID being processed
    _file_id: FileId,

    /// Map of label names to their definition IDs
    label_defs: HashMap<String, DefId>,

    /// Next available expression ID
    next_expr_id: u32,

    /// Next available local definition ID
    next_local_id: u32,
}

impl HirCollector {
    /// Create a new HIR collector
    pub fn new(owner: DefId, file_id: FileId) -> Self {
        Self {
            body: Body { owner, exprs: Vec::new(), instructions: Vec::new(), labels: Vec::new() },
            _file_id: file_id,
            label_defs: HashMap::new(),
            next_expr_id: 0,
            next_local_id: 0,
        }
    }

    /// Generate a new unique expression ID
    fn next_expr_id(&mut self) -> ExprId {
        let id = ExprId(self.next_expr_id);
        self.next_expr_id += 1;
        id
    }

    /// Generate a new unique local definition ID
    fn next_local_id(&mut self) -> LocalDefId {
        let id = LocalDefId(self.next_local_id);
        self.next_local_id += 1;
        id
    }

    /// Add a label definition
    fn add_label_def(&mut self, name: String, id: DefId) {
        self.label_defs.insert(name, id);
    }

    /// Lower an ItemTree to a HIR Body
    pub fn lower_item_tree(&mut self, item_tree: &ItemTree, file_id: FileId) {
        // First, collect all label definitions
        for label in &item_tree.labels {
            let def_id = DefId { file_id, local_id: LocalDefId(label.id.0) };

            self.add_label_def(label.name.clone(), def_id);

            // Add the label to the body
            self.body.labels.push(Label {
                id: LocalDefId(label.id.0),
                name: label.name.clone(),
                instruction_id: None, // Will be set later when processing instructions
            });
        }

        // TODO: Process instructions from AST
        // This would require access to the AST, which we don't have directly from the ItemTree
        // In a real implementation, we would need to get the AST from the database
    }

    /// Lower an AST Program to a HIR Body
    pub fn lower_ast(&mut self, program: &ast::Program) {
        for stmt in program.statements() {
            if let Some(instruction) = stmt.instruction() {
                self.lower_instruction(&instruction);
            }

            if let Some(label_def) = stmt.label_def() {
                // Labels are already processed from the ItemTree
                // We need to associate them with the next instruction
                if let Some(name) = label_def.name() {
                    // Find the label in our body
                    for label in &mut self.body.labels {
                        if label.name == name {
                            // Store the ID of the next instruction that will be created
                            label.instruction_id = Some(LocalDefId(self.next_local_id));
                            break;
                        }
                    }
                }
            }
        }
    }

    /// Lower an AST Instruction to a HIR Instruction
    fn lower_instruction(&mut self, instruction: &ast::Instruction) {
        // Extract the opcode from the instruction
        let opcode = instruction
            .syntax()
            .children_with_tokens()
            .filter_map(|node_or_token| node_or_token.into_token())
            .find(|token| token.kind() == SyntaxKind::IDENTIFIER)
            .map(|token| token.text().to_string().to_uppercase())
            .unwrap_or_else(|| "UNKNOWN".to_string());

        let _kind = InstructionKind::from_name(&opcode);

        // Create the instruction call expression
        let mut operands = Vec::new();

        if let Some(operand) = instruction.operand() {
            let operand_expr_id = self.lower_operand(&operand);
            operands.push(operand_expr_id);
        }

        // Store the first operand for the instruction
        let first_operand = if operands.is_empty() { None } else { Some(operands[0]) };

        let expr_id = self.next_expr_id();
        let expr = Expr {
            id: expr_id,
            kind: ExprKind::InstructionCall(InstructionCall {
                opcode: opcode.clone(),
                operands: operands.clone(),
            }),
        };
        self.body.exprs.push(expr);

        // Create the instruction
        let instr = Instruction { id: self.next_local_id(), opcode, operand: first_operand };
        self.body.instructions.push(instr);
    }

    /// Lower an AST Operand to a HIR Expression
    fn lower_operand(&mut self, operand: &ast::Operand) -> ExprId {
        let expr_id = self.next_expr_id();

        let kind = if let Some(direct) = operand.as_direct() {
            self.lower_direct_operand(direct, expr_id)
        } else if let Some(indirect) = operand.as_indirect() {
            self.lower_indirect_operand(indirect, expr_id)
        } else if let Some(immediate) = operand.as_immediate() {
            self.lower_immediate_operand(immediate, expr_id)
        } else {
            // Default to a literal 0 if we can't determine the operand type
            warn!("Unknown operand type {}", operand.syntax());
            ExprKind::Literal(Literal::Int(0))
        };

        let expr = Expr { id: expr_id, kind };
        self.body.exprs.push(expr);

        expr_id
    }

    /// Lower a direct operand
    fn lower_direct_operand(&mut self, operand: ast::DirectOperand, _expr_id: ExprId) -> ExprKind {
        if let Some(value) = operand.value() {
            if let Some(num) = value.as_number() {
                // For numeric values, treat them as memory references
                return ExprKind::MemoryRef(MemoryRef {
                    mode: AddressingMode::Direct,
                    address: self.create_literal_expr(Literal::Int(num)),
                });
            } else if let Some(ident) = value.as_identifier() {
                // Check if the identifier is a known label
                if let Some(def_id) = self.label_defs.get(&ident) {
                    return ExprKind::LabelRef(LabelRef { label_id: *def_id });
                } else {
                    // If the identifier is not a known label, treat it as a label literal
                    return ExprKind::Literal(Literal::Label(ident));
                }
            }
        }

        // If we can't determine the value, return a literal 0 instead of warning
        warn!("Unknown operand type {}", operand.syntax());
        ExprKind::Literal(Literal::Int(0))
    }

    /// Lower an indirect operand
    fn lower_indirect_operand(
        &mut self,
        operand: ast::IndirectOperand,
        _expr_id: ExprId,
    ) -> ExprKind {
        if let Some(value) = operand.value() {
            if let Some(num) = value.as_number() {
                return ExprKind::MemoryRef(MemoryRef {
                    mode: AddressingMode::Indirect,
                    address: self.create_literal_expr(Literal::Int(num)),
                });
            } else if let Some(ident) = value.as_identifier() {
                if let Some(&def_id) = self.label_defs.get(&ident) {
                    // Create a label reference expression for indirect label references
                    let label_ref_id = self.next_expr_id();
                    let label_ref = Expr {
                        id: label_ref_id,
                        kind: ExprKind::LabelRef(LabelRef { label_id: def_id }),
                    };
                    self.body.exprs.push(label_ref);

                    return ExprKind::MemoryRef(MemoryRef {
                        mode: AddressingMode::Indirect,
                        address: label_ref_id,
                    });
                } else {
                    // If the identifier is not a known label, treat it as a label literal
                    return ExprKind::Literal(Literal::Label(ident));
                }
            }
        }

        // If we can't determine the value, return a literal 0 instead of warning
        warn!("Unknown operand type {}", operand.syntax());
        ExprKind::Literal(Literal::Int(0))
    }

    /// Lower an immediate operand
    fn lower_immediate_operand(
        &mut self,
        operand: ast::ImmediateOperand,
        _expr_id: ExprId,
    ) -> ExprKind {
        if let Some(value) = operand.value() {
            if let Some(num) = value.as_number() {
                return ExprKind::MemoryRef(MemoryRef {
                    mode: AddressingMode::Immediate,
                    address: self.create_literal_expr(Literal::Int(num)),
                });
            } else if let Some(ident) = value.as_identifier() {
                // For immediate addressing with identifiers, we need to handle it differently
                // than direct/indirect addressing
                if let Some(&def_id) = self.label_defs.get(&ident) {
                    // Create a label reference - this will be resolved to a value at runtime
                    return ExprKind::LabelRef(LabelRef { label_id: def_id });
                } else {
                    // If not a known label, treat it as a string literal
                    return ExprKind::Literal(Literal::String(ident));
                }
            }
        }

        // If we can't determine the value, return a literal 0 instead of warning
        warn!("Unknown operand type {}", operand.syntax());
        ExprKind::Literal(Literal::Int(0))
    }

    /// Create a literal expression
    fn create_literal_expr(&mut self, literal: Literal) -> ExprId {
        let expr_id = self.next_expr_id();
        let expr = Expr { id: expr_id, kind: ExprKind::Literal(literal) };
        self.body.exprs.push(expr);
        expr_id
    }

    /// Finish building the body and return it
    pub fn finish(self) -> Body {
        self.body
    }
}

/// Lower an AST Program to a HIR Body
pub fn lower_program(
    program: &ast::Program,
    owner: DefId,
    file_id: FileId,
    item_tree: &ItemTree,
) -> Body {
    let mut collector = HirCollector::new(owner, file_id);
    collector.lower_item_tree(item_tree, file_id);
    collector.lower_ast(program);
    collector.finish()
}
