use cstree::Syntax;
#[cfg(feature = "serde")]
use serde::Serialize;

pub type Ram = SyntaxKind;
pub type SyntaxNode = cstree::syntax::SyntaxNode<Ram>;
pub type SyntaxToken = cstree::syntax::SyntaxToken<Ram>;
pub type SyntaxElement = cstree::syntax::SyntaxElement<Ram>;
pub type SyntaxText<'n, 'i, I> = cstree::text::SyntaxText<'n, 'i, I, Ram>;
pub type ResolvedNode = cstree::syntax::ResolvedNode<Ram>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Syntax)]
#[repr(u32)] // Rowan requires a primitive representation
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "SCREAMING_SNAKE_CASE"))]
#[allow(non_camel_case_types)]
pub enum SyntaxKind {
    // Nodes
    ROOT,
    STMT, // Statement node
    INSTRUCTION,
    LABEL_DEF,
    COMMENT,
    DOC_COMMENT,   // Documentation comment (#*)
    COMMENT_GROUP, // Group of consecutive comments
    OPERAND,
    DIRECT_OPERAND,    // Direct addressing (e.g., 5)
    INDIRECT_OPERAND,  // Indirect addressing (e.g., *5)
    IMMEDIATE_OPERAND, // Immediate addressing (e.g., =5)
    OPERAND_VALUE,
    ARRAY_ACCESSOR, // Array accessor [index]
    IMPORT_STMT,    // Import statement
    IMPORT_PATH,    // Path in an import statement

    // Error nodes
    ERROR,      // Error node used in parsing
    ERROR_NODE, // Legacy error node type

    // --- TOKEN KINDS ---
    // It's conventional in Rowan to include token kinds in the same enum
    // for a unified SyntaxKind type used by the tree.
    WHITESPACE,
    NEWLINE,
    HASH,         // '#' itself (distinct from Comment node/token text)
    HASH_STAR,    // '#*' documentation comment marker
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
    IMPORT_KW, // 'import' keyword
    FROM_KW,   // 'from' keyword
    COLON,
    STAR,        // '*' for indirect addressing
    EQUALS,      // '=' for immediate addressing
    LBRACKET,    // '[' for array access
    RBRACKET,    // ']' for array access
    LBRACE,      // '{' for import specifiers
    RBRACE,      // '}' for import specifiers
    COMMA,       // ',' for separating import specifiers
    STRING,      // String literal for import paths
    ERROR_TOKEN, // Token for unrecognized characters
    EOF,         // Not usually represented in the tree, but needed for parsing
}

impl SyntaxKind {
    #[inline]
    pub fn is_trivia(self) -> bool {
        matches!(
            self,
            SyntaxKind::WHITESPACE
                | SyntaxKind::NEWLINE
                | SyntaxKind::COMMENT
                | SyntaxKind::DOC_COMMENT
                | SyntaxKind::COMMENT_GROUP
        )
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
