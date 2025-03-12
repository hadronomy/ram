//! This code is heavily based on [rust-analyzer](https://github.com/rust-analyzer/rust-analyzer)
//! implementation

mod event;
mod input;
mod parser;
mod syntax_kind;

pub use event::*;
pub use input::*;
pub use parser::*;
pub use syntax_kind::*;
