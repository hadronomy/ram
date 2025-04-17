//! AST wrappers around [`SyntaxNode`]
//!
//! This module provides a typed API for working with the syntax tree.
//! Each struct represents a specific node type in the tree and provides
//! methods for accessing its children and properties.

use crate::{ResolvedNode, SyntaxKind};

/// Trait for all AST node types
pub trait AstNode {
    /// Checks if the given node can be cast to this type
    fn can_cast(node: &ResolvedNode) -> bool;

    /// Attempts to cast the node to this type
    fn cast(node: ResolvedNode) -> Option<Self>
    where
        Self: Sized;

    /// Returns the underlying syntax node
    fn syntax(&self) -> &ResolvedNode;
}

/// Root node of the AST
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Program(ResolvedNode);

impl Program {
    /// Returns an iterator over the statements in the program
    pub fn statements(&self) -> impl Iterator<Item = Statement> + '_ {
        self.syntax().children().filter_map(|node| Statement::cast(node.clone()))
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
pub struct Statement(ResolvedNode);

impl Statement {
    /// Returns the instruction if this statement contains one
    pub fn instruction(&self) -> Option<Instruction> {
        self.syntax().children().find_map(|node| Instruction::cast(node.clone()))
    }

    /// Returns the label definition if this statement contains one
    pub fn label_def(&self) -> Option<LabelDef> {
        self.syntax().children().find_map(|node| LabelDef::cast(node.clone()))
    }

    /// Returns the comment if this statement contains one
    pub fn comment(&self) -> Option<Comment> {
        self.syntax().children().find_map(|node| Comment::cast(node.clone()))
    }

    /// Returns the documentation comment if this statement contains one
    pub fn doc_comment(&self) -> Option<DocComment> {
        self.syntax().children().find_map(|node| DocComment::cast(node.clone()))
    }

    /// Returns the module declaration if this statement contains one
    pub fn mod_stmt(&self) -> Option<ModStmt> {
        self.syntax().children().find_map(|node| ModStmt::cast(node.clone()))
    }

    /// Returns the module use statement if this statement contains one
    pub fn use_stmt(&self) -> Option<UseStmt> {
        self.syntax().children().find_map(|node| UseStmt::cast(node.clone()))
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
pub struct Instruction(ResolvedNode);

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
        self.syntax().children().find_map(|node| Operand::cast(node.clone()))
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
pub struct LabelDef(ResolvedNode);

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
pub struct Comment(ResolvedNode);

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
pub struct DocComment(ResolvedNode);

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
pub struct CommentGroup(ResolvedNode);

impl CommentGroup {
    /// Returns the comments in this group
    pub fn comments(&self) -> impl Iterator<Item = Comment> + '_ {
        self.syntax().children().filter_map(|node| Comment::cast(node.clone()))
    }

    /// Returns the documentation comments in this group
    pub fn doc_comments(&self) -> impl Iterator<Item = DocComment> + '_ {
        self.syntax().children().filter_map(|node| DocComment::cast(node.clone()))
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
pub struct Operand(ResolvedNode);

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
        self.syntax().children().find_map(|node| OperandValue::cast(node.clone()))
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
pub struct DirectOperand(ResolvedNode);

impl DirectOperand {
    /// Returns the value of the operand
    pub fn value(&self) -> Option<OperandValue> {
        self.syntax().children().find_map(|node| OperandValue::cast(node.clone()))
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
pub struct IndirectOperand(ResolvedNode);

impl IndirectOperand {
    /// Returns the value of the operand
    pub fn value(&self) -> Option<OperandValue> {
        self.syntax().children().find_map(|node| OperandValue::cast(node.clone()))
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
pub struct ImmediateOperand(ResolvedNode);

impl ImmediateOperand {
    /// Returns the value of the operand
    pub fn value(&self) -> Option<OperandValue> {
        self.syntax().children().find_map(|node| OperandValue::cast(node.clone()))
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
pub struct OperandValue(ResolvedNode);

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
        self.syntax().children().find_map(|node| ArrayAccessor::cast(node.clone()))
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
pub struct ArrayAccessor(ResolvedNode);

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
pub struct ModStmt(ResolvedNode);

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
pub struct UseStmt(ResolvedNode);

impl UseStmt {
    /// Returns the module path
    pub fn path(&self) -> Option<ModulePath> {
        self.syntax().children().find_map(|node| ModulePath::cast(node.clone()))
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
pub struct ModulePath(ResolvedNode);

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
