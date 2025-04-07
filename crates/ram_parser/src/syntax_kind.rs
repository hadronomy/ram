use num_derive::{FromPrimitive, ToPrimitive};
#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, FromPrimitive, ToPrimitive)]
#[repr(u16)] // Rowan requires a primitive representation
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "SCREAMING_SNAKE_CASE"))]
pub enum SyntaxKind {
    // Nodes
    Root = 0, // Start explicit numbering if using FromPrimitive
    Line,
    Instruction,
    LabelDef,
    Comment,
    Operand,
    DirectOperand,    // Direct addressing (e.g., 5)
    IndirectOperand,  // Indirect addressing (e.g., *5)
    ImmediateOperand, // Immediate addressing (e.g., =5)
    OperandValue,
    ArrayAccessor, // Array accessor [index]

    // Error nodes
    Error,     // Error node used in parsing
    ErrorNode, // Legacy error node type

    // --- TOKEN KINDS ---
    // It's conventional in Rowan to include token kinds in the same enum
    // for a unified SyntaxKind type used by the tree.
    Whitespace = 100, // Start tokens at a higher offset
    Newline,
    Hash,        // '#' itself (distinct from Comment node/token text)
    CommentText, // The text content of a comment token
    Number,
    Identifier,
    LoadKw,
    StoreKw,
    AddKw,
    SubKw,
    MulKw,
    DivKw,
    JumpKw,
    JgtzKw,
    JzeroKw,
    HaltKw,
    Colon,
    Star,     // '*' for indirect addressing
    Equals,   // '=' for immediate addressing
    LBracket, // '[' for array access
    RBracket, // ']' for array access
    ErrorTok, // Token for unrecognized characters
    Eof,      // Not usually represented in the tree, but needed for parsing
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
            None => SyntaxKind::ErrorNode,
        }
    }
}

impl SyntaxKind {
    #[inline]
    pub fn is_trivia(self) -> bool {
        matches!(self, SyntaxKind::Whitespace | SyntaxKind::Newline | SyntaxKind::Comment)
    }

    /// Returns true if this is an identifier or a keyword.
    #[inline]
    pub fn is_any_identifier(self) -> bool {
        self == SyntaxKind::Identifier
    }

    /// Returns true if this is a keyword.
    #[inline]
    pub fn is_keyword(self) -> bool {
        matches!(
            self,
            SyntaxKind::LoadKw
                | SyntaxKind::StoreKw
                | SyntaxKind::AddKw
                | SyntaxKind::SubKw
                | SyntaxKind::MulKw
                | SyntaxKind::DivKw
                | SyntaxKind::JumpKw
                | SyntaxKind::JgtzKw
                | SyntaxKind::JzeroKw
                | SyntaxKind::HaltKw
        )
    }
}
