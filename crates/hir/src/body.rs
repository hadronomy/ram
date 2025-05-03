//! Function and block bodies
//!
//! This module defines the Body, which represents the semantics of
//! executable code within functions, blocks, or instruction sequences.

use std::default::Default;
use std::fmt;
use std::sync::Arc;

use ram_syntax::AstNode;

use crate::expr::ExprId;
use crate::ids::{DefId, LocalDefId};

/// A body of code, such as a function body or a block
#[derive(Clone, PartialEq, Eq, Default)]
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
#[derive(Clone, PartialEq, Eq)]
pub struct Expr {
    /// Unique ID of this expression
    pub id: ExprId,

    /// The kind of expression
    pub kind: ExprKind,

    /// Source span for this expression
    pub span: std::ops::Range<usize>,
}

/// The kind of an expression
#[derive(Clone, PartialEq, Eq)]
pub enum ExprKind {
    /// A literal value
    Literal(Literal),

    /// A reference to a label
    LabelRef(LabelRef),

    /// A memory address reference
    MemoryRef(MemoryRef),

    /// A call to an instruction
    InstructionCall(InstructionCall),

    /// An array access expression (e.g., 2[3])
    ArrayAccess(ArrayAccess),
}

/// A literal value
#[derive(Clone, PartialEq, Eq)]
pub enum Literal {
    /// An integer literal
    Int(i64),

    /// A string literal
    String(String),

    /// A label literal (for identifiers that represent labels)
    Label(String),
}

/// A reference to a label
#[derive(Clone, PartialEq, Eq)]
pub struct LabelRef {
    /// The definition ID of the referenced label
    pub label_id: DefId,
}

/// A memory address reference
#[derive(Clone, PartialEq, Eq)]
pub struct MemoryRef {
    /// The addressing mode
    pub mode: AddressingMode,

    /// The address expression
    pub address: ExprId,
}

/// Memory addressing modes
#[derive(Clone, PartialEq, Eq)]
pub enum AddressingMode {
    /// Direct addressing (e.g., 5)
    Direct,

    /// Indirect addressing (e.g., *5)
    Indirect,

    /// Immediate addressing (e.g., =5)
    Immediate,
}

/// A call to an instruction
#[derive(Clone, PartialEq, Eq)]
pub struct InstructionCall {
    /// The opcode (name) of the instruction
    pub opcode: String,

    /// The operands to the instruction
    pub operands: Vec<ExprId>,
}

/// An array access expression (e.g., 2[3])
#[derive(Clone, PartialEq, Eq)]
pub struct ArrayAccess {
    /// The base expression (array)
    pub array: ExprId,

    /// The index expression
    pub index: ExprId,
}

/// An instruction in the body
#[derive(Clone, PartialEq, Eq)]
pub struct Instruction {
    /// Unique ID of this instruction
    pub id: LocalDefId,

    /// The opcode (name) of the instruction
    pub opcode: String,

    /// The operand to the instruction (if any)
    pub operand: Option<ExprId>,

    /// The label associated with this instruction (if any)
    pub label_name: Option<String>,

    /// Source span for this instruction
    pub span: std::ops::Range<usize>,
}

/// A label in the body
#[derive(Clone, PartialEq, Eq)]
pub struct Label {
    /// Unique ID of this label
    pub id: LocalDefId,

    /// The name of the label
    pub name: String,

    /// The instruction this label is mapped to (if any)
    pub instruction_id: Option<LocalDefId>,

    /// Source span for this label
    pub span: std::ops::Range<usize>,
}

/// Query implementation for retrieving a body from the database
#[allow(dead_code)]
pub(crate) fn body_query(db: &dyn crate::db::HirDatabase, def_id: DefId) -> Arc<Body> {
    // Get the file ID from the definition ID
    let file_id = def_id.file_id;

    // Get the ItemTree for this file
    let item_tree = db.item_tree(file_id);

    // Get the file text
    let file_text = db.file_text(file_id).text(db).to_string();

    // Parse the file text into an AST Program
    let (program, _errors) = ram_parser::parse(&file_text);
    let (tree, cache) = ram_parser::build_tree(program);
    let syntax_node = ram_syntax::SyntaxNode::new_root_with_resolver(tree, cache);
    let program =
        ram_syntax::Program::cast(syntax_node).expect("Failed to cast root node to Program");

    // Lower the AST to HIR
    let body = crate::lower::lower_program(&program, def_id, file_id, &item_tree)
        .expect("Failed to lower program to HIR");

    Arc::new(body)
}

/// Utility functions for debugging HIR nodes
pub mod debug {
    use super::*;

    /// Print a detailed representation of a HIR body
    ///
    /// This function prints a detailed representation of a HIR body,
    /// including all expressions, instructions, and labels.
    ///
    /// # Examples
    ///
    /// ```
    /// use hir::body::{debug, Body};
    ///
    /// let body = /* get a body from somewhere */;
    /// let debug_output = debug::detailed_body(&body);
    /// println!("{:?}", debug_output);
    /// ```
    pub fn detailed_body(body: &Body) -> String {
        format!("{:?}", body)
    }

    /// Print a detailed representation of an expression
    ///
    /// This function prints a detailed representation of an expression,
    /// including its ID, kind, and span.
    ///
    /// # Examples
    ///
    /// ```
    /// use hir::body::{debug, Expr};
    ///
    /// let expr = /* get an expression from somewhere */;
    /// let debug_output = debug::detailed_expr(&expr);
    /// println!("{:?}", debug_output);
    /// ```
    pub fn detailed_expr(expr: &Expr) -> String {
        format!("{:?}", expr)
    }

    /// Print a detailed representation of an instruction
    ///
    /// This function prints a detailed representation of an instruction,
    /// including its ID, opcode, operand, and span.
    ///
    /// # Examples
    ///
    /// ```
    /// use hir::body::{debug, Instruction};
    ///
    /// let instruction = /* get an instruction from somewhere */;
    /// let debug_output = debug::detailed_instruction(&instruction);
    /// println!("{:?}", debug_output);
    /// ```
    pub fn detailed_instruction(instruction: &Instruction) -> String {
        format!("{:?}", instruction)
    }

    /// Print a detailed representation of a label
    ///
    /// This function prints a detailed representation of a label,
    /// including its ID, name, and span.
    ///
    /// # Examples
    ///
    /// ```
    /// use hir::body::{debug, Label};
    ///
    /// let label = /* get a label from somewhere */;
    /// let debug_output = debug::detailed_label(&label);
    /// println!("{:?}", debug_output);
    /// ```
    pub fn detailed_label(label: &Label) -> String {
        format!("{:?}", label)
    }

    /// Print a detailed representation of an expression by ID
    ///
    /// This function looks up an expression by its ID in a body and prints
    /// a detailed representation of it, including its kind and span.
    ///
    /// # Examples
    ///
    /// ```
    /// use hir::body::{debug, Body};
    /// use hir::expr::ExprId;
    ///
    /// let body = /* get a body from somewhere */;
    /// let expr_id = ExprId(0);
    /// let debug_output = debug::expr_by_id(&body, expr_id);
    /// println!("{:?}", debug_output);
    /// ```
    pub fn expr_by_id(body: &Body, expr_id: ExprId) -> String {
        if let Some(expr) = body.exprs.get(expr_id.0 as usize) {
            format!("{:?}", expr)
        } else {
            format!("Expression with ID {:?} not found", expr_id)
        }
    }

    /// Print a detailed representation of all expressions referenced by an instruction
    ///
    /// This function prints a detailed representation of all expressions
    /// referenced by an instruction, including operands and labels.
    ///
    /// # Examples
    ///
    /// ```
    /// use hir::body::{debug, Body, Instruction};
    ///
    /// let body = /* get a body from somewhere */;
    /// let instruction = /* get an instruction from somewhere */;
    /// let debug_output = debug::instruction_with_exprs(&body, &instruction);
    /// println!("{:?}", debug_output);
    /// ```
    pub fn instruction_with_exprs(body: &Body, instruction: &Instruction) -> String {
        let mut result = format!("{:?}", instruction);

        if let Some(operand_id) = instruction.operand {
            result.push_str("\n  Operand: ");
            result.push_str(&expr_by_id(body, operand_id));
        }

        result
    }

    /// Print a detailed representation of the entire HIR structure
    ///
    /// This function prints a detailed representation of the entire HIR structure,
    /// including all expressions, instructions, and labels, with cross-references
    /// resolved.
    ///
    /// # Examples
    ///
    /// ```
    /// use hir::body::{debug, Body};
    ///
    /// let body = /* get a body from somewhere */;
    /// let debug_output = debug::full_hir_structure(&body);
    /// println!("{:?}", debug_output);
    /// ```
    pub fn full_hir_structure(body: &Body) -> String {
        let mut result = format!("Body {{ owner: {:?} }}\n", body.owner);

        // Labels section
        if !body.labels.is_empty() {
            result.push_str("\nLabels:\n");
            for (i, label) in body.labels.iter().enumerate() {
                result.push_str(&format!("  [{:?}] {:?}\n", i, label));

                // Show the instruction this label is mapped to
                if let Some(instr_id) = label.instruction_id {
                    if let Some(pos) = body.instructions.iter().position(|i| i.id == instr_id) {
                        result.push_str(&format!("      → Instruction [{:?}]\n", pos));
                    }
                }
            }
        }

        // Instructions section
        if !body.instructions.is_empty() {
            result.push_str("\nInstructions:\n");
            for (i, instruction) in body.instructions.iter().enumerate() {
                result.push_str(&format!("  [{:?}] {:?}\n", i, instruction));

                // Show the operand expression
                if let Some(operand_id) = instruction.operand {
                    if let Some(expr) = body.exprs.get(operand_id.0 as usize) {
                        result.push_str(&format!("      Operand: {:?}\n", expr));
                    }
                }

                // Show the label associated with this instruction
                if let Some(label_name) = &instruction.label_name {
                    if let Some(pos) = body.labels.iter().position(|l| &l.name == label_name) {
                        result.push_str(&format!("      Label: [{:?}] {:?}\n", pos, label_name));
                    }
                }
            }
        }

        // Expressions section
        if !body.exprs.is_empty() {
            result.push_str("\nExpressions:\n");
            for (i, expr) in body.exprs.iter().enumerate() {
                result.push_str(&format!("  [{:?}] {:?}\n", i, expr));

                // Add details based on expression kind
                match &expr.kind {
                    ExprKind::LabelRef(label_ref) => {
                        result.push_str(&format!("      → Label: {:?}\n", label_ref.label_id));
                    }
                    ExprKind::MemoryRef(mem_ref) => {
                        result.push_str(&format!("      Mode: {:?}\n", mem_ref.mode));
                        if let Some(addr_expr) = body.exprs.get(mem_ref.address.0 as usize) {
                            result.push_str(&format!("      Address: {:?}\n", addr_expr));
                        }
                    }
                    ExprKind::InstructionCall(call) => {
                        result.push_str(&format!("      Opcode: {:?}\n", call.opcode));
                        for (j, operand_id) in call.operands.iter().enumerate() {
                            if let Some(operand_expr) = body.exprs.get(operand_id.0 as usize) {
                                result.push_str(&format!(
                                    "      Operand {:?}: {:?}\n",
                                    j, operand_expr
                                ));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        result
    }
}

impl fmt::Debug for Body {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Body {{ owner: {:?} }}", self.owner)?;

        if !self.labels.is_empty() {
            writeln!(f, "  Labels:")?;
            for label in &self.labels {
                writeln!(f, "    {:?}", label)?;
            }
        }

        if !self.instructions.is_empty() {
            writeln!(f, "  Instructions:")?;
            for instruction in &self.instructions {
                writeln!(f, "    {:?}", instruction)?;
            }
        }

        if !self.exprs.is_empty() {
            writeln!(f, "  Expressions:")?;
            for expr in &self.exprs {
                writeln!(f, "    {:?}", expr)?;
            }
        }

        Ok(())
    }
}

impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {:?} [{:?}..{:?}]", self.id, self.kind, self.span.start, self.span.end)
    }
}

impl fmt::Debug for ExprKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExprKind::Literal(lit) => write!(f, "{:?}", lit),
            ExprKind::LabelRef(label_ref) => write!(f, "{:?}", label_ref),
            ExprKind::MemoryRef(mem_ref) => write!(f, "{:?}", mem_ref),
            ExprKind::InstructionCall(call) => write!(f, "{:?}", call),
            ExprKind::ArrayAccess(array_access) => write!(f, "{:?}", array_access),
        }
    }
}

impl fmt::Debug for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Int(value) => write!(f, "Int({:?})", value),
            Literal::String(value) => write!(f, "String(\"{:?}\")", value),
            Literal::Label(name) => write!(f, "Label({:?})", name),
        }
    }
}

impl fmt::Debug for LabelRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LabelRef({:?})", self.label_id)
    }
}

impl fmt::Debug for MemoryRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MemoryRef({:?}, expr{:?})", self.mode, self.address.0)
    }
}

impl fmt::Debug for AddressingMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AddressingMode::Direct => write!(f, "Direct"),
            AddressingMode::Indirect => write!(f, "Indirect"),
            AddressingMode::Immediate => write!(f, "Immediate"),
        }
    }
}

impl fmt::Debug for InstructionCall {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}(", self.opcode)?;
        for (i, operand) in self.operands.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "expr{:?}", operand.0)?;
        }
        write!(f, ")")
    }
}

impl fmt::Debug for ArrayAccess {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ArrayAccess(expr{:?}[expr{:?}])", self.array.0, self.index.0)
    }
}

impl fmt::Debug for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Instruction {{ id: {:?}, opcode: {:?}", self.id, self.opcode)?;

        if let Some(operand) = &self.operand {
            write!(f, ", operand: expr{:?}", operand.0)?;
        }

        if let Some(label_name) = &self.label_name {
            write!(f, ", label: {:?}", label_name)?;
        }

        write!(f, ", span: {:?}..{:?} }}", self.span.start, self.span.end)
    }
}

impl fmt::Debug for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Label {{ id: {:?}, name: {:?}", self.id, self.name)?;

        if let Some(instruction_id) = &self.instruction_id {
            write!(f, ", instruction: {:?}", instruction_id)?;
        }

        write!(f, ", span: {:?}..{:?} }}", self.span.start, self.span.end)
    }
}
