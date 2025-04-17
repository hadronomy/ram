//! This code is heavily based on [rust-analyzer](https://github.com/rust-analyzer/rust-analyzer)
//! implementation

mod diagnostic;
mod event;
mod grammar;
mod lexer;
mod parser;

#[cfg(test)]
mod tests;

mod tree_builder;

pub use diagnostic::{Diagnostic, DiagnosticBuilder, DiagnosticKind};
pub use event::Event;
pub use lexer::Token;
pub use parser::{convert_errors, parse};
pub use ram_syntax::*;
pub use tree_builder::build_tree;
