//! This code is heavily based on [rust-analyzer](https://github.com/rust-analyzer/rust-analyzer)
//! implementation

pub mod ast;
mod diagnostic;
mod event;
mod grammar;
mod lexer;
mod parser;

#[cfg(test)]
mod tests;

mod syntax_kind;
mod tree_builder;

pub use ast::*;
pub use diagnostic::{Diagnostic, DiagnosticBuilder, DiagnosticKind};
pub use event::Event;
pub use lexer::Token;
pub use parser::{convert_errors, parse};
pub use syntax_kind::*;
pub use tree_builder::build_tree;
