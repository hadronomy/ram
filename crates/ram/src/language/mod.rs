//! # RAM Assembly Language
//!
//! This module provides the parser, data structures, and utility functions for the RAM
//! (Random Access Machine) assembly language. RAM is a simple, instruction-based language
//! designed for educational purposes to simulate a basic computer architecture.
//!
//! ## Language Syntax
//!
//! RAM assembly consists of:
//!
//! - **Instructions**: Operations like `LOAD`, `ADD`, etc. with optional operands
//! - **Labels**: Named positions in code (e.g., `loop:`)
//! - **Comments**: Starting with `#`
//!
//! ## Operand Types
//!
//! - **Direct**: A memory address (e.g., `5`)
//! - **Indirect**: Value at the memory address pointed by the operand (e.g., `*5`)
//! - **Immediate**: The literal value (e.g., `=5`)
//! - **Label**: Reference to a labeled position in code
//!
//! Operands can also include array-like accessors with the syntax `base[index]`.
//!
//! ## Example
//!
//! ```ignore
//! # Simple RAM program that adds two numbers
//! LOAD 1    # Load value from address 1
//! ADD 2     # Add value from address 2
//! STORE 3   # Store result in address 3
//! HALT      # Stop execution
//! ```
//!
//! ## Implementation
//!
//! This module uses the Chumsky parser combinator library to parse RAM assembly code
//! into an abstract syntax tree (AST). The AST can then be used for execution,
//! analysis, or further processing.
//!
//! The main data structures include:
//! - `Program`: The root AST node containing a sequence of lines
//! - `Line`: Represents either an instruction, label definition, or comment
//! - `Instruction`: Contains an opcode and optional operand
//! - `Operand`: Represents different addressing modes (direct, indirect, immediate)
//!
//! The module also provides syntax [`highlighting`] definitions for integration with
//! the Syntect library.

pub use highlighting::*;
pub use parser::*;
pub use ram_parser::*;

pub mod highlighting;
pub mod parser;
