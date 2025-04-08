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

/// Entry point for the grammar
pub(crate) mod entry {
    use super::*;

    /// Top-level grammar productions
    pub(crate) mod top {
        use super::*;

        /// Program is the root of the AST
        /// A program consists of a sequence of statements
        pub(crate) fn program(p: &mut Parser<'_>) {
            let m = p.start();

            // Skip leading whitespace and newlines
            whitespace::skip_ws_and_nl(p);

            // Parse statements until end of file
            while !p.at(EOF) {
                stmt::statement(p);
                whitespace::skip_ws_and_nl(p);
            }

            m.complete(p, ROOT);
        }
    }
}

/// Whitespace handling module
mod whitespace {
    use super::*;

    /// Skip whitespace and newlines
    pub(super) fn skip_ws_and_nl(p: &mut Parser<'_>) {
        while p.at(WHITESPACE) || p.at(NEWLINE) {
            p.bump_any();
        }
    }

    /// Skip only whitespace (not newlines)
    pub(super) fn skip_ws(p: &mut Parser<'_>) {
        while p.at(WHITESPACE) {
            p.bump_any();
        }
    }
}

/// Statement module - handles top-level statements
mod stmt {
    use super::*;

    /// Parse a statement
    /// A statement can be a label definition, an instruction, or a comment group
    pub(super) fn statement(p: &mut Parser<'_>) {
        // Skip leading whitespace
        whitespace::skip_ws(p);

        // Skip empty lines and EOF
        if p.at(EOF) || p.at(NEWLINE) {
            // Nothing to do for empty lines or EOF
            if p.at(NEWLINE) {
                p.bump_any(); // Consume the newline
            }
            return;
        }

        if p.at(HASH) || p.at(HASH_STAR) {
            // Comment statement
            let m = p.start();
            comments::comment_group(p);
            m.complete(p, STMT);
        } else if p.at(IDENTIFIER) && p.at_label_definition_start() {
            // Label definition
            let m = p.start();
            labels::label_definition(p);

            // Skip whitespace after label
            whitespace::skip_ws(p);

            // Check for instruction after label
            if p.at_instruction_start() {
                expr::instruction_expr(p);
            }

            m.complete(p, STMT);
        } else if p.at_instruction_start() {
            // Instruction statement
            let m = p.start();
            expr::instruction_expr(p);
            m.complete(p, STMT);
        } else if p.at(RBRACKET) {
            // Unexpected closing bracket
            let m = p.start();
            p.err_and_bump(
                "Unexpected closing bracket ']'",
                "This closing bracket doesn't match any opening bracket",
            );
            m.complete(p, STMT);
        } else if p.at(ERROR_TOKEN) {
            // Error token
            let m = p.start();
            let text = p.token_text().to_string();
            p.err_and_bump(
                &format!("Unexpected character: {text}"),
                "Remove or replace this character",
            );
            m.complete(p, STMT);
        } else {
            // Unexpected token
            let m = p.start();
            let text = p.token_text().to_string();
            p.err_and_bump(
                &format!("Unexpected token: {text}"),
                "Expected an instruction, label, or comment",
            );
            m.complete(p, STMT);
        }

        // Skip trailing whitespace and newlines
        whitespace::skip_ws_and_nl(p);
    }
}

/// Expression module - handles expressions like instructions and operands
mod expr {
    use super::*;

    /// Parse an instruction expression
    pub(super) fn instruction_expr(p: &mut Parser<'_>) {
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

        // Skip whitespace after opcode
        whitespace::skip_ws(p);

        // Check for unexpected opening bracket
        if p.at(LBRACKET) {
            unexpected_array_accessor(p);
        }
        // Check for unexpected closing bracket
        else if p.at(RBRACKET) {
            p.err_and_bump(
                "Unexpected closing bracket ']'",
                "This closing bracket doesn't match any opening bracket",
            );
        }
        // Parse operand if present
        else if !p.at(NEWLINE) && !p.at(HASH) && !p.at(HASH_STAR) && !p.at(EOF) {
            operand_expr(p);
        }

        m.complete(p, INSTRUCTION);
    }

    /// Handle unexpected array accessor that isn't attached to any operand
    pub(super) fn unexpected_array_accessor(p: &mut Parser<'_>) {
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

    /// Parse an operand expression
    pub(super) fn operand_expr(p: &mut Parser<'_>) {
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

    /// Parse an operand value (number or identifier, possibly with array accessor)
    pub(super) fn operand_value(p: &mut Parser<'_>) {
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

    /// Parse an array accessor [index]
    pub(super) fn array_accessor(p: &mut Parser<'_>) {
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
        whitespace::skip_ws(p);

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

    /// Parse a single comment (either regular or documentation).
    pub(super) fn comment(p: &mut Parser<'_>) {
        let m = p.start();

        // Determine the type of comment
        let comment_kind = if p.at(HASH_STAR) {
            // Documentation comment
            p.bump_any(); // Consume the #* marker
            DOC_COMMENT
        } else if p.at(HASH) {
            // Regular comment
            p.bump_any(); // Consume the # marker
            COMMENT
        } else {
            // Error case
            let span = p.token_span();
            p.error(
                "Expected a comment starting with # or #*".to_string(),
                "Comments must start with # or #*".to_string(),
                span,
            );
            COMMENT // Default to regular comment in error case
        };

        // Parse the comment text if present
        if p.at(COMMENT_TEXT) {
            p.bump_any();
        }

        m.complete(p, comment_kind);
    }

    /// Parse a group of consecutive comments of the same type.
    /// This will create a COMMENT_GROUP node that contains one or more comments of the same type.
    /// Comments are grouped even across line breaks.
    /// Returns the type of comments that were parsed (true for doc comments, false for regular comments).
    pub(super) fn comment_group(p: &mut Parser<'_>) -> bool {
        // Start a comment group marker
        let group_marker = p.start();

        // Determine the type of the first comment
        let is_doc_comment = p.at(HASH_STAR);

        // Parse the first comment
        comment(p);

        // Keep track of the current position to detect if we're making progress
        let mut last_pos = p.current_pos();

        // Continue parsing comments as long as we see more comment markers of the same type
        // after optional whitespace and newlines
        loop {
            // Skip any whitespace
            whitespace::skip_ws(p);

            // If we see a newline, consume it and check for more comments
            if p.at(NEWLINE) {
                p.bump_any(); // Consume the newline

                // Skip whitespace at the beginning of the next line
                whitespace::skip_ws(p);

                // If the next line starts with a comment of the same type, parse it
                if (p.at(HASH_STAR) && is_doc_comment)
                    || (p.at(HASH) && !p.at(HASH_STAR) && !is_doc_comment)
                {
                    comment(p);

                    // Check if we're making progress
                    let current_pos = p.current_pos();
                    if current_pos <= last_pos {
                        // We're not making progress, break to avoid infinite loop
                        break;
                    }
                    last_pos = current_pos;
                } else {
                    // Not a comment of the same type, we're done with this group
                    break;
                }
            } else if p.at(HASH) || p.at(HASH_STAR) {
                // Another comment on the same line
                let current_is_doc = p.at(HASH_STAR);
                if current_is_doc != is_doc_comment {
                    // Different type of comment, we're done with this group
                    break;
                }

                // Parse the next comment
                comment(p);

                // Check if we're making progress
                let current_pos = p.current_pos();
                if current_pos <= last_pos {
                    // We're not making progress, break to avoid infinite loop
                    break;
                }
                last_pos = current_pos;
            } else {
                // Not a comment or newline, we're done with this group
                break;
            }
        }

        // Complete the comment group
        group_marker.complete(p, COMMENT_GROUP);

        // Return the type of comments that were parsed
        is_doc_comment
    }
}
