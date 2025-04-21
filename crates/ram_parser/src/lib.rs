//! This code is heavily based on [rust-analyzer](https://github.com/rust-analyzer/rust-analyzer)
//! implementation

pub mod diagnostic;
pub mod event;
mod grammar;
pub mod lexer;
pub mod parser;
mod tree_builder;

#[cfg(test)]
mod tests;

pub use diagnostic::{Diagnostic, DiagnosticBuilder, DiagnosticKind};
pub use event::Event;
pub use lexer::Token;
pub use parser::{convert_errors, parse};
pub use ram_syntax::*;
pub use tree_builder::build_tree;
