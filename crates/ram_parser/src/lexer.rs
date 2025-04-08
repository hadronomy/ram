//! Lexer for RAM assembly language.
//!
//! This module provides the lexer for tokenizing RAM assembly code.
#![allow(clippy::enum_glob_use)]

use std::ops::Range;

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::SyntaxKind;
use crate::SyntaxKind::*;

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

/// Lexer for RAM assembly language.
///
/// Converts a string into a sequence of tokens.
pub(crate) struct Lexer<'a> {
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
    pub(crate) fn new(source: &'a str) -> Self {
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
                kind: WHITESPACE,
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

        Token { kind: NEWLINE, text: "\n".to_string(), span: start..self.position }
    }

    /// Tokenize a comment (# followed by text until end of line).
    fn tokenize_comment(&mut self) -> (Token, Option<Token>) {
        let hash_start = self.position;
        self.advance(); // Consume '#'

        let hash_token =
            Token { kind: HASH, text: "#".to_string(), span: hash_start..self.position };

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
                kind: COMMENT_TEXT,
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
            kind: NUMBER,
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
            "LOAD" => LOAD_KW,
            "STORE" => STORE_KW,
            "ADD" => ADD_KW,
            "SUB" => SUB_KW,
            "MUL" => MUL_KW,
            "DIV" => DIV_KW,
            "JUMP" => JUMP_KW,
            "JGTZ" => JGTZ_KW,
            "JZERO" => JZERO_KW,
            "HALT" => HALT_KW,
            _ => IDENTIFIER,
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
            Some(':') => Some(self.tokenize_single_char(COLON)),
            Some('*') => Some(self.tokenize_single_char(STAR)),
            Some('=') => Some(self.tokenize_single_char(EQUALS)),
            Some('[') => Some(self.tokenize_single_char(LBRACKET)),
            Some(']') => Some(self.tokenize_single_char(RBRACKET)),

            // Numbers and identifiers
            Some(c) if c.is_ascii_digit() => Some(self.tokenize_number()),
            Some(c) if c.is_ascii_alphabetic() => Some(self.tokenize_identifier()),

            // Error handling
            Some(_) => {
                // Unrecognized character
                let start = self.position;
                self.advance();
                Some(Token {
                    kind: ERROR_TOKEN,
                    text: self.source[start..self.position].to_string(),
                    span: start..self.position,
                })
            }
            None => None,
        }
    }

    /// Tokenize the entire source text.
    pub(crate) fn tokenize(&mut self) -> Vec<Token> {
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
