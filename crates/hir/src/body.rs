//! Function and block bodies
//!
//! This module defines the Body, which represents the semantics of
//! executable code within functions, blocks, or instruction sequences.

use std::default::Default;
use std::sync::Arc;

use crate::expr::ExprId;
use crate::ids::{DefId, LocalDefId};

/// A body of code, such as a function body or a block
#[derive(Debug, PartialEq, Eq, Default)]
pub struct Body {
    /// The owner of this body
    pub owner: DefId,

    /// The expressions in this body
    pub exprs: Vec<Expr>,

    /// The instructions in this body
    pub instructions: Vec<Instruction>,

    /// Labels defined in this body
    pub labels: Vec<Label>,
}

/// An expression in the body
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expr {
    /// Unique ID of this expression
    pub id: ExprId,

    /// The kind of expression
    pub kind: ExprKind,
}

/// The kind of an expression
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExprKind {
    /// A literal value
    Literal(Literal),

    /// A reference to a label
    LabelRef(LabelRef),

    /// A memory address reference
    MemoryRef(MemoryRef),

    /// A call to an instruction
    InstructionCall(InstructionCall),
}

/// A literal value
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Literal {
    /// An integer literal
    Int(i64),

    /// A string literal
    String(String),
}

/// A reference to a label
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabelRef {
    /// The definition ID of the referenced label
    pub label_id: DefId,
}

/// A memory address reference
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryRef {
    /// The addressing mode
    pub mode: AddressingMode,

    /// The address expression
    pub address: ExprId,
}

/// Memory addressing modes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddressingMode {
    /// Direct addressing (e.g., 5)
    Direct,

    /// Indirect addressing (e.g., *5)
    Indirect,

    /// Immediate addressing (e.g., =5)
    Immediate,
}

/// A call to an instruction
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionCall {
    /// The opcode (name) of the instruction
    pub opcode: String,

    /// The operands to the instruction
    pub operands: Vec<ExprId>,
}

/// An instruction in the body
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instruction {
    /// Unique ID of this instruction
    pub id: LocalDefId,

    /// The opcode (name) of the instruction
    pub opcode: String,

    /// The operand to the instruction (if any)
    pub operand: Option<ExprId>,
}

/// A label in the body
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label {
    /// Unique ID of this label
    pub id: LocalDefId,

    /// The name of the label
    pub name: String,
}

/// Query implementation for retrieving a body from the database
pub(crate) fn body_query(_db: &dyn crate::db::HirDatabase, def_id: DefId) -> Arc<Body> {
    // Here we would normally lower the AST for the given definition into a Body
    // For now, we'll just return an empty Body
    Arc::new(Body { owner: def_id, ..Body::default() })
}
