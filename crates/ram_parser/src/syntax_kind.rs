use num_derive::{FromPrimitive, ToPrimitive};
#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, FromPrimitive, ToPrimitive)]
#[repr(u16)] // Rowan requires a primitive representation
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "SCREAMING_SNAKE_CASE"))]
#[allow(non_camel_case_types)]
pub enum SyntaxKind {
    // Nodes
    ROOT = 0, // Start explicit numbering if using FromPrimitive
    LINE,
    INSTRUCTION,
    LABEL_DEF,
    COMMENT,
    OPERAND,
    DIRECT_OPERAND,    // Direct addressing (e.g., 5)
    INDIRECT_OPERAND,  // Indirect addressing (e.g., *5)
    IMMEDIATE_OPERAND, // Immediate addressing (e.g., =5)
    OPERAND_VALUE,
    ARRAY_ACCESSOR, // Array accessor [index]

    // Error nodes
    ERROR,      // Error node used in parsing
    ERROR_NODE, // Legacy error node type

    // --- TOKEN KINDS ---
    // It's conventional in Rowan to include token kinds in the same enum
    // for a unified SyntaxKind type used by the tree.
    WHITESPACE = 100, // Start tokens at a higher offset
    NEWLINE,
    HASH,         // '#' itself (distinct from Comment node/token text)
    COMMENT_TEXT, // The text content of a comment token
    NUMBER,
    IDENTIFIER,
    LOAD_KW,
    STORE_KW,
    ADD_KW,
    SUB_KW,
    MUL_KW,
    DIV_KW,
    JUMP_KW,
    JGTZ_KW,
    JZERO_KW,
    HALT_KW,
    COLON,
    STAR,        // '*' for indirect addressing
    EQUALS,      // '=' for immediate addressing
    LBRACKET,    // '[' for array access
    RBRACKET,    // ']' for array access
    ERROR_TOKEN, // Token for unrecognized characters
    EOF,         // Not usually represented in the tree, but needed for parsing
}

// Implement conversion for Rowan
impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        Self(num_traits::ToPrimitive::to_u16(&kind).unwrap())
    }
}

// Implement conversion from rowan::SyntaxKind
impl From<rowan::SyntaxKind> for SyntaxKind {
    fn from(kind: rowan::SyntaxKind) -> Self {
        match num_traits::FromPrimitive::from_u16(kind.0) {
            Some(kind) => kind,
            None => SyntaxKind::ERROR_NODE,
        }
    }
}

impl SyntaxKind {
    #[inline]
    pub fn is_trivia(self) -> bool {
        matches!(self, SyntaxKind::WHITESPACE | SyntaxKind::NEWLINE | SyntaxKind::COMMENT)
    }

    /// Returns true if this is an identifier or a keyword.
    #[inline]
    pub fn is_any_identifier(self) -> bool {
        self == SyntaxKind::IDENTIFIER
    }

    /// Returns true if this is a keyword.
    #[inline]
    pub fn is_keyword(self) -> bool {
        matches!(
            self,
            SyntaxKind::LOAD_KW
                | SyntaxKind::STORE_KW
                | SyntaxKind::ADD_KW
                | SyntaxKind::SUB_KW
                | SyntaxKind::MUL_KW
                | SyntaxKind::DIV_KW
                | SyntaxKind::JUMP_KW
                | SyntaxKind::JGTZ_KW
                | SyntaxKind::JZERO_KW
                | SyntaxKind::HALT_KW
        )
    }
}
