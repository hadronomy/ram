//! This code is heavily based on [rust-analyzer](https://github.com/rust-analyzer/rust-analyzer)
//! implementation

mod ast;
mod event;
mod grammar;
mod language;
mod parser;

#[cfg(test)]
mod tests;

mod syntax_kind;
mod tree_builder;

pub use ast::{
    Comment, DirectOperand, ImmediateOperand, IndirectOperand, Instruction, LabelDef, Line,
    Operand, OperandValue, Program, RamAstNode,
};
pub use event::Event;
pub use language::{
    RamLang, SyntaxElement, SyntaxElementChildren, SyntaxNode, SyntaxNodeChildren, SyntaxToken,
};
pub use parser::{SyntaxError, Token, convert_errors, parse};
pub use syntax_kind::SyntaxKind;
pub use tree_builder::build_tree;
