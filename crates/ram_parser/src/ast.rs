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
        kind == SyntaxKind::ROOT
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
    /// Returns an iterator over all statements in the program.
    pub fn statements(&self) -> impl Iterator<Item = Statement> {
        self.0.children().filter_map(Statement::cast)
    }

    /// Returns an iterator over all lines in the program (legacy method).
    #[deprecated(since = "0.2.0", note = "Use statements() instead")]
    pub fn lines(&self) -> impl Iterator<Item = Statement> {
        self.statements()
    }
}

// --- Statement ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Statement(SyntaxNode);

impl AstNode for Statement {
    type Language = RamLang;

    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::STMT
    }

    fn cast(node: SyntaxNode) -> Option<Self> {
        if Self::can_cast(node.kind()) { Some(Self(node)) } else { None }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.0
    }
}

impl RamAstNode for Statement {}

impl Statement {
    /// Returns the instruction in this statement, if any.
    pub fn instruction(&self) -> Option<Instruction> {
        self.0.children().find_map(Instruction::cast)
    }

    /// Returns the label definition in this statement, if any.
    pub fn label_def(&self) -> Option<LabelDef> {
        self.0.children().find_map(LabelDef::cast)
    }

    /// Returns the regular comment in this statement, if any.
    pub fn comment(&self) -> Option<Comment> {
        self.0.children().find_map(Comment::cast)
    }

    /// Returns the documentation comment in this statement, if any.
    pub fn doc_comment(&self) -> Option<DocComment> {
        self.0.children().find_map(DocComment::cast)
    }

    /// Returns the comment group in this statement, if any.
    pub fn comment_group(&self) -> Option<CommentGroup> {
        self.0.children().find_map(CommentGroup::cast)
    }

    /// Returns any type of comment in this statement (regular or documentation).
    pub fn any_comment(&self) -> Option<AnyComment> {
        self.doc_comment().map(AnyComment::Doc).or_else(|| self.comment().map(AnyComment::Regular))
    }
}

// --- Instruction ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Instruction(SyntaxNode);

impl AstNode for Instruction {
    type Language = RamLang;

    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::INSTRUCTION
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
/// A label definition in the program.
///
/// A label definition must be followed by an instruction, either on the same line
/// or on a subsequent line.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LabelDef(SyntaxNode);

impl AstNode for LabelDef {
    type Language = RamLang;

    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::LABEL_DEF
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
            .find(|token| token.kind() == SyntaxKind::IDENTIFIER)
    }

    /// Returns the label name as a string.
    pub fn name(&self) -> Option<String> {
        self.name_token().map(|t| t.text().to_string())
    }
}

// --- Comment Types ---

/// Enum representing any type of comment (regular or documentation)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AnyComment {
    /// A regular comment (starting with #)
    Regular(Comment),
    /// A documentation comment (starting with #*)
    Doc(DocComment),
}

impl AnyComment {
    /// Returns the comment text as a string, regardless of comment type.
    pub fn text(&self) -> Option<String> {
        match self {
            AnyComment::Regular(comment) => comment.text(),
            AnyComment::Doc(doc_comment) => doc_comment.text(),
        }
    }

    /// Returns true if this is a documentation comment.
    pub fn is_doc_comment(&self) -> bool {
        matches!(self, AnyComment::Doc(_))
    }

    /// Returns true if this is a regular comment.
    pub fn is_regular_comment(&self) -> bool {
        matches!(self, AnyComment::Regular(_))
    }
}

// --- Comment Group ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommentGroup(SyntaxNode);

impl AstNode for CommentGroup {
    type Language = RamLang;

    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::COMMENT_GROUP
    }

    fn cast(node: SyntaxNode) -> Option<Self> {
        if Self::can_cast(node.kind()) { Some(Self(node)) } else { None }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.0
    }
}

impl RamAstNode for CommentGroup {}

impl CommentGroup {
    /// Returns all regular comments in this group.
    pub fn comments(&self) -> impl Iterator<Item = Comment> {
        self.0.children().filter_map(Comment::cast)
    }

    /// Returns all documentation comments in this group.
    pub fn doc_comments(&self) -> impl Iterator<Item = DocComment> {
        self.0.children().filter_map(DocComment::cast)
    }

    /// Returns all comments in this group (both regular and documentation).
    pub fn all_comments(&self) -> impl Iterator<Item = AnyComment> {
        let regular = self.comments().map(AnyComment::Regular);
        let docs = self.doc_comments().map(AnyComment::Doc);
        regular.chain(docs)
    }

    /// Returns true if this group contains at least one documentation comment.
    pub fn has_doc_comments(&self) -> bool {
        self.doc_comments().next().is_some()
    }

    /// Returns all comment texts as a vector of strings.
    pub fn comment_texts(&self) -> Vec<String> {
        self.all_comments().filter_map(|c| c.text()).collect()
    }
}

// --- Regular Comment ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Comment(SyntaxNode);

impl AstNode for Comment {
    type Language = RamLang;

    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::COMMENT
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
            .find(|token| token.kind() == SyntaxKind::HASH)
    }

    /// Returns the comment text token (content after '#').
    pub fn text_token(&self) -> Option<SyntaxToken> {
        self.0
            .children_with_tokens()
            .filter_map(SyntaxElement::into_token)
            .find(|token| token.kind() == SyntaxKind::COMMENT_TEXT)
    }

    /// Returns the comment text as a string.
    pub fn text(&self) -> Option<String> {
        self.text_token().map(|t| t.text().to_string())
    }
}

// --- Documentation Comment ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DocComment(SyntaxNode);

impl AstNode for DocComment {
    type Language = RamLang;

    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::DOC_COMMENT
    }

    fn cast(node: SyntaxNode) -> Option<Self> {
        if Self::can_cast(node.kind()) { Some(Self(node)) } else { None }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.0
    }
}

impl RamAstNode for DocComment {}

impl DocComment {
    /// Returns the '#*' token.
    pub fn hash_star_token(&self) -> Option<SyntaxToken> {
        self.0
            .children_with_tokens()
            .filter_map(SyntaxElement::into_token)
            .find(|token| token.kind() == SyntaxKind::HASH_STAR)
    }

    /// Returns the comment text token (content after '#*').
    pub fn text_token(&self) -> Option<SyntaxToken> {
        self.0
            .children_with_tokens()
            .filter_map(SyntaxElement::into_token)
            .find(|token| token.kind() == SyntaxKind::COMMENT_TEXT)
    }

    /// Returns the comment text as a string.
    pub fn text(&self) -> Option<String> {
        self.text_token().map(|t| t.text().to_string())
    }

    /// Returns the documentation text as a string, with leading/trailing whitespace trimmed.
    /// This is useful for extracting the actual documentation content.
    pub fn doc_text(&self) -> Option<String> {
        self.text().map(|s| s.trim().to_string())
    }
}

// --- Operand ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Operand(SyntaxNode);

impl AstNode for Operand {
    type Language = RamLang;

    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::OPERAND
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
        kind == SyntaxKind::DIRECT_OPERAND
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
        kind == SyntaxKind::INDIRECT_OPERAND
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
            .find(|token| token.kind() == SyntaxKind::STAR)
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
        kind == SyntaxKind::IMMEDIATE_OPERAND
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
            .find(|token| token.kind() == SyntaxKind::EQUALS)
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
        kind == SyntaxKind::OPERAND_VALUE
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
            token.kind() == SyntaxKind::NUMBER || token.kind() == SyntaxKind::IDENTIFIER
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
