use rowan::ast::AstNode;

use crate::SyntaxKind;
use crate::language::{RamLang, SyntaxElement, SyntaxNode, SyntaxToken};

// --- AST Node Trait (Marker Trait) ---
pub trait RamAstNode: AstNode<Language = RamLang> {}

// --- Top Level Program ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Program(SyntaxNode);

impl AstNode for Program {
    type Language = RamLang;

    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::Root
    }

    fn cast(node: SyntaxNode) -> Option<Self> {
        if Self::can_cast(node.kind()) { Some(Self(node)) } else { None }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.0
    }
}

impl RamAstNode for Program {}

impl Program {
    /// Returns an iterator over all lines in the program.
    pub fn lines(&self) -> impl Iterator<Item = Line> {
        self.0.children().filter_map(Line::cast)
    }
}

// --- Line ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Line(SyntaxNode);

impl AstNode for Line {
    type Language = RamLang;

    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::Line
    }

    fn cast(node: SyntaxNode) -> Option<Self> {
        if Self::can_cast(node.kind()) { Some(Self(node)) } else { None }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.0
    }
}

impl RamAstNode for Line {}

impl Line {
    /// Returns the instruction in this line, if any.
    pub fn instruction(&self) -> Option<Instruction> {
        self.0.children().find_map(Instruction::cast)
    }

    /// Returns the label definition in this line, if any.
    pub fn label_def(&self) -> Option<LabelDef> {
        self.0.children().find_map(LabelDef::cast)
    }

    /// Returns the comment in this line, if any.
    pub fn comment(&self) -> Option<Comment> {
        self.0.children().find_map(Comment::cast)
    }
}

// --- Instruction ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Instruction(SyntaxNode);

impl AstNode for Instruction {
    type Language = RamLang;

    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::Instruction
    }

    fn cast(node: SyntaxNode) -> Option<Self> {
        if Self::can_cast(node.kind()) { Some(Self(node)) } else { None }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.0
    }
}

impl RamAstNode for Instruction {}

impl Instruction {
    /// Returns the opcode token (e.g., LOAD, ADD).
    pub fn opcode_token(&self) -> Option<SyntaxToken> {
        self.0
            .children_with_tokens()
            .filter_map(SyntaxElement::into_token)
            .find(|token| token.kind().is_keyword())
    }

    /// Returns the opcode kind.
    pub fn opcode(&self) -> Option<SyntaxKind> {
        self.opcode_token().map(|t| t.kind())
    }

    /// Returns the operand node, if present.
    pub fn operand(&self) -> Option<Operand> {
        self.0.children().find_map(Operand::cast)
    }
}

// --- Label Definition ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LabelDef(SyntaxNode);

impl AstNode for LabelDef {
    type Language = RamLang;

    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::LabelDef
    }

    fn cast(node: SyntaxNode) -> Option<Self> {
        if Self::can_cast(node.kind()) { Some(Self(node)) } else { None }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.0
    }
}

impl RamAstNode for LabelDef {}

impl LabelDef {
    /// Returns the identifier token for the label name.
    pub fn name_token(&self) -> Option<SyntaxToken> {
        self.0
            .children_with_tokens()
            .filter_map(SyntaxElement::into_token)
            .find(|token| token.kind() == SyntaxKind::Identifier)
    }

    /// Returns the label name as a string.
    pub fn name(&self) -> Option<String> {
        self.name_token().map(|t| t.text().to_string())
    }
}

// --- Comment ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Comment(SyntaxNode);

impl AstNode for Comment {
    type Language = RamLang;

    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::Comment
    }

    fn cast(node: SyntaxNode) -> Option<Self> {
        if Self::can_cast(node.kind()) { Some(Self(node)) } else { None }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.0
    }
}

impl RamAstNode for Comment {}

impl Comment {
    /// Returns the '#' token.
    pub fn hash_token(&self) -> Option<SyntaxToken> {
        self.0
            .children_with_tokens()
            .filter_map(SyntaxElement::into_token)
            .find(|token| token.kind() == SyntaxKind::Hash)
    }

    /// Returns the comment text token (content after '#').
    pub fn text_token(&self) -> Option<SyntaxToken> {
        self.0
            .children_with_tokens()
            .filter_map(SyntaxElement::into_token)
            .find(|token| token.kind() == SyntaxKind::CommentText)
    }

    /// Returns the comment text as a string.
    pub fn text(&self) -> Option<String> {
        self.text_token().map(|t| t.text().to_string())
    }
}

// --- Operand ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Operand(SyntaxNode);

impl AstNode for Operand {
    type Language = RamLang;

    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::Operand
    }

    fn cast(node: SyntaxNode) -> Option<Self> {
        if Self::can_cast(node.kind()) { Some(Self(node)) } else { None }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.0
    }
}

impl RamAstNode for Operand {}

impl Operand {
    /// Returns the direct operand if this is a direct operand.
    pub fn direct(&self) -> Option<DirectOperand> {
        self.0.children().find_map(DirectOperand::cast)
    }

    /// Returns the indirect operand if this is an indirect operand.
    pub fn indirect(&self) -> Option<IndirectOperand> {
        self.0.children().find_map(IndirectOperand::cast)
    }

    /// Returns the immediate operand if this is an immediate operand.
    pub fn immediate(&self) -> Option<ImmediateOperand> {
        self.0.children().find_map(ImmediateOperand::cast)
    }

    /// Returns true if this is an indirect operand (*)
    pub fn is_indirect(&self) -> bool {
        self.indirect().is_some()
    }

    /// Returns true if this is an immediate operand (=)
    pub fn is_immediate(&self) -> bool {
        self.immediate().is_some()
    }

    /// Returns true if this is a direct operand (no prefix)
    pub fn is_direct(&self) -> bool {
        self.direct().is_some()
    }

    /// Returns the core value node (Number or Identifier).
    pub fn value(&self) -> Option<OperandValue> {
        if let Some(direct) = self.direct() {
            direct.value()
        } else if let Some(indirect) = self.indirect() {
            indirect.value()
        } else if let Some(immediate) = self.immediate() {
            immediate.value()
        } else {
            None
        }
    }
}

// --- Direct Operand ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DirectOperand(SyntaxNode);

impl AstNode for DirectOperand {
    type Language = RamLang;

    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::DirectOperand
    }

    fn cast(node: SyntaxNode) -> Option<Self> {
        if Self::can_cast(node.kind()) { Some(Self(node)) } else { None }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.0
    }
}

impl RamAstNode for DirectOperand {}

impl DirectOperand {
    /// Returns the core value node (Number or Identifier).
    pub fn value(&self) -> Option<OperandValue> {
        self.0.children().find_map(OperandValue::cast)
    }
}

// --- Indirect Operand ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IndirectOperand(SyntaxNode);

impl AstNode for IndirectOperand {
    type Language = RamLang;

    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::IndirectOperand
    }

    fn cast(node: SyntaxNode) -> Option<Self> {
        if Self::can_cast(node.kind()) { Some(Self(node)) } else { None }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.0
    }
}

impl RamAstNode for IndirectOperand {}

impl IndirectOperand {
    /// Returns the star token.
    pub fn star_token(&self) -> Option<SyntaxToken> {
        self.0
            .children_with_tokens()
            .filter_map(SyntaxElement::into_token)
            .find(|token| token.kind() == SyntaxKind::Star)
    }

    /// Returns the core value node (Number or Identifier).
    pub fn value(&self) -> Option<OperandValue> {
        self.0.children().find_map(OperandValue::cast)
    }
}

// --- Immediate Operand ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImmediateOperand(SyntaxNode);

impl AstNode for ImmediateOperand {
    type Language = RamLang;

    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::ImmediateOperand
    }

    fn cast(node: SyntaxNode) -> Option<Self> {
        if Self::can_cast(node.kind()) { Some(Self(node)) } else { None }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.0
    }
}

impl RamAstNode for ImmediateOperand {}

impl ImmediateOperand {
    /// Returns the equals token.
    pub fn equals_token(&self) -> Option<SyntaxToken> {
        self.0
            .children_with_tokens()
            .filter_map(SyntaxElement::into_token)
            .find(|token| token.kind() == SyntaxKind::Equals)
    }

    /// Returns the core value node (Number or Identifier).
    pub fn value(&self) -> Option<OperandValue> {
        self.0.children().find_map(OperandValue::cast)
    }
}

// --- Operand Value ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OperandValue(SyntaxNode);

impl AstNode for OperandValue {
    type Language = RamLang;

    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::OperandValue
    }

    fn cast(node: SyntaxNode) -> Option<Self> {
        if Self::can_cast(node.kind()) { Some(Self(node)) } else { None }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.0
    }
}

impl RamAstNode for OperandValue {}

impl OperandValue {
    /// Returns the underlying token (Number or Identifier).
    pub fn token(&self) -> Option<SyntaxToken> {
        self.0.children_with_tokens().filter_map(SyntaxElement::into_token).find(|token| {
            token.kind() == SyntaxKind::Number || token.kind() == SyntaxKind::Identifier
        })
    }

    /// Returns the text content of the value.
    pub fn text(&self) -> Option<String> {
        self.token().map(|t| t.text().to_string())
    }

    /// Returns the kind (Number or Identifier).
    pub fn value_kind(&self) -> Option<SyntaxKind> {
        self.token().map(|t| t.kind())
    }
}
