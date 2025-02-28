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
//! ```
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
//! The module also provides syntax highlighting definitions for integration with
//! the Syntect library.

use std::sync::OnceLock;

use chumsky::prelude::*;
use miette::highlighters::SyntectHighlighter;
#[cfg(feature = "serde")]
use serde::Serialize;
use syntect::highlighting::ThemeSet;
use syntect::parsing::{SyntaxDefinition, SyntaxSet, SyntaxSetBuilder};

// --- Data Structures ---

/// Holds the `.sublime-syntax` definition for RAM assembly language.
pub static SUBLIME_SYNTAX: &str = include_str!("ram.sublime-syntax");

/// Stores the loaded syntax definition for RAM assembly language.
pub static SYNTAX_DEFINITION: OnceLock<SyntaxDefinition> = OnceLock::new();

pub fn syntax_definition() -> &'static SyntaxDefinition {
    SYNTAX_DEFINITION.get_or_init(|| {
        SyntaxDefinition::load_from_str(SUBLIME_SYNTAX, false, Some("source.ram")).unwrap()
    })
}

pub fn syntax_set() -> SyntaxSet {
    let mut set = SyntaxSetBuilder::new();
    set.add(syntax_definition().to_owned());
    set.build()
}

pub fn highlighter() -> SyntectHighlighter {
    let theme = &ThemeSet::load_defaults().themes["base16-mocha.dark"];
    SyntectHighlighter::new(syntax_set(), theme.clone(), false)
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Program {
    lines: Vec<Line>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum Line {
    Instruction { instruction: Instruction, comment: Option<String> },
    LabelDefinition { label: String, instruction: Instruction, comment: Option<String> },
    Comment(String),
    Invalid,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum Instruction {
    Basic { opcode: String, arg: Option<Operand> },
    Invalid,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum Operand {
    Number(NumberOperand),
    Indirect(IndirectOperand),
    Immediate(ImmediateOperand),
    Label(String),
    Invalid,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct NumberOperand {
    value: u64,
    accessor: Option<Accessor>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct IndirectOperand {
    value: u64,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct ImmediateOperand {
    value: u64,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Accessor {
    index: Box<Operand>,
}

// --- Parser ---

pub fn parser() -> impl Parser<char, Program, Error = Simple<char>> {
    program_parser()
}

fn program_parser() -> impl Parser<char, Program, Error = Simple<char>> {
    line_parser()
        .padded()
        .separated_by(newline_parser().or(inline_whitespace_parser()))
        .allow_trailing()
        .map(|lines| Program {
            lines: lines.into_iter().filter(|line| !matches!(line, Line::Invalid)).collect(),
        })
        .recover_with(skip_parser(end().map(|_| Program { lines: vec![] })))
        .labelled("program")
}

fn line_parser() -> impl Parser<char, Line, Error = Simple<char>> {
    choice((
        label_definition_parser(),
        instruction_parser().map(|inst| Line::Instruction { instruction: inst, comment: None }),
        comment_parser().map(Line::Comment),
    ))
    .padded()
    // Modify the recovery to ensure it doesn't eat our unclosed bracket errors
    .recover_with(skip_then_retry_until(['\n']))
    .labelled("line")
}

fn label_definition_parser() -> impl Parser<char, Line, Error = Simple<char>> {
    identifier_parser()
        .then_ignore(just(':').padded())
        .then(instruction_parser().padded())
        .then(comment_parser().or_not())
        .map(|((label, instruction), comment)| Line::LabelDefinition {
            label,
            instruction,
            comment,
        })
        .labelled("label definition")
}

fn instruction_parser() -> impl Parser<char, Instruction, Error = Simple<char>> {
    identifier_parser()
        .padded()
        .then(operand_parser().padded().or_not())
        // Make sure we don't discard the operand errors
        .map(|(opcode, arg)| Instruction::Basic { opcode, arg })
        .labelled("instruction")
}

fn operand_parser() -> impl Parser<char, Operand, Error = Simple<char>> {
    recursive(|operand: Recursive<'_, char, Operand, Simple<char>>| {
        // We need to detect unclosed brackets
        let opening_bracket = just('[').map_with_span(|_, span| span).labelled("opening bracket");

        let closing_bracket = just(']').labelled("closing bracket");

        // Create a more specific accessor parser that reports unclosed brackets
        let accessor = opening_bracket
            .then(operand.clone().padded())
            // Using `then_with` to keep track of the opening bracket span
            .then_with(move |state| {
                let (opening_span, inner) = state;
                closing_bracket
                    // If closing bracket is missing, create an unclosed delimiter error
                    .map_err(move |_| {
                        Simple::unclosed_delimiter(
                            opening_span.clone(),
                            '[',
                            opening_span.clone(),
                            ']',
                            None,
                        )
                    })
                    .map(move |_| Accessor { index: Box::new(inner.clone()) })
            })
            .labelled("array accessor");

        // Now ensure the direct number parser properly handles array accessors
        let direct = number_parser()
            .then(accessor.or_not())
            .map(|(value, acc)| Operand::Number(NumberOperand { value, accessor: acc }))
            .labelled("direct operand");

        // Rest of the parsers remain the same
        let indirect = just('*')
            .padded()
            .ignore_then(number_parser())
            .map(|value| Operand::Indirect(IndirectOperand { value }))
            .labelled("indirect operand");

        let immediate = just('=')
            .padded()
            .ignore_then(number_parser())
            .map(|value| Operand::Immediate(ImmediateOperand { value }))
            .labelled("immediate operand");

        let label_operand = identifier_parser().map(Operand::Label).labelled("label operand");

        // Don't recover with nested_delimiters here - let the error propagate up
        choice((direct, immediate, indirect, label_operand))
            // Remove the nested_delimiters recovery which is consuming the unclosed bracket error
            //.recover_with(nested_delimiters('[', ']', [], |_| Operand::Invalid))
            .recover_with(skip_then_retry_until(['\n', ' ', '#']))
            .labelled("operand")
    })
}

fn comment_parser() -> impl Parser<char, String, Error = Simple<char>> {
    just('#')
        .then(take_until(newline_parser().or(end())))
        .map(|(_, (chars, _))| chars.into_iter().collect::<String>().trim().to_string())
        .labelled("comment")
}

fn identifier_parser() -> impl Parser<char, String, Error = Simple<char>> {
    filter(|c: &char| c.is_alphabetic())
        .chain(filter(|c: &char| c.is_alphanumeric() || *c == '_').repeated())
        .collect::<String>()
        .labelled("identifier")
}

fn number_parser() -> impl Parser<char, u64, Error = Simple<char>> {
    text::int(10)
        .map(|s: String| s.parse().unwrap()) // TODO: Handle parse errors properly
        .labelled("number")
}

fn newline_parser() -> impl Parser<char, (), Error = Simple<char>> {
    text::newline().repeated().at_least(1).ignored().labelled("newline")
}

fn inline_whitespace_parser() -> impl Parser<char, (), Error = Simple<char>> {
    filter(|c: &char| c.is_whitespace()).repeated().ignored()
}
