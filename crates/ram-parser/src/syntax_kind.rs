use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, FromPrimitive, ToPrimitive)]
#[allow(non_camel_case_types)]
#[repr(u16)]
pub enum SyntaxKind {
    // Tokens
    IDENTIFIER = 0,
    NUMBER,
    COLON,
    EQUALS,
    ASTERISK,
    LBRACKET,
    RBRACKET,
    COMMENT,
    WHITESPACE,
    NEWLINE,

    // Keywords (if any, add here)

    // Nodes
    PROGRAM,
    LINE,
    LABEL_DEFINITION,
    INSTRUCTION,
    ARGUMENT,
    DIRECT,
    INDIRECT,
    IMMEDIATE,
    LABEL,
    COMMENT_NODE,
    ACCESSOR,
    INDEX,
    EOF,
    IDENT,
    // Error
    ERROR,
    __LAST,
}

impl From<u16> for SyntaxKind {
    #[inline]
    fn from(d: u16) -> SyntaxKind {
        SyntaxKind::from_u16(d).unwrap_or(SyntaxKind::ERROR)
    }
}

impl From<SyntaxKind> for u16 {
    #[inline]
    fn from(k: SyntaxKind) -> u16 {
        k as u16
    }
}

impl SyntaxKind {
    #[inline]
    pub fn is_trivia(self) -> bool {
        matches!(self, SyntaxKind::WHITESPACE | SyntaxKind::COMMENT)
    }

    /// Returns true if this is an identifier or a keyword.
    #[inline]
    pub fn is_any_identifier(self) -> bool {
        self == SyntaxKind::IDENTIFIER
    }
}
