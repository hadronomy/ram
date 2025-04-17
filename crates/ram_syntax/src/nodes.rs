//! AST node implementations
//!
//! This module contains the implementations of all AST node types.
//! Each struct represents a specific node type in the tree and provides
//! methods for accessing its children and properties.

use crate::ast::{AstChildren, AstNode};
use crate::{ResolvedNode, SyntaxKind};

/// Root node of the AST
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Program(pub(crate) ResolvedNode);

impl Program {
    /// Returns an iterator over the statements in the program
    pub fn statements(&self) -> AstChildren<Statement> {
        AstChildren::<Statement>::new(self.syntax())
    }
}

impl std::fmt::Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.syntax().text())
    }
}

impl AstNode for Program {
    fn can_cast(node: &ResolvedNode) -> bool {
        node.kind() == SyntaxKind::ROOT
    }

    fn cast(node: ResolvedNode) -> Option<Self> {
        if Self::can_cast(&node) { Some(Self(node)) } else { None }
    }

    fn syntax(&self) -> &ResolvedNode {
        &self.0
    }
}

/// Statement node
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Statement(pub(crate) ResolvedNode);

impl Statement {
    /// Returns the instruction if this statement contains one
    pub fn instruction(&self) -> Option<Instruction> {
        AstChildren::<Instruction>::new(self.syntax()).next()
    }

    /// Returns the label definition if this statement contains one
    pub fn label_def(&self) -> Option<LabelDef> {
        AstChildren::<LabelDef>::new(self.syntax()).next()
    }

    /// Returns the comment if this statement contains one
    pub fn comment(&self) -> Option<Comment> {
        AstChildren::<Comment>::new(self.syntax()).next()
    }

    /// Returns the documentation comment if this statement contains one
    pub fn doc_comment(&self) -> Option<DocComment> {
        AstChildren::<DocComment>::new(self.syntax()).next()
    }

    /// Returns the module declaration if this statement contains one
    pub fn mod_stmt(&self) -> Option<ModStmt> {
        AstChildren::<ModStmt>::new(self.syntax()).next()
    }

    /// Returns the module use statement if this statement contains one
    pub fn use_stmt(&self) -> Option<UseStmt> {
        AstChildren::<UseStmt>::new(self.syntax()).next()
    }
}

impl AstNode for Statement {
    fn can_cast(node: &ResolvedNode) -> bool {
        node.kind() == SyntaxKind::STMT
    }

    fn cast(node: ResolvedNode) -> Option<Self> {
        if Self::can_cast(&node) { Some(Self(node)) } else { None }
    }

    fn syntax(&self) -> &ResolvedNode {
        &self.0
    }
}

/// Instruction node
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Instruction(pub(crate) ResolvedNode);

impl Instruction {
    /// Returns the opcode of the instruction
    pub fn opcode(&self) -> Option<String> {
        self.syntax()
            .children_with_tokens()
            .filter_map(cstree::util::NodeOrToken::into_token)
            .find(|token| token.kind() == SyntaxKind::IDENTIFIER)
            .map(|token| token.text().to_string())
    }

    /// Returns the operand of the instruction if it has one
    pub fn operand(&self) -> Option<Operand> {
        AstChildren::<Operand>::new(self.syntax()).next()
    }
}

impl AstNode for Instruction {
    fn can_cast(node: &ResolvedNode) -> bool {
        node.kind() == SyntaxKind::INSTRUCTION
    }

    fn cast(node: ResolvedNode) -> Option<Self> {
        if Self::can_cast(&node) { Some(Self(node)) } else { None }
    }

    fn syntax(&self) -> &ResolvedNode {
        &self.0
    }
}

/// Label definition node
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LabelDef(pub(crate) ResolvedNode);

impl LabelDef {
    /// Returns the name of the label
    pub fn name(&self) -> Option<String> {
        self.syntax()
            .children_with_tokens()
            .filter_map(cstree::util::NodeOrToken::into_token)
            .find(|token| token.kind() == SyntaxKind::IDENTIFIER)
            .map(|token| token.text().to_string())
    }
}

impl AstNode for LabelDef {
    fn can_cast(node: &ResolvedNode) -> bool {
        node.kind() == SyntaxKind::LABEL_DEF
    }

    fn cast(node: ResolvedNode) -> Option<Self> {
        if Self::can_cast(&node) { Some(Self(node)) } else { None }
    }

    fn syntax(&self) -> &ResolvedNode {
        &self.0
    }
}

/// Comment node
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Comment(pub(crate) ResolvedNode);

impl Comment {
    /// Returns the text of the comment
    pub fn text(&self) -> Option<String> {
        self.syntax()
            .children_with_tokens()
            .filter_map(cstree::util::NodeOrToken::into_token)
            .find(|token| token.kind() == SyntaxKind::COMMENT_TEXT)
            .map(|token| token.text().to_string())
    }
}

impl AstNode for Comment {
    fn can_cast(node: &ResolvedNode) -> bool {
        node.kind() == SyntaxKind::COMMENT
    }

    fn cast(node: ResolvedNode) -> Option<Self> {
        if Self::can_cast(&node) { Some(Self(node)) } else { None }
    }

    fn syntax(&self) -> &ResolvedNode {
        &self.0
    }
}

/// Documentation comment node
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DocComment(pub(crate) ResolvedNode);

impl DocComment {
    /// Returns the text of the documentation comment
    pub fn text(&self) -> Option<String> {
        self.syntax()
            .children_with_tokens()
            .filter_map(cstree::util::NodeOrToken::into_token)
            .find(|token| token.kind() == SyntaxKind::COMMENT_TEXT)
            .map(|token| token.text().to_string())
    }
}

impl AstNode for DocComment {
    fn can_cast(node: &ResolvedNode) -> bool {
        node.kind() == SyntaxKind::DOC_COMMENT
    }

    fn cast(node: ResolvedNode) -> Option<Self> {
        if Self::can_cast(&node) { Some(Self(node)) } else { None }
    }

    fn syntax(&self) -> &ResolvedNode {
        &self.0
    }
}

/// Comment group node
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommentGroup(pub(crate) ResolvedNode);

impl CommentGroup {
    /// Returns the comments in this group
    pub fn comments(&self) -> AstChildren<Comment> {
        AstChildren::<Comment>::new(self.syntax())
    }

    /// Returns the documentation comments in this group
    pub fn doc_comments(&self) -> AstChildren<DocComment> {
        AstChildren::<DocComment>::new(self.syntax())
    }
}

impl AstNode for CommentGroup {
    fn can_cast(node: &ResolvedNode) -> bool {
        node.kind() == SyntaxKind::COMMENT_GROUP
    }

    fn cast(node: ResolvedNode) -> Option<Self> {
        if Self::can_cast(&node) { Some(Self(node)) } else { None }
    }

    fn syntax(&self) -> &ResolvedNode {
        &self.0
    }
}

/// Base operand node
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Operand(pub(crate) ResolvedNode);

impl Operand {
    /// Returns the direct operand if this is a direct operand
    pub fn as_direct(&self) -> Option<DirectOperand> {
        if DirectOperand::can_cast(self.syntax()) {
            Some(DirectOperand(self.0.clone()))
        } else {
            None
        }
    }

    /// Returns the indirect operand if this is an indirect operand
    pub fn as_indirect(&self) -> Option<IndirectOperand> {
        if IndirectOperand::can_cast(self.syntax()) {
            Some(IndirectOperand(self.0.clone()))
        } else {
            None
        }
    }

    /// Returns the immediate operand if this is an immediate operand
    pub fn as_immediate(&self) -> Option<ImmediateOperand> {
        if ImmediateOperand::can_cast(self.syntax()) {
            Some(ImmediateOperand(self.0.clone()))
        } else {
            None
        }
    }

    /// Returns the value of the operand
    pub fn value(&self) -> Option<OperandValue> {
        AstChildren::<OperandValue>::new(self.syntax()).next()
    }
}

impl AstNode for Operand {
    fn can_cast(node: &ResolvedNode) -> bool {
        node.kind() == SyntaxKind::OPERAND
    }

    fn cast(node: ResolvedNode) -> Option<Self> {
        if Self::can_cast(&node) { Some(Self(node)) } else { None }
    }

    fn syntax(&self) -> &ResolvedNode {
        &self.0
    }
}

/// Direct operand node (e.g., 5)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DirectOperand(pub(crate) ResolvedNode);

impl DirectOperand {
    /// Returns the value of the operand
    pub fn value(&self) -> Option<OperandValue> {
        AstChildren::<OperandValue>::new(self.syntax()).next()
    }
}

impl AstNode for DirectOperand {
    fn can_cast(node: &ResolvedNode) -> bool {
        node.kind() == SyntaxKind::DIRECT_OPERAND
    }

    fn cast(node: ResolvedNode) -> Option<Self> {
        if Self::can_cast(&node) { Some(Self(node)) } else { None }
    }

    fn syntax(&self) -> &ResolvedNode {
        &self.0
    }
}

/// Indirect operand node (e.g., *5)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IndirectOperand(pub(crate) ResolvedNode);

impl IndirectOperand {
    /// Returns the value of the operand
    pub fn value(&self) -> Option<OperandValue> {
        AstChildren::<OperandValue>::new(self.syntax()).next()
    }
}

impl AstNode for IndirectOperand {
    fn can_cast(node: &ResolvedNode) -> bool {
        node.kind() == SyntaxKind::INDIRECT_OPERAND
    }

    fn cast(node: ResolvedNode) -> Option<Self> {
        if Self::can_cast(&node) { Some(Self(node)) } else { None }
    }

    fn syntax(&self) -> &ResolvedNode {
        &self.0
    }
}

/// Immediate operand node (e.g., =5)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImmediateOperand(pub(crate) ResolvedNode);

impl ImmediateOperand {
    /// Returns the value of the operand
    pub fn value(&self) -> Option<OperandValue> {
        AstChildren::<OperandValue>::new(self.syntax()).next()
    }
}

impl AstNode for ImmediateOperand {
    fn can_cast(node: &ResolvedNode) -> bool {
        node.kind() == SyntaxKind::IMMEDIATE_OPERAND
    }

    fn cast(node: ResolvedNode) -> Option<Self> {
        if Self::can_cast(&node) { Some(Self(node)) } else { None }
    }

    fn syntax(&self) -> &ResolvedNode {
        &self.0
    }
}

/// Operand value node
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OperandValue(pub(crate) ResolvedNode);

impl OperandValue {
    /// Returns the numeric value if this is a number
    pub fn as_number(&self) -> Option<i64> {
        self.syntax()
            .children_with_tokens()
            .filter_map(cstree::util::NodeOrToken::into_token)
            .find(|token| token.kind() == SyntaxKind::NUMBER)
            .and_then(|token| token.text().parse::<i64>().ok())
    }

    /// Returns the identifier value if this is an identifier
    pub fn as_identifier(&self) -> Option<String> {
        self.syntax()
            .children_with_tokens()
            .filter_map(cstree::util::NodeOrToken::into_token)
            .find(|token| token.kind() == SyntaxKind::IDENTIFIER)
            .map(|token| token.text().to_string())
    }

    /// Returns the array accessor if this has one
    pub fn array_accessor(&self) -> Option<ArrayAccessor> {
        AstChildren::<ArrayAccessor>::new(self.syntax()).next()
    }
}

impl AstNode for OperandValue {
    fn can_cast(node: &ResolvedNode) -> bool {
        node.kind() == SyntaxKind::OPERAND_VALUE
    }

    fn cast(node: ResolvedNode) -> Option<Self> {
        if Self::can_cast(&node) { Some(Self(node)) } else { None }
    }

    fn syntax(&self) -> &ResolvedNode {
        &self.0
    }
}

/// Array accessor node (e.g., [5])
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArrayAccessor(pub(crate) ResolvedNode);

impl ArrayAccessor {
    /// Returns the index value
    pub fn index(&self) -> Option<i64> {
        self.syntax()
            .children_with_tokens()
            .filter_map(cstree::util::NodeOrToken::into_token)
            .find(|token| token.kind() == SyntaxKind::NUMBER)
            .and_then(|token| token.text().parse::<i64>().ok())
    }
}

impl AstNode for ArrayAccessor {
    fn can_cast(node: &ResolvedNode) -> bool {
        node.kind() == SyntaxKind::ARRAY_ACCESSOR
    }

    fn cast(node: ResolvedNode) -> Option<Self> {
        if Self::can_cast(&node) { Some(Self(node)) } else { None }
    }

    fn syntax(&self) -> &ResolvedNode {
        &self.0
    }
}

/// Module declaration statement node
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModStmt(pub(crate) ResolvedNode);

impl ModStmt {
    /// Returns the name of the module
    pub fn name(&self) -> Option<String> {
        self.syntax()
            .children_with_tokens()
            .filter_map(cstree::util::NodeOrToken::into_token)
            .find(|token| token.kind() == SyntaxKind::IDENTIFIER)
            .map(|token| token.text().to_string())
    }
}

impl AstNode for ModStmt {
    fn can_cast(node: &ResolvedNode) -> bool {
        node.kind() == SyntaxKind::MOD_STMT
    }

    fn cast(node: ResolvedNode) -> Option<Self> {
        if Self::can_cast(&node) { Some(Self(node)) } else { None }
    }

    fn syntax(&self) -> &ResolvedNode {
        &self.0
    }
}

/// Module use statement node
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UseStmt(pub(crate) ResolvedNode);

impl UseStmt {
    /// Returns the module path
    pub fn path(&self) -> Option<ModulePath> {
        AstChildren::<ModulePath>::new(self.syntax()).next()
    }
}

impl AstNode for UseStmt {
    fn can_cast(node: &ResolvedNode) -> bool {
        node.kind() == SyntaxKind::USE_STMT
    }

    fn cast(node: ResolvedNode) -> Option<Self> {
        if Self::can_cast(&node) { Some(Self(node)) } else { None }
    }

    fn syntax(&self) -> &ResolvedNode {
        &self.0
    }
}

/// Module path node
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModulePath(pub(crate) ResolvedNode);

impl ModulePath {
    /// Returns the path as a string
    pub fn as_string(&self) -> Option<String> {
        self.syntax()
            .children_with_tokens()
            .filter_map(cstree::util::NodeOrToken::into_token)
            .find(|token| token.kind() == SyntaxKind::STRING)
            .map(|token| token.text().to_string())
    }
}

impl AstNode for ModulePath {
    fn can_cast(node: &ResolvedNode) -> bool {
        node.kind() == SyntaxKind::MODULE_PATH
    }

    fn cast(node: ResolvedNode) -> Option<Self> {
        if Self::can_cast(&node) { Some(Self(node)) } else { None }
    }

    fn syntax(&self) -> &ResolvedNode {
        &self.0
    }
}
