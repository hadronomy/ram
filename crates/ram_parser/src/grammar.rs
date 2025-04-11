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
use crate::diagnostic::{Diagnostic, DiagnosticKind};
use crate::parser::{Parser, TokenSet};

/// Entry point for the grammar
pub(crate) mod entry {
    use super::*;

    /// Top-level grammar productions
    pub(crate) mod top {
        use super::*;

        /// Parses a complete RAM program.
        ///
        /// # Structure
        /// A program consists of a sequence of statements until EOF.
        ///
        /// # Returns
        /// Completes a [`ROOT`] syntax node
        ///
        /// # Diagram
        /// ```text
        /// ┌─────────── ROOT ───────────┐
        /// │                            │
        /// │  ┌─────── STMT ────────┐   │
        /// │  │ ...                 │   │
        /// │  └─────────────────────┘   │
        /// │                            │
        /// │  ┌─────── STMT ────────┐   │
        /// │  │ ...                 │   │
        /// │  └─────────────────────┘   │
        /// │                            │
        /// │           ...              │
        /// │                            │
        /// └────────────────────────────┘
        /// ```
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
    ///
    /// # Behavior
    /// Consumes all consecutive [`WHITESPACE`] and [`NEWLINE`] tokens.
    ///
    /// # Diagram
    /// ```text
    /// ┌────┬────┬────┬─────────┐
    /// │ WS │ WS │ NL │ ...     │ => All consumed
    /// └────┴────┴────┴─────────┘
    /// ```
    pub(super) fn skip_ws_and_nl(p: &mut Parser<'_>) {
        while p.at(WHITESPACE) || p.at(NEWLINE) {
            p.bump_any();
        }
    }

    /// Skip only whitespace (not newlines)
    ///
    /// # Behavior
    /// Consumes all consecutive [`WHITESPACE`] tokens but preserves newlines.
    ///
    /// # Diagram
    /// ```text
    /// ┌────┬────┬────┬────┬─────────┐
    /// │ WS │ WS │ NL │ WS │ ...     │
    /// └────┴────┴────┴────┴─────────┘
    ///   ^    ^         ^
    ///   |____|         |
    ///   consumed    preserved
    /// ```
    pub(super) fn skip_ws(p: &mut Parser<'_>) {
        while p.at(WHITESPACE) {
            p.bump_any();
        }
    }
}

/// Statement module - handles top-level statements
mod stmt {
    use super::*;

    // Recovery token set for error handling
    const RECOVERY_SET: TokenSet = TokenSet::new(&[
        NEWLINE, HASH, HASH_STAR, IDENTIFIER, LOAD_KW, STORE_KW, ADD_KW, SUB_KW, MUL_KW, DIV_KW,
        JUMP_KW, JGTZ_KW, JZERO_KW, HALT_KW, MOD_KW, USE_KW,
    ]);

    /// Parses a statement.
    ///
    /// # Structure
    /// A statement can be one of:
    /// - A label definition (must be followed by an instruction, either on the same line or a subsequent line)
    /// - An instruction
    /// - A comment group
    ///
    /// # Diagram
    /// ```text
    /// ┌─────────── STMT ───────────┐
    /// │                            │
    /// │  ┌─── LABEL_DEF ────────┐  │
    /// │  │ identifier:          │  │
    /// │  └──────────────────────┘  │
    /// │                            │
    /// │  ┌─── INSTRUCTION ───────┐ │ ← Required after label (can be on next line)
    /// │  │ ...                   │ │
    /// │  └───────────────────────┘ │
    /// │                            │
    /// └────────────────────────────┘
    ///
    /// ┌─────────── STMT ───────────┐
    /// │                            │
    /// │  ┌─── INSTRUCTION ───────┐ │
    /// │  │ ...                   │ │
    /// │  └───────────────────────┘ │
    /// │                            │
    /// └────────────────────────────┘
    ///
    /// ┌─────────── STMT ───────────┐
    /// │                            │
    /// │  ┌─── COMMENT_GROUP ─────┐ │
    /// │  │ ...                   │ │
    /// │  └───────────────────────┘ │
    /// │                            │
    /// └────────────────────────────┘
    /// ```
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

        // Match on the current token to determine statement type
        match p.current() {
            MOD_KW => parse_module_declaration(p),
            USE_KW => parse_module_use(p),
            HASH | HASH_STAR => parse_comment_statement(p),
            IDENTIFIER if p.at_label_definition_start() => parse_label_statement(p),
            _ if p.at_instruction_start() => parse_instruction_statement(p),
            _ => handle_unexpected_token_in_statement(p),
        }

        // Skip trailing whitespace and newlines
        whitespace::skip_ws_and_nl(p);
    }

    // Helper function to parse module declaration statements
    fn parse_module_declaration(p: &mut Parser<'_>) {
        let m = p.start();
        modules::mod_stmt(p);
        m.complete(p, STMT);
    }

    // Helper function to parse module use statements
    fn parse_module_use(p: &mut Parser<'_>) {
        let m = p.start();
        modules::use_stmt(p);
        m.complete(p, STMT);
    }

    // Helper function to parse comment statements
    fn parse_comment_statement(p: &mut Parser<'_>) {
        let m = p.start();
        comments::comment_group(p);
        m.complete(p, STMT);
    }

    // Helper function to parse label statements
    fn parse_label_statement(p: &mut Parser<'_>) {
        let m = p.start();

        // Store the span of the label for error reporting
        let label_span = p.token_span().clone();

        labels::label_definition(p);

        // Skip whitespace after label
        whitespace::skip_ws(p);

        if p.at_instruction_start() {
            // Instruction on same line
            expr::instruction_expr(p);
        } else if p.at(NEWLINE) {
            // Check for instruction on next line
            p.bump_any(); // Consume the newline
            whitespace::skip_ws(p);

            if p.at_instruction_start() {
                expr::instruction_expr(p);
            } else {
                handle_missing_instruction_after_label(p, label_span);
            }
        } else if !p.at(EOF) {
            handle_missing_instruction_after_label(p, label_span);
        }

        m.complete(p, STMT);
    }

    // Helper function to parse instruction statements
    fn parse_instruction_statement(p: &mut Parser<'_>) {
        let m = p.start();
        expr::instruction_expr(p);
        m.complete(p, STMT);
    }

    /// Handles situations where a label is defined but not followed by an instruction.
    ///
    /// # Arguments
    /// * `p` - The parser
    /// * `label_span` - The span of the label definition for error reporting
    ///
    /// # Behavior
    /// Looks ahead for any subsequent instruction and provides a helpful error message,
    /// suggesting to place the label directly above an instruction.
    fn handle_missing_instruction_after_label(
        p: &mut Parser<'_>,
        label_span: std::ops::Range<usize>,
    ) {
        if let Some((_, instr_span)) =
            p.look_ahead_for(|kind| kind.is_keyword() || kind == IDENTIFIER)
        {
            // Found an instruction later in the file
            p.add_diagnostic(
                Diagnostic::builder()
                    .with_message("Label must be followed by an instruction")
                    .with_help("Place the label definition directly above an instruction")
                    .with_primary_span(label_span, "label defined here")
                    .with_secondary_span(
                        instr_span,
                        "instruction found here - place the label directly above this",
                    )
                    .with_code("E001")
                    .with_note("Labels must be followed by an instruction, either on the same line or the next line.")
                    .with_note("Consider moving this label directly above the instruction."),
                DiagnosticKind::Error
            );
        } else {
            // No instruction found later in the file
            p.add_diagnostic(
                Diagnostic::builder()
                    .with_message("Label must be followed by an instruction")
                    .with_help("Add an instruction after the label definition")
                    .with_primary_span(label_span, "label defined here")
                    .with_code("E001")
                    .with_note("Labels must be followed by an instruction, either on the same line or the next line.")
                    .with_note("Add an instruction like 'LOAD', 'STORE', 'ADD', etc. after this label."),
                DiagnosticKind::Error
            );
        }
    }

    /// Handles unexpected tokens encountered while parsing statements.
    ///
    /// # Arguments
    /// * `p` - The parser
    ///
    /// # Behavior
    /// Generates appropriate error messages based on the token type and
    /// creates a statement node to allow parsing to continue.
    ///
    /// # Diagram
    /// ```text
    /// ┌─────────── STMT ───────────┐
    /// │                            │
    /// │  <unexpected token>        │ ← Error reported here
    /// │                            │
    /// └────────────────────────────┘
    /// ```
    fn handle_unexpected_token_in_statement(p: &mut Parser<'_>) {
        let m = p.start();

        let (message, help) = match p.current() {
            RBRACKET => (
                "Unexpected closing bracket ']'".to_string(),
                "This closing bracket doesn't match any opening bracket",
            ),
            ERROR_TOKEN => {
                let text = p.token_text();
                (format!("Unexpected character: {text}"), "Remove or replace this character")
            }
            _ => {
                let text = p.token_text();
                (format!("Unexpected token: {text}"), "Expected an instruction, label, or comment")
            }
        };

        let span = p.token_span().clone();
        let builder = Diagnostic::builder()
            .with_message(message)
            .with_help(help)
            .with_primary_span(span, "here")
            .with_code("E002");

        // Use a warning for unexpected closing brackets, error for other cases
        let kind =
            if p.current() == RBRACKET { DiagnosticKind::Warning } else { DiagnosticKind::Error };

        p.add_diagnostic(builder, kind);
        p.bump_any(); // Consume the unexpected token

        // Try to recover by skipping to a token we know how to handle
        p.skip_until(RECOVERY_SET);

        m.complete(p, STMT);
    }
}

/// Module system - handles module declarations and imports
mod modules {
    use super::*;

    // Constants for error recovery
    const MODULE_PATH_RECOVERY: TokenSet = TokenSet::new(&[NEWLINE, EOF, HASH, HASH_STAR]);

    /// Parse a module declaration statement.
    ///
    /// # Syntax
    /// ```text
    /// mod mymodule
    /// ```
    pub(super) fn mod_stmt(p: &mut Parser<'_>) -> bool {
        if !p.at(MOD_KW) {
            return false;
        }

        let m = p.start();
        p.bump_any(); // Consume 'mod'
        whitespace::skip_ws(p);

        // Parse module name (identifier)
        if p.at(IDENTIFIER) {
            p.bump_any(); // Consume the module name
        } else {
            p.diagnostic_and_bump(
                "Expected module name",
                "Module declarations must be followed by a valid identifier",
                DiagnosticKind::Error,
            );
        }

        m.complete(p, MOD_STMT);
        true
    }

    /// Parse a module use statement.
    ///
    /// # Syntax
    /// ```text
    /// use mymodule::*
    /// use mymodule::symbol
    /// ```
    pub(super) fn use_stmt(p: &mut Parser<'_>) -> bool {
        if !p.at(USE_KW) {
            return false;
        }

        let m = p.start();
        p.bump_any(); // Consume 'use'
        whitespace::skip_ws(p);

        // Parse module path
        parse_module_path(p);

        m.complete(p, USE_STMT);
        true
    }

    /// Parse a module path.
    ///
    /// # Syntax
    /// ```text
    /// mymodule::*
    /// mymodule::symbol
    /// ```
    fn parse_module_path(p: &mut Parser<'_>) {
        let m = p.start();

        // Parse the module name (identifier)
        if p.at(IDENTIFIER) {
            p.bump_any(); // Consume the module name
            whitespace::skip_ws(p);

            // Check for double colon
            if p.at(COLON) && p.nth_at(1, COLON) {
                p.bump_any(); // Consume first colon
                p.bump_any(); // Consume second colon
                whitespace::skip_ws(p);

                // Parse what comes after the double colon
                if p.at(STAR) {
                    // Import everything from the module
                    p.bump_any(); // Consume '*'
                } else if p.at(IDENTIFIER) {
                    // Import a specific symbol
                    p.bump_any(); // Consume the symbol name
                } else {
                    p.error(
                        "Expected '*' or identifier after '::'",
                        "Use '::*' to import everything or '::symbol' to import a specific symbol",
                        p.token_span(),
                    );
                }
            } else {
                p.error(
                    "Expected '::' after module name",
                    "Use '::*' to import everything or '::symbol' to import a specific symbol",
                    p.token_span(),
                );
            }
        } else {
            p.error(
                "Expected module name",
                "Use statements must specify a valid module name",
                p.token_span(),
            );
            p.skip_until(MODULE_PATH_RECOVERY);
        }

        m.complete(p, MODULE_PATH);
    }
}

/// Expression module - handles expressions like instructions and operands
mod expr {
    use super::*;

    /// Parses an instruction expression.
    ///
    /// # Structure
    /// An instruction consists of an opcode followed by an optional operand.
    ///
    /// # Returns
    /// Completes an [`INSTRUCTION`] syntax node
    ///
    /// # Diagram
    /// ```text
    /// ┌───────── INSTRUCTION ───────────┐
    /// │                                 │
    /// │  ┌─── Opcode ─────┐             │
    /// │  │ identifier     │             │
    /// │  └────────────────┘             │
    /// │                                 │
    /// │  ┌─── OPERAND ────┐             │
    /// │  │ ...            │  ← Optional │
    /// │  └────────────────┘             │
    /// │                                 │
    /// └─────────────────────────────────┘
    /// ```
    pub(super) fn instruction_expr(p: &mut Parser<'_>) {
        let m = p.start();

        // Parse the opcode
        if p.at_instruction_start() {
            p.bump_any();
        } else {
            let span = p.token_span();
            p.error(
                "Expected an instruction opcode",
                "Opcodes must be valid identifiers",
                span,
            );
        }

        // Skip whitespace after opcode
        whitespace::skip_ws(p);

        // Check for operand or special cases
        match p.current() {
            LBRACKET => unexpected_array_accessor(p),
            RBRACKET => p.err_and_bump(
                "Unexpected closing bracket ']'",
                "This closing bracket doesn't match any opening bracket",
            ),
            NEWLINE | HASH | HASH_STAR | EOF => {} // No operand, which is fine
            _ => operand_expr(p),                  // Parse operand
        }

        m.complete(p, INSTRUCTION);
    }

    /// Handle unexpected array accessor that isn't attached to any operand.
    ///
    /// # Behavior
    /// Reports an appropriate error when an array accessor appears in an invalid position.
    ///
    /// # Diagram
    /// ```text
    /// ┌──────────────────────────┐
    /// │ LOAD [5]                 │ ← Error: Array accessor to nowhere
    /// └──────────────────────────┘
    ///       ^^^^^
    ///       |||||
    ///       Error: should be after an identifier/number
    /// ```
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
                    "Array accessor to nowhere",
                    "Array accessors can only be used after an identifier or number",
                    spans,
                );
            } else {
                // Missing closing bracket
                let spans = vec![
                    (open_bracket_span.clone(), "here".to_string()),
                    (open_bracket_span.clone(), "accessing nothing".to_string()),
                ];

                p.labeled_error(
                    "Unclosed array accessor to nowhere",
                    "Array accessors can only be used after an identifier or number and must be closed with ']'",
                    spans,
                );
            }
        } else {
            // No valid index inside brackets
            p.error(
                "Empty array accessor",
                "Array accessors must contain a number or identifier",
                open_bracket_span,
            );

            // Skip to closing bracket if present
            if p.at(RBRACKET) {
                p.bump_any(); // Consume the closing bracket
            }
        }
    }

    /// Parses an operand expression.
    ///
    /// # Structure
    /// An operand consists of an optional addressing mode indicator followed by a value.
    ///
    /// # Returns
    /// Completes an [`OPERAND`] syntax node with the appropriate addressing mode.
    ///
    /// # Diagram
    /// ```text
    /// ┌────────── OPERAND ────────────┐
    /// │                               │
    /// │  ┌─── DIRECT_OPERAND ──────┐  │
    /// │  │ value                   │  │
    /// │  └─────────────────────────┘  │
    /// │                               │
    /// └───────────────────────────────┘
    ///
    /// ┌────────── OPERAND ────────────┐
    /// │                               │
    /// │  ┌─── IMMEDIATE_OPERAND ────┐ │
    /// │  │ =value                   │ │
    /// │  └──────────────────────────┘ │
    /// │                               │
    /// └───────────────────────────────┘
    ///
    /// ┌────────── OPERAND ────────────┐
    /// │                               │
    /// │  ┌─── INDIRECT_OPERAND ─────┐ │
    /// │  │ *value                   │ │
    /// │  └──────────────────────────┘ │
    /// │                               │
    /// └───────────────────────────────┘
    /// ```
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

    /// Parses an operand value.
    ///
    /// # Structure
    /// An operand value is a number or identifier, optionally followed by an array accessor.
    ///
    /// # Returns
    /// Completes an [`OPERAND_VALUE`] syntax node.
    ///
    /// # Diagram
    /// ```text
    /// ┌──────── OPERAND_VALUE ────────┐
    /// │                               │
    /// │  identifier                   │
    /// │                               │
    /// └───────────────────────────────┘
    ///
    /// ┌──────── OPERAND_VALUE ────────┐
    /// │                               │
    /// │  number                       │
    /// │                               │
    /// └───────────────────────────────┘
    ///
    /// ┌──────── OPERAND_VALUE ────────┐
    /// │                               │
    /// │  identifier                   │
    /// │                               │
    /// │  ┌─── ARRAY_ACCESSOR ──────┐  │
    /// │  │ [index]                 │  │
    /// │  └─────────────────────────┘  │
    /// │                               │
    /// └───────────────────────────────┘
    /// ```
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
                "Expected a number or identifier",
                "Operands must be numbers or identifiers",
                span,
            );
        }

        m.complete(p, OPERAND_VALUE);
    }

    /// Parses an array accessor.
    ///
    /// # Structure
    /// An array accessor is a pair of square brackets containing an index expression.
    ///
    /// # Returns
    /// Completes an [`ARRAY_ACCESSOR`] syntax node.
    ///
    /// # Diagram
    /// ```text
    /// ┌────── ARRAY_ACCESSOR ─────────┐
    /// │                               │
    /// │  [ index ]                    │
    /// │    ^                          │
    /// │    └── number or identifier   │
    /// │                               │
    /// └───────────────────────────────┘
    /// ```
    pub(super) fn array_accessor(p: &mut Parser<'_>) {
        let m = p.start();

        // Record the position of the opening bracket for error reporting
        let open_bracket_span = p.token_span();

        // Consume the opening bracket
        p.bump_any(); // Consume '['

        // Parse the index (must be a number or identifier)
        if p.at(NUMBER) || p.at(IDENTIFIER) {
            p.bump_any();
        } else {
            p.error(
                "Expected a number or identifier as array index",
                "Array indices must be numbers or identifiers",
                p.token_span(),
            );
        }

        // Check for the closing bracket
        if p.at(RBRACKET) {
            p.bump_any();
        } else {
            // Report unclosed bracket error
            p.error(
                "Unclosed bracket in array accessor",
                "Add a closing bracket ']' to complete the array accessor",
                open_bracket_span,
            );
        }

        m.complete(p, ARRAY_ACCESSOR);
    }
}

// Labels module
mod labels {
    use super::*;

    /// Parses a label definition.
    ///
    /// # Structure
    /// A label definition consists of an identifier followed by a colon.
    /// A label definition must be followed by an instruction, either on the same line
    /// or on a subsequent line.
    ///
    /// # Returns
    /// Completes a [`LABEL_DEF`] syntax node.
    ///
    /// # Diagram
    /// ```text
    /// ┌─────────── LABEL_DEF ────────────┐
    /// │                                  │
    /// │  identifier:                     │
    /// │  ^         ^                     │
    /// │  |         |                     │
    /// │  label     colon                 │
    /// │                                  │
    /// └──────────────────────────────────┘
    /// ```
    ///
    /// # Note
    /// A label definition must be followed by an instruction. This can be on the same line
    /// or on the next line. For example:
    ///
    /// ```text
    /// label: LOAD 1    # Valid: instruction on same line
    ///
    /// label:           # Valid: instruction on next line
    ///     LOAD 1
    ///
    /// label:           # Invalid: no instruction follows the label
    /// ```
    pub(super) fn label_definition(p: &mut Parser<'_>) {
        let m = p.start();

        // Parse the label name
        if p.at(IDENTIFIER) {
            p.bump_any();
        } else {
            // This shouldn't happen due to the at_label_definition_start check
            let span = p.token_span();
            p.error(
                "Expected a label name",
                "Label names must start with a letter",
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
                "Expected a colon after label name",
                "Add a colon after the label name",
                span,
            );
        }

        m.complete(p, LABEL_DEF);
    }
}

/// Comments handling module for RAM assembly language comments
mod comments {
    use super::*;

    /// Parses a single comment (either regular or documentation).
    ///
    /// # Structure
    /// This function handles two types of comments:
    /// - Regular comments starting with `#`
    /// - Documentation comments starting with `#*`
    ///
    /// # Returns
    /// Completes a [`COMMENT`] or [`DOC_COMMENT`] syntax node
    ///
    /// # Diagram
    /// ```text
    /// # Regular comment:
    /// ┌────┬───────────────┐
    /// │ #  │ Comment text  │
    /// └────┴───────────────┘
    ///   ^         ^
    ///   |         └── Optional comment text
    ///   └── Comment marker
    ///
    /// # Documentation comment:
    /// ┌─────┬───────────────┐
    /// │ #*  │ Comment text  │
    /// └─────┴───────────────┘
    ///   ^          ^
    ///   |          └── Optional comment text
    ///   └── Doc comment marker
    /// ```
    pub(super) fn comment(p: &mut Parser<'_>) {
        let m = p.start();

        let comment_kind = if p.at(HASH_STAR) {
            p.bump_any();
            DOC_COMMENT
        } else if p.at(HASH) {
            p.bump_any();
            COMMENT
        } else {
            let span = p.token_span();
            p.error(
                "Expected a comment starting with # or #*",
                "Comments must start with # or #*",
                span,
            );
            COMMENT
        };

        if p.at(COMMENT_TEXT) {
            p.bump_any();
        }

        m.complete(p, comment_kind);
    }

    /// Parses a group of consecutive comments of the same type.
    ///
    /// # Overview
    /// Creates a [`COMMENT_GROUP`] node containing one or more comments
    /// of the same type, grouped even across line breaks.
    ///
    /// # Returns
    /// `bool` - Whether the parsed comments were documentation comments (`true`)
    /// or regular comments (`false`).
    ///
    /// # Structure
    ///
    /// ```text
    /// ┌─────────────── COMMENT_GROUP ───────────────────┐
    /// │                                                 │
    /// │  ┌─ COMMENT ─┐  ┌─ COMMENT ─┐                   │
    /// │  │ # Text 1  │  │ # Text 2  │  ← Same line      │
    /// │  └───────────┘  └───────────┘                   │
    /// │                                                 │
    /// │  ┌─ COMMENT ─┐                                  │
    /// │  │ # Text 3  │  ← After line break              │
    /// │  └───────────┘                                  │
    /// │                                                 │
    /// └─────────────────────────────────────────────────┘
    /// ```
    ///
    /// # Algorithm
    ///
    /// ```text
    /// ┌────────────────────┐
    /// │ Start Comment Group│
    /// └──────────┬─────────┘
    ///            │
    ///            ▼
    /// ┌──────────────────────┐
    /// │Parse 1st Comment     │
    /// │Remember Comment Type │
    /// └──────────┬───────────┘
    ///            │
    ///            ▼
    /// ┌──────────────────────┐
    /// │Skip Whitespace/      │◄─────┐
    /// │Newlines              │      │
    /// └──────────┬───────────┘      │
    ///            │                  │
    ///            ▼                  │
    /// ┌──────────────────────┐      │
    /// │More Comments of      │ Yes  │
    /// │Same Type?            ├──────┘
    /// └──────────┬───────────┘
    ///            │ No
    ///            ▼
    /// ┌──────────────────────┐
    /// │Complete Comment Group│
    /// └──────────────────────┘
    /// ```
    ///
    /// See [`comment`] for details on how individual comments are parsed.
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
        loop {
            // Skip any whitespace
            whitespace::skip_ws(p);

            let continue_group = match p.current() {
                // If we see a newline, check for comments on the next line
                NEWLINE => {
                    p.bump_any(); // Consume the newline
                    whitespace::skip_ws(p);

                    // Check if next line has a comment of the same type
                    let has_matching_comment = (p.at(HASH_STAR) && is_doc_comment)
                        || (p.at(HASH) && !p.at(HASH_STAR) && !is_doc_comment);

                    if has_matching_comment {
                        comment(p);
                    }
                    has_matching_comment
                }
                // Another comment on the same line
                HASH | HASH_STAR => {
                    let current_is_doc = p.at(HASH_STAR);
                    if current_is_doc == is_doc_comment {
                        comment(p);
                        true
                    } else {
                        // Different type of comment, we're done with this group
                        false
                    }
                }
                // Not a comment or newline, we're done with this group
                _ => false,
            };

            if !continue_group {
                break;
            }

            // Check if we're making progress
            let current_pos = p.current_pos();
            if current_pos <= last_pos {
                // We're not making progress, break to avoid infinite loop
                break;
            }
            last_pos = current_pos;
        }

        // Complete the comment group
        group_marker.complete(p, COMMENT_GROUP);

        // Return the type of comments that were parsed
        is_doc_comment
    }
}
