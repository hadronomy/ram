//! Recursive descent parser for RAM assembly language.
//!
//! This module provides a parser for the RAM assembly language, which
//! produces a sequence of events that can be used to build a syntax tree.
//!
#![allow(dead_code)]

use std::cell::Cell;
use std::ops::Range;

use drop_bomb::DropBomb;
#[cfg(feature = "serde")]
use serde::Serialize;

use crate::SyntaxKind;
use crate::event::Event;

/// The maximum number of steps the parser will take before giving up.
const PARSER_STEP_LIMIT: usize = 100_000;

/// EOF (end of file) token is used to indicate that there are no more tokens.
const EOF: SyntaxKind = SyntaxKind::Eof;

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

/// Input to the parser - a sequence of tokens.
#[derive(Debug)]
pub(crate) struct Input {
    /// The tokens in the input.
    tokens: Vec<Token>,
}

impl Input {
    /// Create a new input from a sequence of tokens.
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens }
    }

    /// Get the kind of token at the given position.
    fn kind(&self, pos: usize) -> SyntaxKind {
        self.tokens.get(pos).map_or(EOF, |t| t.kind)
    }

    /// Get a reference to the token at the given position.
    fn token(&self, pos: usize) -> Option<&Token> {
        self.tokens.get(pos)
    }
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
            // Special characters
            Some('\n') => Some(self.tokenize_newline()),
            Some('#') => {
                let (hash_token, _) = self.tokenize_comment();
                Some(hash_token)
            }

            // Single character tokens
            Some(':') => Some(self.tokenize_single_char(SyntaxKind::Colon)),
            Some('*') => Some(self.tokenize_single_char(SyntaxKind::Star)),
            Some('=') => Some(self.tokenize_single_char(SyntaxKind::Equals)),
            Some('[') => Some(self.tokenize_single_char(SyntaxKind::LBracket)),
            Some(']') => Some(self.tokenize_single_char(SyntaxKind::RBracket)),

            // Numbers and identifiers
            Some(c) if c.is_ascii_digit() => Some(self.tokenize_number()),
            Some(c) if c.is_ascii_alphabetic() => Some(self.tokenize_identifier()),

            // Error handling
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

// Macro for token set operations
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TokenSet(u128);

impl TokenSet {
    pub(crate) const EMPTY: TokenSet = TokenSet(0);

    pub(crate) const fn new(kinds: &[SyntaxKind]) -> TokenSet {
        let mut res = 0u128;
        let mut i = 0;
        while i < kinds.len() {
            let kind = kinds[i] as usize;
            if kind < 128 {
                res |= 1u128 << kind;
            }
            i += 1;
        }
        TokenSet(res)
    }

    pub(crate) const fn contains(&self, kind: SyntaxKind) -> bool {
        let kind = kind as usize;
        kind < 128 && (self.0 & (1u128 << kind)) != 0
    }
}

/// See [`Parser::start`].
///
/// A marker remembers the position of a syntax tree node that is in the process
/// of being parsed. It can be completed or abandoned.
#[must_use]
pub(crate) struct Marker {
    pos: u32,
    bomb: DropBomb,
}

impl Marker {
    /// Create a new marker at the given position.
    fn new(pos: u32) -> Self {
        Self { pos, bomb: DropBomb::new("Marker must be either completed or abandoned") }
    }

    /// Finishes the syntax tree node and assigns `kind` to it,
    /// and returns a `CompletedMarker` for possible future
    /// operation like `.precede()`.
    pub(crate) fn complete(mut self, p: &mut Parser<'_>, kind: SyntaxKind) -> CompletedMarker {
        self.bomb.defuse();
        let idx = self.pos as usize;
        match &mut p.events[idx] {
            Event::Placeholder { kind_slot } => {
                *kind_slot = kind;
            }
            _ => unreachable!(),
        }
        p.push_event(Event::FinishNode);
        CompletedMarker::new(self.pos, kind)
    }

    /// Abandons the syntax tree node. All its children
    /// are attached to its parent instead.
    pub(crate) fn abandon(mut self, p: &mut Parser<'_>) {
        self.bomb.defuse();
        let idx = self.pos as usize;
        if idx == p.events.len() - 1 {
            // If this is the last event, just pop it
            p.events.pop();
        } else {
            // Otherwise replace it with a Tombstone
            p.events[idx] = Event::Tombstone;
        }
    }
}

/// A completed marker that remembers the position and kind of a syntax tree node.
pub(crate) struct CompletedMarker {
    start_pos: u32,
    kind: SyntaxKind,
}

impl CompletedMarker {
    fn new(start_pos: u32, kind: SyntaxKind) -> Self {
        Self { start_pos, kind }
    }

    /// This method allows to create a new node which starts
    /// *before* the current one. That is, parser could start
    /// node `A`, then complete it, and then after parsing the
    /// whole `A`, decide that it should have started some node
    /// `B` before starting `A`. `precede` allows to do exactly
    /// that.
    pub(crate) fn precede(self, p: &mut Parser<'_>) -> Marker {
        let new_pos = p.start();
        let idx = self.start_pos as usize;
        match &mut p.events[idx] {
            Event::Placeholder { kind_slot: _ } => {
                p.events[idx] =
                    Event::StartNodeBefore { kind: self.kind, before_pos: new_pos.pos as usize };
            }
            _ => unreachable!(),
        }
        new_pos
    }

    /// Returns the kind of syntax tree node this marker represents.
    pub(crate) fn kind(&self) -> SyntaxKind {
        self.kind
    }
}

/// `Parser` struct provides the low-level API for
/// navigating through the stream of tokens and
/// constructing the parse tree.
pub(crate) struct Parser<'t> {
    /// The input tokens.
    inp: &'t Input,
    /// Current position in the token stream.
    pos: usize,
    /// The events produced by the parser.
    events: Vec<Event>,
    /// The errors encountered during parsing.
    errors: Vec<ParseError>,
    /// The number of steps the parser has taken.
    steps: Cell<u32>,
}

impl<'t> Parser<'t> {
    /// Create a new parser for the given tokens.
    pub(crate) fn new(inp: &'t Input) -> Parser<'t> {
        Parser { inp, pos: 0, events: Vec::new(), errors: Vec::new(), steps: Cell::new(0) }
    }

    /// Extract the events produced by the parser.
    pub(crate) fn finish(self) -> (Vec<Event>, Vec<ParseError>) {
        (self.events, self.errors)
    }

    /// Returns the kind of the current token.
    /// If parser has already reached the end of input,
    /// the special `EOF` kind is returned.
    pub(crate) fn current(&self) -> SyntaxKind {
        self.nth(0)
    }

    /// Lookahead operation: returns the kind of the next nth
    /// token.
    pub(crate) fn nth(&self, n: usize) -> SyntaxKind {
        let steps = self.steps.get();
        assert!((steps as usize) < PARSER_STEP_LIMIT, "the parser seems stuck");
        self.steps.set(steps + 1);

        self.inp.kind(self.pos + n)
    }

    /// Returns the text of the current token.
    pub(crate) fn token_text(&self) -> &str {
        self.inp.token(self.pos).map_or("", |t| &t.text)
    }

    /// Returns the span of the current token.
    pub(crate) fn token_span(&self) -> Range<usize> {
        self.inp.token(self.pos).map_or(0..0, |t| t.span.clone())
    }

    /// Checks if the current token is `kind`.
    pub(crate) fn at(&self, kind: SyntaxKind) -> bool {
        self.current() == kind
    }

    /// Checks if the nth token is `kind`.
    pub(crate) fn nth_at(&self, n: usize, kind: SyntaxKind) -> bool {
        self.nth(n) == kind
    }

    /// Checks if the current token is in `kinds`.
    pub(crate) fn at_ts(&self, kinds: TokenSet) -> bool {
        kinds.contains(self.current())
    }

    /// Consume the next token if `kind` matches.
    pub(crate) fn eat(&mut self, kind: SyntaxKind) -> bool {
        if !self.at(kind) {
            return false;
        }
        self.do_bump();
        true
    }

    /// Skip tokens until a token with one of the given kinds is found.
    pub(crate) fn skip_until(&mut self, kinds: TokenSet) {
        while !self.at(EOF) && !self.at_ts(kinds) {
            self.bump_any();
        }
    }

    /// Consume the next token if `kind` matches or emit an error
    /// otherwise.
    pub(crate) fn expect(&mut self, kind: SyntaxKind) -> bool {
        if self.eat(kind) {
            return true;
        }
        let token_text = self.token_text().to_string();
        let span = self.token_span();
        self.error(
            format!("Expected {kind:?}, got {token_text:?}"),
            format!("Try using {kind:?} here"),
            span,
        );
        false
    }

    /// Add an error with a single labeled span.
    pub(crate) fn error(&mut self, message: String, help: String, span: Range<usize>) {
        self.errors.push(ParseError {
            message,
            help,
            labeled_spans: vec![(span, "here".to_string())],
        });
    }

    /// Add an error with multiple labeled spans.
    pub(crate) fn labeled_error(
        &mut self,
        message: String,
        help: String,
        spans: Vec<(Range<usize>, String)>,
    ) {
        if !spans.is_empty() {
            return self.errors.push(ParseError { message, help, labeled_spans: spans });
        }
        // Fallback if no spans provided
        self.error(message, help, 0..0);
    }

    /// Starts a new node in the syntax tree. All nodes and tokens
    /// consumed between the `start` and the corresponding `Marker::complete`
    /// belong to the same node.
    pub(crate) fn start(&mut self) -> Marker {
        let pos = u32::try_from(self.events.len()).expect("Too many parser events");
        self.push_event(Event::Placeholder { kind_slot: SyntaxKind::Root });
        Marker::new(pos)
    }

    /// Advances the parser by one token
    pub(crate) fn bump_any(&mut self) {
        if self.at(EOF) {
            return;
        }
        self.do_bump();
    }

    /// Consume the next token. Panics if the parser isn't currently at `kind`.
    pub(crate) fn bump(&mut self, kind: SyntaxKind) {
        assert!(self.eat(kind));
    }

    /// Create an error node and consume the next token.
    pub(crate) fn err_and_bump(&mut self, message: &str, help: &str) {
        let m = self.start();
        let span = self.token_span();
        self.error(message.to_string(), help.to_string(), span);
        self.bump_any();
        m.complete(self, SyntaxKind::Error);
    }

    /// Create an error node and recover until a token in the recovery set.
    pub(crate) fn err_recover(&mut self, message: &str, help: &str, recovery: TokenSet) -> bool {
        if self.at_ts(recovery) {
            let span = self.token_span();
            self.error(message.to_string(), help.to_string(), span);
            return true;
        }

        let m = self.start();
        let span = self.token_span();
        self.error(message.to_string(), help.to_string(), span);

        // Consume tokens until we hit recovery point or EOF
        while !self.at(EOF) && !self.at_ts(recovery) {
            self.bump_any();
        }

        m.complete(self, SyntaxKind::Error);
        false
    }

    fn do_bump(&mut self) {
        // Get the current token and create an AddToken event
        if let Some(token) = self.inp.token(self.pos) {
            self.push_event(Event::AddToken {
                kind: token.kind,
                text: token.text.clone(),
                span: token.span.clone(),
            });
        }
        self.pos += 1;
        self.steps.set(0);
    }

    fn push_event(&mut self, event: Event) {
        self.events.push(event);
    }

    /// Returns true if the current token looks like the start of an instruction.
    fn at_instruction_start(&self) -> bool {
        let kind = self.current();
        kind.is_keyword() || kind == SyntaxKind::Identifier
    }

    /// Returns true if the current token looks like the start of a label definition.
    fn at_label_definition_start(&self) -> bool {
        if self.at(SyntaxKind::Identifier) {
            // Look ahead for a colon, skipping whitespace
            let mut n = 1;
            loop {
                match self.nth(n) {
                    SyntaxKind::Whitespace => n += 1,
                    SyntaxKind::Colon => return true,
                    _ => return false,
                }
            }
        }
        false
    }

    /// Parse a program (the root of the AST).
    pub(crate) fn parse_program(&mut self) {
        let m = self.start();

        while !self.at(EOF) {
            self.parse_line();
        }

        m.complete(self, SyntaxKind::Root);
    }

    /// Parse a line, which can be an instruction, label definition, or comment.
    fn parse_line(&mut self) {
        let m = self.start();

        // Skip whitespace at the beginning of the line
        while self.at(SyntaxKind::Whitespace) {
            self.bump_any();
        }

        // Check what kind of line this is
        match self.current() {
            EOF => {
                // End of input, nothing to do
                m.abandon(self);
                return;
            }
            SyntaxKind::Hash => {
                // Comment line
                self.parse_comment();
            }
            SyntaxKind::Newline => {
                // Empty line
                self.bump_any(); // Consume newline
            }
            SyntaxKind::LBracket => {
                // Unexpected opening bracket
                self.err_and_bump(
                    "Unexpected opening bracket '['",
                    "Square brackets can only be used in array accessors after an identifier or number"
                );
            }
            SyntaxKind::RBracket => {
                // Extra closing bracket
                self.err_and_bump(
                    "Unexpected closing bracket ']'",
                    "This closing bracket doesn't match any opening bracket",
                );
            }
            SyntaxKind::ErrorTok => {
                // Error token
                let text = self.token_text().to_string();
                self.err_and_bump(
                    &format!("Unexpected character: {text}"),
                    "Remove or replace this character",
                );
            }
            SyntaxKind::Identifier => {
                // Could be a label definition or an instruction
                if self.at_label_definition_start() {
                    // Label definition
                    self.parse_label_definition();

                    // After a label definition, check for an instruction on the same line
                    // Consume whitespace after the label
                    while self.at(SyntaxKind::Whitespace) {
                        self.bump_any();
                    }

                    // Check if there's an instruction after the label
                    if self.at_instruction_start() {
                        self.parse_instruction();
                    }
                } else if self.at_instruction_start() {
                    // Instruction
                    self.parse_instruction();
                } else {
                    // Unexpected identifier
                    let text = self.token_text().to_string();
                    self.err_and_bump(
                        &format!("Unexpected identifier: {text}"),
                        "Expected an instruction, label, or comment",
                    );
                }
            }
            _ if self.at_instruction_start() => {
                // Instruction with keyword
                self.parse_instruction();
            }
            _ => {
                // Unexpected token
                let text = self.token_text().to_string();
                self.err_and_bump(
                    &format!("Unexpected token: {text}"),
                    "Expected an instruction, label, or comment",
                );
            }
        }

        // Consume trailing whitespace
        while self.at(SyntaxKind::Whitespace) {
            self.bump_any();
        }

        // Consume trailing comment if present
        if self.at(SyntaxKind::Hash) {
            self.parse_comment();
        }

        // Consume newline at the end of the line
        if self.at(SyntaxKind::Newline) {
            self.bump_any();
        }

        m.complete(self, SyntaxKind::Line);
    }

    /// Parse a label definition.
    fn parse_label_definition(&mut self) {
        let m = self.start();

        // Parse the label name
        if self.at(SyntaxKind::Identifier) {
            self.bump_any();
        } else {
            // This shouldn't happen due to the at_label_definition_start check
            let span = self.token_span();
            self.error(
                "Expected a label name".to_string(),
                "Label names must start with a letter".to_string(),
                span,
            );
        }

        // Consume whitespace between label name and colon
        while self.at(SyntaxKind::Whitespace) {
            self.bump_any();
        }

        // Parse the colon
        if self.at(SyntaxKind::Colon) {
            self.bump_any();
        } else {
            // This shouldn't happen due to the at_label_definition_start check
            let span = self.token_span();
            self.error(
                "Expected a colon after label name".to_string(),
                "Add a colon after the label name".to_string(),
                span,
            );
        }

        m.complete(self, SyntaxKind::LabelDef);
    }

    /// Parse an instruction.
    fn parse_instruction(&mut self) {
        let m = self.start();

        // Parse the opcode
        if self.at_instruction_start() {
            self.bump_any();
        } else {
            let span = self.token_span();
            self.error(
                "Expected an instruction opcode".to_string(),
                "Opcodes must be valid identifiers".to_string(),
                span,
            );
        }

        // Consume whitespace after opcode
        while self.at(SyntaxKind::Whitespace) {
            self.bump_any();
        }

        // Parse operand if present
        if !self.at(SyntaxKind::Newline) && !self.at(SyntaxKind::Hash) && !self.at(EOF) {
            // Check for unexpected opening bracket
            if self.at(SyntaxKind::LBracket) {
                let open_bracket_span = self.token_span();
                self.bump_any(); // Consume the opening bracket

                // Check if there's a number or identifier inside the brackets
                if self.at(SyntaxKind::Number) || self.at(SyntaxKind::Identifier) {
                    self.bump_any(); // Consume the number or identifier

                    // Check for closing bracket
                    if self.at(SyntaxKind::RBracket) {
                        let close_bracket_span = self.token_span();
                        self.bump_any(); // Consume the closing bracket

                        // Create a more descriptive error with both spans
                        let spans = vec![
                            (open_bracket_span.clone(), "here".to_string()),
                            (
                                open_bracket_span.start..close_bracket_span.end,
                                "accessing nothing".to_string(),
                            ),
                        ];

                        self.labeled_error(
                            "Array accessor to nowhere".to_string(),
                            "Array accessors can only be used after an identifier or number"
                                .to_string(),
                            spans,
                        );
                    } else {
                        // Missing closing bracket
                        let spans = vec![
                            (open_bracket_span.clone(), "here".to_string()),
                            (
                                open_bracket_span.clone(),
                                "accessing nothing".to_string(),
                            ),
                        ];

                        self.labeled_error(
                            "Unclosed array accessor to nowhere".to_string(),
                            "Array accessors can only be used after an identifier or number and must be closed with ']'".to_string(),
                            spans,
                        );
                    }
                } else {
                    // No valid index inside brackets
                    self.error(
                        "Empty array accessor".to_string(),
                        "Array accessors must contain a number or identifier".to_string(),
                        open_bracket_span,
                    );

                    // Skip to closing bracket if present
                    if self.at(SyntaxKind::RBracket) {
                        self.bump_any(); // Consume the closing bracket
                    }
                }
            } else {
                self.parse_operand();
            }
        }

        m.complete(self, SyntaxKind::Instruction);
    }

    /// Parse an operand.
    fn parse_operand(&mut self) {
        let m = self.start();

        // Check for addressing mode indicators
        match self.current() {
            SyntaxKind::Star => {
                // Indirect addressing
                let m_inner = self.start();
                self.bump_any(); // Consume *
                self.parse_operand_value();
                m_inner.complete(self, SyntaxKind::IndirectOperand);
            }
            SyntaxKind::Equals => {
                // Immediate addressing
                let m_inner = self.start();
                self.bump_any(); // Consume =
                self.parse_operand_value();
                m_inner.complete(self, SyntaxKind::ImmediateOperand);
            }
            _ => {
                // Direct addressing (default)
                let m_inner = self.start();
                self.parse_operand_value();
                m_inner.complete(self, SyntaxKind::DirectOperand);
            }
        }

        m.complete(self, SyntaxKind::Operand);
    }

    /// Parse an operand value (number or identifier, possibly with array accessor).
    fn parse_operand_value(&mut self) {
        let m = self.start();

        // Parse the base value (number or identifier)
        if self.at(SyntaxKind::Number) || self.at(SyntaxKind::Identifier) {
            self.bump_any();

            // Check for array accessor [index]
            if self.at(SyntaxKind::LBracket) {
                self.parse_array_accessor();
            }
        } else {
            let span = self.token_span();
            self.error(
                "Expected a number or identifier".to_string(),
                "Operands must be numbers or identifiers".to_string(),
                span,
            );
        }

        m.complete(self, SyntaxKind::OperandValue);
    }

    /// Parse an array accessor [index].
    fn parse_array_accessor(&mut self) {
        let m = self.start();

        // Record the position of the opening bracket for error reporting
        let open_bracket_span = self.token_span();

        // Consume the opening bracket
        if self.at(SyntaxKind::LBracket) {
            self.bump_any();
        }

        // Parse the index (must be a number or identifier)
        if self.at(SyntaxKind::Number) || self.at(SyntaxKind::Identifier) {
            self.bump_any();
        } else {
            let span = self.token_span();
            self.error(
                "Expected a number or identifier as array index".to_string(),
                "Array indices must be numbers or identifiers".to_string(),
                span,
            );
        }

        // Check for the closing bracket
        if self.at(SyntaxKind::RBracket) {
            self.bump_any();
        } else {
            // Report unclosed bracket error
            self.error(
                "Unclosed bracket in array accessor".to_string(),
                "Add a closing bracket ']' to complete the array accessor".to_string(),
                open_bracket_span,
            );
        }

        m.complete(self, SyntaxKind::ArrayAccessor);
    }

    /// Parse a comment.
    fn parse_comment(&mut self) {
        let m = self.start();

        // Parse the hash symbol
        if self.at(SyntaxKind::Hash) {
            self.bump_any();
        } else {
            let span = self.token_span();
            self.error(
                "Expected a comment starting with #".to_string(),
                "Comments must start with #".to_string(),
                span,
            );
        }

        // Parse the comment text if present
        if self.at(SyntaxKind::CommentText) {
            self.bump_any();
        }

        m.complete(self, SyntaxKind::Comment);
    }
}

/// Parse RAM assembly code into a sequence of events and errors.
///
/// The events can be used to build a syntax tree using the `build_tree` function.
pub fn parse(source: &str) -> (Vec<Event>, Vec<ParseError>) {
    // Tokenize the source text
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();

    // Create the input and parser
    let input = Input::new(tokens);
    let mut parser = Parser::new(&input);

    // Parse the program
    parser.parse_program();

    // Return the events and errors
    parser.finish()
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

    /// Helper function to parse a string and return the events
    fn parse_test(source: &str) -> (Vec<Event>, Vec<ParseError>) {
        parse(source)
    }

    /// Helper function to check if parsing succeeded without errors
    fn assert_no_errors(errors: &[ParseError]) {
        assert!(errors.is_empty(), "Expected no errors, got: {errors:?}");
    }

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
    fn test_basic_parser() {
        let source = "LOAD 1 # Load value\nHALT\n";
        let (events, errors) = parse_test(source);

        assert_no_errors(&errors);
        assert!(!events.is_empty(), "Expected events, got none");

        // Verify we have the expected structure
        let has_root = events.iter().any(
            |e| matches!(e, Event::Placeholder { kind_slot } if *kind_slot == SyntaxKind::Root),
        );
        assert!(has_root, "Missing Root node in events");
    }

    #[test]
    fn test_complex_program() {
        let source = "# Simple test program
start:  LOAD =0    # Initialize accumulator 
        STORE x    # Initialize x
loop:   LOAD x     # Load x
        SUB =10    # Check if x >= 10
        JGTZ end   # If so, end
        LOAD x     # Load x again
        ADD =1     # Increment
        STORE x    # Store back
        JUMP loop  # Repeat
end:    HALT       # Stop execution
x:      LOAD =0    # Data";

        let (events, errors) = parse_test(source);
        assert_no_errors(&errors);

        // Count how many instructions we have
        let instruction_count = events.iter().filter(|e| {
            matches!(e, Event::Placeholder { kind_slot } if *kind_slot == SyntaxKind::Instruction)
        }).count();

        assert_eq!(instruction_count, 11, "Expected 11 instructions");
    }

    #[test]
    fn test_parse_label() {
        let source = "loop: LOAD 1\n";
        let (events, errors) = parse_test(source);

        assert_no_errors(&errors);

        // Check for LabelDef node
        let has_label = events.iter().any(
            |e| matches!(e, Event::Placeholder { kind_slot } if *kind_slot == SyntaxKind::LabelDef),
        );
        assert!(has_label, "Missing LabelDef node in events");
    }

    #[test]
    fn test_parse_indirect() {
        let source = "LOAD *1\n";
        let (events, errors) = parse_test(source);

        assert_no_errors(&errors);

        // Check for IndirectOperand node
        let has_indirect = events.iter().any(|e| matches!(e, Event::Placeholder { kind_slot } if *kind_slot == SyntaxKind::IndirectOperand));
        assert!(has_indirect, "Missing IndirectOperand node in events");
    }

    #[test]
    fn test_parse_immediate() {
        let source = "LOAD =1\n";
        let (events, errors) = parse_test(source);

        assert_no_errors(&errors);

        // Check for ImmediateOperand node
        let has_immediate = events.iter().any(|e| matches!(e, Event::Placeholder { kind_slot } if *kind_slot == SyntaxKind::ImmediateOperand));
        assert!(has_immediate, "Missing ImmediateOperand node in events");
    }

    #[test]
    fn test_parse_error() {
        let source = "LOAD @1\n"; // @ is not a valid character
        let (_, errors) = parse_test(source);

        assert!(!errors.is_empty(), "Expected errors, got none");
        // The current parser produces an "Expected a number or identifier" error when it encounters @ in an operand
        assert!(
            errors[0].message.contains("Expected a number or identifier"),
            "Expected error about missing operand value"
        );
    }

    #[test]
    fn test_unclosed_bracket() {
        let source = "LOAD x[5\n"; // Missing closing bracket
        let (_, errors) = parse_test(source);

        assert!(!errors.is_empty(), "Expected errors, got none");
        assert!(
            errors[0].message.contains("Unclosed bracket"),
            "Expected error about unclosed bracket"
        );
    }

    #[test]
    fn test_extra_closing_bracket() {
        let source = "LOAD 5]\n"; // Extra closing bracket
        let (_, errors) = parse_test(source);

        assert!(!errors.is_empty(), "Expected errors, got none");
        assert!(
            errors[0].message.contains("Unexpected closing bracket"),
            "Expected error about unexpected closing bracket"
        );
    }

    #[test]
    fn test_unexpected_opening_bracket() {
        let source = "LOAD [5]\n"; // Unexpected opening bracket
        let (_, errors) = parse_test(source);

        assert!(!errors.is_empty(), "Expected errors, got none");
        assert!(
            errors[0].message.contains("Array accessor to nowhere"),
            "Expected error about array accessor to nowhere"
        );
    }

    #[test]
    fn test_valid_array_accessor() {
        let source = "LOAD x[5]\n"; // Valid array accessor
        let (_, errors) = parse_test(source);

        assert_no_errors(&errors);
    }

    #[test]
    fn test_label_with_newline() {
        let source = "label:\nLOAD 1\n"; // Label followed by newline
        let (events, errors) = parse_test(source);

        assert_no_errors(&errors);

        // Check that we have at least two lines
        let line_count = events
            .iter()
            .filter(
                |e| matches!(e, Event::Placeholder { kind_slot } if *kind_slot == SyntaxKind::Line),
            )
            .count();

        assert!(line_count >= 2, "Expected at least 2 lines");
    }

    #[test]
    fn test_comment_only() {
        let source = "# Just a comment\n";
        let (events, errors) = parse_test(source);

        assert_no_errors(&errors);

        // Check for Comment node
        let has_comment = events.iter().any(
            |e| matches!(e, Event::Placeholder { kind_slot } if *kind_slot == SyntaxKind::Comment),
        );
        assert!(has_comment, "Missing Comment node in events");
    }

    #[test]
    fn test_empty_file() {
        let source = "";
        let (events, errors) = parse_test(source);

        assert_no_errors(&errors);
        assert!(!events.is_empty(), "Should have at least a Root node");
    }

    #[test]
    fn test_marker_handling() {
        // This test verifies that our marker system works properly
        let input = Input::new(vec![
            Token { kind: SyntaxKind::LoadKw, text: "LOAD".to_string(), span: 0..4 },
            Token { kind: SyntaxKind::Whitespace, text: " ".to_string(), span: 4..5 },
            Token { kind: SyntaxKind::Number, text: "42".to_string(), span: 5..7 },
        ]);

        let mut parser = Parser::new(&input);

        // Start outer node
        let outer = parser.start();
        parser.bump_any(); // LOAD
        parser.bump_any(); // whitespace

        // Start inner node
        let inner = parser.start();
        parser.bump_any(); // 42
        inner.complete(&mut parser, SyntaxKind::OperandValue);

        // Complete outer node
        outer.complete(&mut parser, SyntaxKind::Instruction);

        // Verify events
        let (events, errors) = parser.finish();
        assert_no_errors(&errors);

        // Check event sequence
        assert!(
            matches!(events[0], Event::Placeholder { kind_slot } if kind_slot == SyntaxKind::Instruction)
        );
        assert!(
            matches!(events[3], Event::Placeholder { kind_slot } if kind_slot == SyntaxKind::OperandValue)
        );
    }

    #[test]
    fn test_precede_marker() {
        // This tests the marker.precede() functionality
        let input = Input::new(vec![
            Token { kind: SyntaxKind::LoadKw, text: "LOAD".to_string(), span: 0..4 },
            Token { kind: SyntaxKind::Whitespace, text: " ".to_string(), span: 4..5 },
            Token { kind: SyntaxKind::Number, text: "42".to_string(), span: 5..7 },
        ]);

        let mut parser = Parser::new(&input);

        // Start value node first
        let value = parser.start();
        parser.bump_any(); // LOAD
        parser.bump_any(); // whitespace
        parser.bump_any(); // 42
        let completed = value.complete(&mut parser, SyntaxKind::OperandValue);

        // Now precede it with an instruction node
        let instruction = completed.precede(&mut parser);
        instruction.complete(&mut parser, SyntaxKind::Instruction);

        // Check events
        let (events, errors) = parser.finish();
        assert_no_errors(&errors);

        // Verify we have a StartNodeBefore event
        let has_start_before = events.iter().any(|e| matches!(e, Event::StartNodeBefore { .. }));
        assert!(has_start_before, "Missing StartNodeBefore event");
    }
}
