//! This is the actual "grammar" of the RAM assembly language.
//!
//! Each function in this module and its children corresponds
//! to a production of the formal grammar. Submodules roughly
//! correspond to different *areas* of the grammar. By convention,
//! each submodule starts with `use super::*` import and exports
//! "public" productions via `pub(super)`.
//!
//! See docs for [`Parser`](super::parser::Parser) to learn about API,
//! available to the grammar, and see docs for [`Event`](super::event::Event)
//! to learn how this actually manages to produce parse trees.
#![allow(clippy::wildcard_imports)]
#![allow(clippy::enum_glob_use)]

use crate::SyntaxKind::*;
use crate::parser::Parser;

pub(crate) mod entry {
    use super::*;

    pub(crate) mod top {
        use super::*;

        /// Program is the root of the AST
        pub(crate) fn program(p: &mut Parser<'_>) {
            let m = p.start();

            while !p.at(EOF) {
                line(p);
            }

            m.complete(p, ROOT);
        }
    }
}

/// Parse a line, which can be an instruction, label definition, or comment.
fn line(p: &mut Parser<'_>) {
    let m = p.start();

    // Skip whitespace at the beginning of the line
    while p.at(WHITESPACE) {
        p.bump_any();
    }

    // Check what kind of line this is
    match p.current() {
        EOF => {
            // End of input, nothing to do
            m.abandon(p);
            return;
        }
        HASH => {
            // Comment line
            comments::comment(p);
        }
        NEWLINE => {
            // Empty line
            p.bump_any(); // Consume newline
        }
        LBRACKET => {
            // Unexpected opening bracket
            p.err_and_bump(
                "Unexpected opening bracket '['",
                "Square brackets can only be used in array accessors after an identifier or number",
            );
        }
        RBRACKET => {
            // Extra closing bracket
            p.err_and_bump(
                "Unexpected closing bracket ']'",
                "This closing bracket doesn't match any opening bracket",
            );
        }
        ERROR_TOKEN => {
            // Error token
            let text = p.token_text().to_string();
            p.err_and_bump(
                &format!("Unexpected character: {text}"),
                "Remove or replace this character",
            );
        }
        IDENTIFIER => {
            // Could be a label definition or an instruction
            if p.at_label_definition_start() {
                // Label definition
                labels::label_definition(p);

                // After a label definition, check for an instruction on the same line
                // Consume whitespace after the label
                while p.at(WHITESPACE) {
                    p.bump_any();
                }

                // Check if there's an instruction after the label
                if p.at_instruction_start() {
                    instructions::instruction(p);
                }
            } else if p.at_instruction_start() {
                // Instruction
                instructions::instruction(p);
            } else {
                // Unexpected identifier
                let text = p.token_text().to_string();
                p.err_and_bump(
                    &format!("Unexpected identifier: {text}"),
                    "Expected an instruction, label, or comment",
                );
            }
        }
        _ if p.at_instruction_start() => {
            // Instruction with keyword
            instructions::instruction(p);
        }
        _ => {
            // Unexpected token
            let text = p.token_text().to_string();
            p.err_and_bump(
                &format!("Unexpected token: {text}"),
                "Expected an instruction, label, or comment",
            );
        }
    }

    // Consume trailing whitespace
    while p.at(WHITESPACE) {
        p.bump_any();
    }

    // Consume trailing comment if present
    if p.at(HASH) {
        comments::comment(p);
    }

    // Consume newline at the end of the line
    if p.at(NEWLINE) {
        p.bump_any();
    }

    m.complete(p, LINE);
}

// Instructions module
mod instructions {
    use super::*;

    /// Parse an instruction.
    pub(super) fn instruction(p: &mut Parser<'_>) {
        let m = p.start();

        // Parse the opcode
        if p.at_instruction_start() {
            p.bump_any();
        } else {
            let span = p.token_span();
            p.error(
                "Expected an instruction opcode".to_string(),
                "Opcodes must be valid identifiers".to_string(),
                span,
            );
        }

        // Consume whitespace after opcode
        while p.at(WHITESPACE) {
            p.bump_any();
        }

        // Parse operand if present
        if !p.at(NEWLINE) && !p.at(HASH) && !p.at(EOF) {
            // Check for unexpected opening bracket
            if p.at(LBRACKET) {
                unexpected_array_accessor(p);
            } else {
                operands::operand(p);
            }
        }

        m.complete(p, INSTRUCTION);
    }

    /// Handle unexpected array accessor that isn't attached to any operand
    fn unexpected_array_accessor(p: &mut Parser<'_>) {
        let open_bracket_span = p.token_span();
        p.bump_any(); // Consume the opening bracket

        // Check if there's a number or identifier inside the brackets
        if p.at(NUMBER) || p.at(IDENTIFIER) {
            p.bump_any(); // Consume the number or identifier

            // Check for closing bracket
            if p.at(RBRACKET) {
                let close_bracket_span = p.token_span();
                p.bump_any(); // Consume the closing bracket

                // Create a more descriptive error with both spans
                let spans = vec![
                    (open_bracket_span.clone(), "here".to_string()),
                    (
                        open_bracket_span.start..close_bracket_span.end,
                        "accessing nothing".to_string(),
                    ),
                ];

                p.labeled_error(
                    "Array accessor to nowhere".to_string(),
                    "Array accessors can only be used after an identifier or number".to_string(),
                    spans,
                );
            } else {
                // Missing closing bracket
                let spans = vec![
                    (open_bracket_span.clone(), "here".to_string()),
                    (open_bracket_span.clone(), "accessing nothing".to_string()),
                ];

                p.labeled_error(
                    "Unclosed array accessor to nowhere".to_string(),
                    "Array accessors can only be used after an identifier or number and must be closed with ']'".to_string(),
                    spans,
                );
            }
        } else {
            // No valid index inside brackets
            p.error(
                "Empty array accessor".to_string(),
                "Array accessors must contain a number or identifier".to_string(),
                open_bracket_span,
            );

            // Skip to closing bracket if present
            if p.at(RBRACKET) {
                p.bump_any(); // Consume the closing bracket
            }
        }
    }
}

// Operands module
mod operands {
    use super::*;

    /// Parse an operand.
    pub(super) fn operand(p: &mut Parser<'_>) {
        let m = p.start();

        // Check for addressing mode indicators
        match p.current() {
            STAR => {
                // Indirect addressing
                let m_inner = p.start();
                p.bump_any(); // Consume *
                operand_value(p);
                m_inner.complete(p, INDIRECT_OPERAND);
            }
            EQUALS => {
                // Immediate addressing
                let m_inner = p.start();
                p.bump_any(); // Consume =
                operand_value(p);
                m_inner.complete(p, IMMEDIATE_OPERAND);
            }
            _ => {
                // Direct addressing (default)
                let m_inner = p.start();
                operand_value(p);
                m_inner.complete(p, DIRECT_OPERAND);
            }
        }

        m.complete(p, OPERAND);
    }

    /// Parse an operand value (number or identifier, possibly with array accessor).
    fn operand_value(p: &mut Parser<'_>) {
        let m = p.start();

        // Parse the base value (number or identifier)
        if p.at(NUMBER) || p.at(IDENTIFIER) {
            p.bump_any();

            // Check for array accessor [index]
            if p.at(LBRACKET) {
                array_accessor(p);
            }
        } else {
            let span = p.token_span();
            p.error(
                "Expected a number or identifier".to_string(),
                "Operands must be numbers or identifiers".to_string(),
                span,
            );
        }

        m.complete(p, OPERAND_VALUE);
    }

    /// Parse an array accessor [index].
    fn array_accessor(p: &mut Parser<'_>) {
        let m = p.start();

        // Record the position of the opening bracket for error reporting
        let open_bracket_span = p.token_span();

        // Consume the opening bracket
        if p.at(LBRACKET) {
            p.bump_any();
        }

        // Parse the index (must be a number or identifier)
        if p.at(NUMBER) || p.at(IDENTIFIER) {
            p.bump_any();
        } else {
            let span = p.token_span();
            p.error(
                "Expected a number or identifier as array index".to_string(),
                "Array indices must be numbers or identifiers".to_string(),
                span,
            );
        }

        // Check for the closing bracket
        if p.at(RBRACKET) {
            p.bump_any();
        } else {
            // Report unclosed bracket error
            p.error(
                "Unclosed bracket in array accessor".to_string(),
                "Add a closing bracket ']' to complete the array accessor".to_string(),
                open_bracket_span,
            );
        }

        m.complete(p, ARRAY_ACCESSOR);
    }
}

// Labels module
mod labels {
    use super::*;

    /// Parse a label definition.
    pub(super) fn label_definition(p: &mut Parser<'_>) {
        let m = p.start();

        // Parse the label name
        if p.at(IDENTIFIER) {
            p.bump_any();
        } else {
            // This shouldn't happen due to the at_label_definition_start check
            let span = p.token_span();
            p.error(
                "Expected a label name".to_string(),
                "Label names must start with a letter".to_string(),
                span,
            );
        }

        // Consume whitespace between label name and colon
        while p.at(WHITESPACE) {
            p.bump_any();
        }

        // Parse the colon
        if p.at(COLON) {
            p.bump_any();
        } else {
            // This shouldn't happen due to the at_label_definition_start check
            let span = p.token_span();
            p.error(
                "Expected a colon after label name".to_string(),
                "Add a colon after the label name".to_string(),
                span,
            );
        }

        m.complete(p, LABEL_DEF);
    }
}

// Comments module
mod comments {
    use super::*;

    /// Parse a comment.
    pub(super) fn comment(p: &mut Parser<'_>) {
        let m = p.start();

        // Parse the hash symbol
        if p.at(HASH) {
            p.bump_any();
        } else {
            let span = p.token_span();
            p.error(
                "Expected a comment starting with #".to_string(),
                "Comments must start with #".to_string(),
                span,
            );
        }

        // Parse the comment text if present
        if p.at(COMMENT_TEXT) {
            p.bump_any();
        }

        m.complete(p, COMMENT);
    }
}
