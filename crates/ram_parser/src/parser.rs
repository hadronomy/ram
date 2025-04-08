//! Recursive descent parser for RAM assembly language.
//!
//! This module provides the parser infrastructure for the RAM assembly language.
//! The actual grammar is defined in the grammar.rs module.
//!
#![allow(dead_code)]
#![allow(clippy::enum_glob_use)]

use std::cell::Cell;
use std::ops::Range;

use drop_bomb::DropBomb;

use crate::SyntaxKind::*;
use crate::event::Event;
use crate::lexer::{Lexer, Token};
use crate::{SyntaxKind, grammar};

/// The maximum number of steps the parser will take before giving up.
const PARSER_STEP_LIMIT: usize = 100_000;

/// EOF (end of file) token is used to indicate that there are no more tokens.
const EOF: SyntaxKind = SyntaxKind::EOF;

/// Parse RAM assembly code into a sequence of events and errors.
///
/// The events can be used to build a syntax tree using the `build_tree` function.
pub fn parse(source: &str) -> (Vec<Event>, Vec<SyntaxError>) {
    // Tokenize the source text
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();

    // Create the input and parser
    let input = Input::new(tokens);
    let mut parser = Parser::new(&input);

    grammar::entry::top::program(&mut parser);

    // Return the events and errors
    parser.finish()
}

/// Convert internal ParseError to ram_error types.
///
/// This function converts our internal ParseError to the ram_error types
/// that can be used with miette for nice error reporting.
pub fn convert_errors(source: &str, errors: Vec<SyntaxError>) -> ram_error::ParserError {
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
    errors: Vec<SyntaxError>,
    /// The number of steps the parser has taken.
    steps: Cell<u32>,
}

impl<'t> Parser<'t> {
    /// Create a new parser for the given tokens.
    pub(crate) fn new(inp: &'t Input) -> Parser<'t> {
        Parser { inp, pos: 0, events: Vec::new(), errors: Vec::new(), steps: Cell::new(0) }
    }

    /// Extract the events produced by the parser.
    pub(crate) fn finish(self) -> (Vec<Event>, Vec<SyntaxError>) {
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
        self.errors.push(SyntaxError {
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
            return self.errors.push(SyntaxError { message, help, labeled_spans: spans });
        }
        // Fallback if no spans provided
        self.error(message, help, 0..0);
    }

    /// Starts a new node in the syntax tree. All nodes and tokens
    /// consumed between the `start` and the corresponding `Marker::complete`
    /// belong to the same node.
    pub(crate) fn start(&mut self) -> Marker {
        let pos = u32::try_from(self.events.len()).expect("Too many parser events");
        self.push_event(Event::Placeholder { kind_slot: ROOT });
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
        m.complete(self, ERROR);
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

        m.complete(self, ERROR);
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
    pub(crate) fn at_instruction_start(&self) -> bool {
        let kind = self.current();
        kind.is_keyword() || kind == IDENTIFIER
    }

    /// Returns true if the current token looks like the start of a label definition.
    pub(crate) fn at_label_definition_start(&self) -> bool {
        if self.at(IDENTIFIER) {
            // Look ahead for a colon, skipping whitespace
            let mut n = 1;
            loop {
                match self.nth(n) {
                    WHITESPACE => n += 1,
                    COLON => return true,
                    _ => return false,
                }
            }
        }
        false
    }

    /// Parse a program (the root of the AST).
    ///
    /// # Deprecated
    ///
    /// This function is deprecated and will be removed use
    /// [`crate::grammar::entry::top::program`] instead.
    ///
    /// Example:
    ///
    /// ```rust ignore
    /// use ram_parser::{Parser, SyntaxKind, grammar};
    ///
    /// let source = "LOAD 1 # Load value\nHALT\n";
    /// let mut parser = Parser::new(&source);
    /// grammar::entry::top::program(&mut parser);
    /// ```
    #[deprecated(note = "Use [`crate::grammar::entry::top::program`] instead")]
    pub(crate) fn parse_program(&mut self) {
        grammar::entry::top::program(self);
    }
}

/// A simple error type used during parsing.
/// This will be converted to ram_error::SingleParserError later.
#[derive(Debug, Clone)]
pub struct SyntaxError {
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
    pub(crate) fn new(tokens: Vec<Token>) -> Self {
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
