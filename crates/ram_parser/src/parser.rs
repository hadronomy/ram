//! Recursive descent parser for RAM assembly language.
//!
//! This module provides a parser for the RAM assembly language, which
//! produces a sequence of events that can be used to build a syntax tree.

use std::ops::Range;

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::SyntaxKind;
use crate::event::Event;

/// A token produced by the lexer.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Token {
    /// The kind of token.
    pub kind: SyntaxKind,
    /// The text of the token.
    pub text: String,
    /// The span of the token in the source text.
    pub span: Range<usize>,
}

/// A simple error type used during parsing.
/// This will be converted to ram_error::SingleParserError later.
#[derive(Debug, Clone)]
pub struct ParseError {
    /// The error message.
    pub message: String,
    /// Additional help text.
    pub help: String,
    /// The labeled spans for this error.
    pub labeled_spans: Vec<(Range<usize>, String)>,
}

/// Lexer for RAM assembly language.
///
/// Converts a string into a sequence of tokens.
struct Lexer<'a> {
    /// The source text.
    source: &'a str,
    /// The current position in the source text.
    position: usize,
    /// The current line number (1-based).
    line: usize,
    /// The current column number (1-based).
    column: usize,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer for the given source text.
    fn new(source: &'a str) -> Self {
        Self { source, position: 0, line: 1, column: 1 }
    }

    /// Get the current character without advancing.
    fn peek(&self) -> Option<char> {
        self.source[self.position..].chars().next()
    }

    /// Advance to the next character.
    fn advance(&mut self) {
        if let Some(c) = self.peek() {
            self.position += c.len_utf8();
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
    }

    /// Skip whitespace characters.
    fn skip_whitespace(&mut self) -> Option<Token> {
        let start = self.position;
        let mut has_whitespace = false;

        while let Some(c) = self.peek() {
            if c.is_whitespace() && c != '\n' {
                has_whitespace = true;
                self.advance();
            } else {
                break;
            }
        }

        if has_whitespace {
            Some(Token {
                kind: SyntaxKind::Whitespace,
                text: self.source[start..self.position].to_string(),
                span: start..self.position,
            })
        } else {
            None
        }
    }

    /// Tokenize a newline character.
    fn tokenize_newline(&mut self) -> Token {
        let start = self.position;
        self.advance(); // Consume '\n'

        Token { kind: SyntaxKind::Newline, text: "\n".to_string(), span: start..self.position }
    }

    /// Tokenize a comment (# followed by text until end of line).
    fn tokenize_comment(&mut self) -> (Token, Option<Token>) {
        let hash_start = self.position;
        self.advance(); // Consume '#'

        let hash_token = Token {
            kind: SyntaxKind::Hash,
            text: "#".to_string(),
            span: hash_start..self.position,
        };

        let comment_start = self.position;
        while let Some(c) = self.peek() {
            if c == '\n' {
                break;
            }
            self.advance();
        }

        let comment_text = self.source[comment_start..self.position].to_string();
        let comment_token = if comment_text.is_empty() {
            None
        } else {
            Some(Token {
                kind: SyntaxKind::CommentText,
                text: comment_text,
                span: comment_start..self.position,
            })
        };

        (hash_token, comment_token)
    }

    /// Tokenize a number.
    fn tokenize_number(&mut self) -> Token {
        let start = self.position;

        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                self.advance();
            } else {
                break;
            }
        }

        Token {
            kind: SyntaxKind::Number,
            text: self.source[start..self.position].to_string(),
            span: start..self.position,
        }
    }

    /// Tokenize an identifier or keyword.
    fn tokenize_identifier(&mut self) -> Token {
        let start = self.position;

        // First character must be a letter
        if let Some(c) = self.peek() {
            if c.is_ascii_alphabetic() {
                self.advance();
            }
        }

        // Subsequent characters can be letters, digits, or underscores
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == '_' {
                self.advance();
            } else {
                break;
            }
        }

        let text = self.source[start..self.position].to_string();
        let kind = match text.to_uppercase().as_str() {
            "LOAD" => SyntaxKind::LoadKw,
            "STORE" => SyntaxKind::StoreKw,
            "ADD" => SyntaxKind::AddKw,
            "SUB" => SyntaxKind::SubKw,
            "MUL" => SyntaxKind::MulKw,
            "DIV" => SyntaxKind::DivKw,
            "JUMP" => SyntaxKind::JumpKw,
            "JGTZ" => SyntaxKind::JgtzKw,
            "JZERO" => SyntaxKind::JzeroKw,
            "HALT" => SyntaxKind::HaltKw,
            _ => SyntaxKind::Identifier,
        };

        Token { kind, text, span: start..self.position }
    }

    /// Tokenize a single character token.
    fn tokenize_single_char(&mut self, kind: SyntaxKind) -> Token {
        let start = self.position;
        let c = self.peek().unwrap();
        self.advance();

        Token { kind, text: c.to_string(), span: start..self.position }
    }

    /// Get the next token from the source text.
    fn next_token(&mut self) -> Option<Token> {
        if self.position >= self.source.len() {
            return None;
        }

        // Skip whitespace first
        if let Some(ws_token) = self.skip_whitespace() {
            return Some(ws_token);
        }

        // Check the current character
        match self.peek() {
            Some('\n') => Some(self.tokenize_newline()),
            Some('#') => {
                let (hash_token, _) = self.tokenize_comment();
                Some(hash_token)
            }
            Some(':') => Some(self.tokenize_single_char(SyntaxKind::Colon)),
            Some('*') => Some(self.tokenize_single_char(SyntaxKind::Star)),
            Some('=') => Some(self.tokenize_single_char(SyntaxKind::Equals)),
            Some('[') => Some(self.tokenize_single_char(SyntaxKind::LBracket)),
            Some(']') => Some(self.tokenize_single_char(SyntaxKind::RBracket)),
            Some(c) if c.is_ascii_digit() => Some(self.tokenize_number()),
            Some(c) if c.is_ascii_alphabetic() => Some(self.tokenize_identifier()),
            Some(_) => {
                // Unrecognized character
                let start = self.position;
                self.advance();
                Some(Token {
                    kind: SyntaxKind::ErrorTok,
                    text: self.source[start..self.position].to_string(),
                    span: start..self.position,
                })
            }
            None => None,
        }
    }

    /// Tokenize the entire source text.
    fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while self.position < self.source.len() {
            // Handle comments specially to include both the hash and the comment text
            if self.peek() == Some('#') {
                let (hash_token, comment_token) = self.tokenize_comment();
                tokens.push(hash_token);
                if let Some(token) = comment_token {
                    tokens.push(token);
                }
            } else if let Some(token) = self.next_token() {
                tokens.push(token);
            }
        }

        tokens
    }
}

/// Parser for RAM assembly language.
///
/// Converts a sequence of tokens into a sequence of events.
struct Parser {
    /// The tokens to parse.
    tokens: Vec<Token>,
    /// The current position in the token stream.
    position: usize,
    /// The events produced by the parser.
    events: Vec<Event>,
    /// The errors encountered during parsing.
    errors: Vec<ParseError>,
}

impl Parser {
    /// Create a new parser for the given tokens.
    pub(crate) fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, position: 0, events: Vec::new(), errors: Vec::new() }
    }

    /// Get the current token without advancing.
    pub(crate) fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    /// Get the token at the given offset from the current position.
    pub(crate) fn peek_ahead(&self, offset: usize) -> Option<&Token> {
        self.tokens.get(self.position + offset)
    }

    /// Advance to the next token.
    pub(crate) fn advance(&mut self) {
        self.position += 1;
    }

    /// Check if the current token has the given kind.
    pub(crate) fn at(&self, kind: SyntaxKind) -> bool {
        self.peek().is_some_and(|t| t.kind == kind)
    }

    /// Consume the current token if it has the given kind.
    pub(crate) fn eat(&mut self, kind: SyntaxKind) -> bool {
        if self.at(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Skip tokens until a token with one of the given kinds is found.
    pub(crate) fn skip_until(&mut self, kinds: &[SyntaxKind]) {
        while let Some(token) = self.peek() {
            if kinds.contains(&token.kind) {
                break;
            }
            self.advance();
        }
    }

    /// Add an event to the event stream.
    pub(crate) fn push_event(&mut self, event: Event) {
        self.events.push(event);
    }

    /// Add an error with a single labeled span.
    pub(crate) fn push_error(&mut self, message: String, help: String, span: Range<usize>) {
        self.errors.push(ParseError {
            message,
            help,
            labeled_spans: vec![(span, "here".to_string())],
        });
    }

    /// Add an error with multiple labeled spans.
    #[allow(clippy::if_not_else)]
    pub(crate) fn push_labeled_error(
        &mut self,
        message: String,
        help: String,
        spans: Vec<(Range<usize>, String)>,
    ) {
        if !spans.is_empty() {
            self.errors.push(ParseError { message, help, labeled_spans: spans });
        } else {
            // Fallback if no spans provided
            self.push_error(message, help, 0..0);
        }
    }

    /// Start a new node with the given kind.
    pub(crate) fn start_node(&mut self, kind: SyntaxKind) {
        self.push_event(Event::StartNode { kind });
    }

    /// Finish the current node.
    pub(crate) fn finish_node(&mut self) {
        self.push_event(Event::FinishNode);
    }

    /// Add a token to the event stream.
    pub(crate) fn token(&mut self) {
        let token = self.peek().unwrap().clone();
        self.push_event(Event::AddToken {
            kind: token.kind,
            text: token.text.clone(),
            span: token.span.clone(),
        });
        self.advance();
    }

    /// Parse a program (the root of the AST).
    fn parse_program(&mut self) {
        self.start_node(SyntaxKind::Root);

        while self.peek().is_some() {
            self.parse_line();
        }

        self.finish_node();
    }

    /// Parse a line, which can be an instruction, label definition, or comment.
    fn parse_line(&mut self) {
        self.start_node(SyntaxKind::Line);

        // Skip whitespace at the beginning of the line
        while self.at(SyntaxKind::Whitespace) {
            self.token();
        }

        // Check what kind of line this is
        if self.at(SyntaxKind::Hash) {
            // Comment line
            self.parse_comment();
        } else if self.is_label_definition() {
            // Label definition
            self.parse_label_definition();

            // After a label definition, check for an instruction on the same line
            // Consume whitespace after the label
            while self.at(SyntaxKind::Whitespace) {
                self.token();
            }

            // Check if there's an instruction after the label
            if self.is_instruction() {
                self.parse_instruction();
            }
        } else if self.is_instruction() {
            // Instruction
            self.parse_instruction();
        } else if self.at(SyntaxKind::Newline) {
            // Empty line
            self.token(); // Consume newline
        } else if self.at(SyntaxKind::LBracket) {
            // Unexpected opening bracket
            let token = self.peek().unwrap();
            self.push_error(
                "Unexpected opening bracket '['".to_string(),
                "Square brackets can only be used in array accessors after an identifier or number"
                    .to_string(),
                token.span.clone(),
            );
            self.token(); // Consume the bracket
        } else if self.at(SyntaxKind::RBracket) {
            // Extra closing bracket
            let token = self.peek().unwrap();
            self.push_error(
                "Unexpected closing bracket ']'".to_string(),
                "This closing bracket doesn't match any opening bracket".to_string(),
                token.span.clone(),
            );
            self.token(); // Consume the bracket
        } else if self.at(SyntaxKind::ErrorTok) {
            // Error token
            let token = self.peek().unwrap();
            self.push_error(
                format!("Unexpected character: {}", token.text),
                "Remove or replace this character".to_string(),
                token.span.clone(),
            );
            self.token(); // Consume error token
        } else {
            // Unexpected token
            if let Some(token) = self.peek() {
                self.push_error(
                    format!("Unexpected token: {}", token.text),
                    "Expected an instruction, label, or comment".to_string(),
                    token.span.clone(),
                );
                self.token(); // Consume unexpected token
            }
        }

        // Consume trailing whitespace
        while self.at(SyntaxKind::Whitespace) {
            self.token();
        }

        // Consume trailing comment if present
        if self.at(SyntaxKind::Hash) {
            self.parse_comment();
        }

        // Consume newline at the end of the line
        if self.at(SyntaxKind::Newline) {
            self.token();
        }

        self.finish_node();
    }

    /// Check if the current tokens form a label definition.
    fn is_label_definition(&self) -> bool {
        if let Some(token) = self.peek() {
            if token.kind == SyntaxKind::Identifier {
                // Check if the next non-whitespace token is a colon
                let mut offset = 1;
                while let Some(next) = self.peek_ahead(offset) {
                    if next.kind == SyntaxKind::Whitespace {
                        offset += 1;
                    } else {
                        return next.kind == SyntaxKind::Colon;
                    }
                }
            }
        }
        false
    }

    /// Check if the current tokens form an instruction.
    fn is_instruction(&self) -> bool {
        if let Some(token) = self.peek() {
            token.kind.is_keyword() || token.kind == SyntaxKind::Identifier
        } else {
            false
        }
    }

    /// Parse a label definition.
    fn parse_label_definition(&mut self) {
        self.start_node(SyntaxKind::LabelDef);

        // Parse the label name
        if self.at(SyntaxKind::Identifier) {
            self.token();
        } else {
            // This shouldn't happen due to the is_label_definition check
            let span = self.peek().map_or(0..0, |t| t.span.clone());
            self.push_error(
                "Expected a label name".to_string(),
                "Label names must start with a letter".to_string(),
                span,
            );
        }

        // Consume whitespace between label name and colon
        while self.at(SyntaxKind::Whitespace) {
            self.token();
        }

        // Parse the colon
        if self.at(SyntaxKind::Colon) {
            self.token();
        } else {
            // This shouldn't happen due to the is_label_definition check
            let span = self.peek().map_or(0..0, |t| t.span.clone());
            self.push_error(
                "Expected a colon after label name".to_string(),
                "Add a colon after the label name".to_string(),
                span,
            );
        }

        self.finish_node();
    }

    /// Parse an instruction.
    fn parse_instruction(&mut self) {
        self.start_node(SyntaxKind::Instruction);

        // Parse the opcode
        if self.peek().is_some_and(|t| t.kind.is_keyword() || t.kind == SyntaxKind::Identifier) {
            self.token();
        } else {
            let span = self.peek().map_or(0..0, |t| t.span.clone());
            self.push_error(
                "Expected an instruction opcode".to_string(),
                "Opcodes must be valid identifiers".to_string(),
                span,
            );
        }

        // Consume whitespace after opcode
        while self.at(SyntaxKind::Whitespace) {
            self.token();
        }

        // Parse operand if present
        if !self.at(SyntaxKind::Newline) && !self.at(SyntaxKind::Hash) && self.peek().is_some() {
            // Check for unexpected opening bracket
            if self.at(SyntaxKind::LBracket) {
                let open_bracket = self.peek().unwrap();
                let open_bracket_span = open_bracket.span.clone();
                self.token(); // Consume the opening bracket

                // Check if there's a number or identifier inside the brackets
                if self.at(SyntaxKind::Number) || self.at(SyntaxKind::Identifier) {
                    self.token(); // Consume the number or identifier

                    // Check for closing bracket
                    if self.at(SyntaxKind::RBracket) {
                        let close_bracket = self.peek().unwrap();
                        let close_bracket_span = close_bracket.span.clone();
                        self.token(); // Consume the closing bracket

                        // Create a more descriptive error with both spans
                        let spans = vec![
                            (open_bracket_span.clone(), "here".to_string()),
                            (
                                open_bracket_span.start..close_bracket_span.end,
                                "accessing nothing".to_string(),
                            ),
                        ];

                        self.push_labeled_error(
                            "Array accessor to nowhere".to_string(),
                            "Array accessors can only be used after an identifier or number"
                                .to_string(),
                            spans,
                        );
                    } else {
                        // Missing closing bracket
                        self.push_error(
                            "Unclosed array accessor".to_string(),
                            "Add a closing bracket ']' to complete the array accessor".to_string(),
                            open_bracket_span,
                        );
                    }
                } else {
                    // No valid index inside brackets
                    self.push_error(
                        "Empty array accessor".to_string(),
                        "Array accessors must contain a number or identifier".to_string(),
                        open_bracket_span,
                    );

                    // Skip to closing bracket if present
                    if self.at(SyntaxKind::RBracket) {
                        self.token(); // Consume the closing bracket
                    }
                }
            } else {
                self.parse_operand();
            }
        }

        self.finish_node();
    }

    /// Parse an operand.
    fn parse_operand(&mut self) {
        self.start_node(SyntaxKind::Operand);

        // Check for addressing mode indicators
        if self.at(SyntaxKind::Star) {
            // Indirect addressing
            self.start_node(SyntaxKind::IndirectOperand);
            self.token(); // Consume *
            self.parse_operand_value();
            self.finish_node(); // IndirectOperand
        } else if self.at(SyntaxKind::Equals) {
            // Immediate addressing
            self.start_node(SyntaxKind::ImmediateOperand);
            self.token(); // Consume =
            self.parse_operand_value();
            self.finish_node(); // ImmediateOperand
        } else {
            // Direct addressing
            self.start_node(SyntaxKind::DirectOperand);
            self.parse_operand_value();
            self.finish_node(); // DirectOperand
        }

        self.finish_node(); // Operand
    }

    /// Parse an operand value (number or identifier, possibly with array accessor).
    fn parse_operand_value(&mut self) {
        self.start_node(SyntaxKind::OperandValue);

        // Parse the base value (number or identifier)
        if self.at(SyntaxKind::Number) || self.at(SyntaxKind::Identifier) {
            self.token();

            // Check for array accessor [index]
            if self.at(SyntaxKind::LBracket) {
                self.parse_array_accessor();
            }
        } else {
            let span = self.peek().map_or(0..0, |t| t.span.clone());
            self.push_error(
                "Expected a number or identifier".to_string(),
                "Operands must be numbers or identifiers".to_string(),
                span,
            );
        }

        self.finish_node();
    }

    /// Parse an array accessor [index].
    fn parse_array_accessor(&mut self) {
        // Start a new node for the array accessor
        self.start_node(SyntaxKind::ArrayAccessor);

        // Record the position of the opening bracket for error reporting
        let open_bracket_pos = self.peek().map(|t| t.span.clone());

        // Consume the opening bracket
        if self.at(SyntaxKind::LBracket) {
            self.token();
        }

        // Parse the index (must be a number or identifier)
        if self.at(SyntaxKind::Number) || self.at(SyntaxKind::Identifier) {
            self.token();
        } else {
            let span = self.peek().map_or(0..0, |t| t.span.clone());
            self.push_error(
                "Expected a number or identifier as array index".to_string(),
                "Array indices must be numbers or identifiers".to_string(),
                span,
            );
        }

        // Check for the closing bracket
        if self.at(SyntaxKind::RBracket) {
            self.token();
        } else {
            // Report unclosed bracket error
            if let Some(span) = open_bracket_pos {
                self.push_error(
                    "Unclosed bracket in array accessor".to_string(),
                    "Add a closing bracket ']' to complete the array accessor".to_string(),
                    span,
                );
            }
        }

        self.finish_node();
    }

    /// Parse a comment.
    fn parse_comment(&mut self) {
        self.start_node(SyntaxKind::Comment);

        // Parse the hash symbol
        if self.at(SyntaxKind::Hash) {
            self.token();
        } else {
            let span = self.peek().map_or(0..0, |t| t.span.clone());
            self.push_error(
                "Expected a comment starting with #".to_string(),
                "Comments must start with #".to_string(),
                span,
            );
        }

        // Parse the comment text if present
        if self.at(SyntaxKind::CommentText) {
            self.token();
        }

        self.finish_node();
    }

    /// Parse the token stream and produce events.
    fn parse(mut self) -> (Vec<Event>, Vec<ParseError>) {
        self.parse_program();
        (self.events, self.errors)
    }
}

/// Parse RAM assembly code into a sequence of events and errors.
///
/// The events can be used to build a syntax tree using the `build_tree` function.
pub fn parse(source: &str) -> (Vec<Event>, Vec<ParseError>) {
    // Tokenize the source text
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();

    // Parse the tokens
    let parser = Parser::new(tokens);
    parser.parse()
}

/// Convert internal ParseError to ram_error types.
///
/// This function converts our internal ParseError to the ram_error types
/// that can be used with miette for nice error reporting.
pub fn convert_errors(source: &str, errors: Vec<ParseError>) -> ram_error::ParserError {
    use miette::LabeledSpan;
    use ram_error::{ParserError, SingleParserError};

    // Convert each ParseError to a SingleParserError
    let single_errors = errors
        .into_iter()
        .map(|e| {
            // Convert labeled spans to miette LabeledSpans
            let labels = e
                .labeled_spans
                .iter()
                .map(|(span, label)| {
                    LabeledSpan::new(Some(label.clone()), span.start, span.end - span.start)
                })
                .collect::<Vec<_>>();

            // Create a SingleParserError
            SingleParserError { message: e.message, labels }
        })
        .collect();

    // Create a ParserError with all the SingleParserErrors
    ParserError {
        src: miette::NamedSource::new("input.ram", source.to_string()),
        errors: single_errors,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer() {
        let source = "LOAD 1 # Load value\nHALT\n";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();

        assert_eq!(tokens.len(), 9);
        assert_eq!(tokens[0].kind, SyntaxKind::LoadKw);
        assert_eq!(tokens[1].kind, SyntaxKind::Whitespace);
        assert_eq!(tokens[2].kind, SyntaxKind::Number);
        assert_eq!(tokens[3].kind, SyntaxKind::Whitespace);
        assert_eq!(tokens[4].kind, SyntaxKind::Hash);
        assert_eq!(tokens[5].kind, SyntaxKind::CommentText);
        assert_eq!(tokens[6].kind, SyntaxKind::Newline);
        assert_eq!(tokens[7].kind, SyntaxKind::HaltKw);
        assert_eq!(tokens[8].kind, SyntaxKind::Newline);
    }

    #[test]
    fn test_parser() {
        let source = "LOAD 1 # Load value\nHALT\n";
        let (events, errors) = parse(source);

        assert!(errors.is_empty(), "Expected no errors, got: {errors:?}");
        assert!(!events.is_empty(), "Expected events, got none");
    }

    #[test]
    fn test_parse_label() {
        let source = "loop: LOAD 1\n";
        let (events, errors) = parse(source);

        assert!(errors.is_empty(), "Expected no errors, got: {errors:?}");
        assert!(!events.is_empty(), "Expected events, got none");
    }

    #[test]
    fn test_parse_indirect() {
        let source = "LOAD *1\n";
        let (events, errors) = parse(source);

        assert!(errors.is_empty(), "Expected no errors, got: {errors:?}");
        assert!(!events.is_empty(), "Expected events, got none");
    }

    #[test]
    fn test_parse_immediate() {
        let source = "LOAD =1\n";
        let (events, errors) = parse(source);

        assert!(errors.is_empty(), "Expected no errors, got: {errors:?}");
        assert!(!events.is_empty(), "Expected events, got none");
    }

    #[test]
    fn test_parse_error() {
        let source = "LOAD @1\n"; // @ is not a valid character
        let (_, errors) = parse(source);

        assert!(!errors.is_empty(), "Expected errors, got none");
    }

    #[test]
    fn test_unclosed_bracket() {
        let source = "LOAD x[5\n"; // Missing closing bracket
        let (_, errors) = parse(source);

        assert!(!errors.is_empty(), "Expected errors, got none");
        assert_eq!(errors[0].message, "Unclosed bracket in array accessor");
    }

    #[test]
    fn test_extra_closing_bracket() {
        let source = "LOAD 5]\n"; // Extra closing bracket
        let (_, errors) = parse(source);

        assert!(!errors.is_empty(), "Expected errors, got none");
        assert_eq!(errors[0].message, "Unexpected closing bracket ']'");
    }

    #[test]
    fn test_unexpected_opening_bracket() {
        let source = "LOAD [5]\n"; // Unexpected opening bracket
        let (_, errors) = parse(source);

        assert!(!errors.is_empty(), "Expected errors, got none");
        assert_eq!(errors[0].message, "Array accessor to nowhere");
    }

    #[test]
    fn test_valid_array_accessor() {
        let source = "LOAD x[5]\n"; // Valid array accessor
        let (_, errors) = parse(source);

        assert!(errors.is_empty(), "Expected no errors, got: {errors:?}");
    }

    #[test]
    fn test_label_with_newline() {
        let source = "label:\nLOAD 1\n"; // Label followed by newline
        let (events, errors) = parse(source);

        assert!(errors.is_empty(), "Expected no errors, got: {errors:?}");

        // Check that the newline token is present in the events
        let newline_found = events.iter().any(|e| {
            if let Event::AddToken { kind, .. } = e { *kind == SyntaxKind::Newline } else { false }
        });

        assert!(newline_found, "Newline token not found in events");
    }
}
