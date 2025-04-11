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
    MOD_STMT,       // Module declaration statement
    USE_STMT,       // Module use statement
    MODULE_PATH,    // Path in a module statement

    // Error nodes
    ERROR,      // Error node used in parsing
    ERROR_NODE, // Legacy error node type

    // --- TOKEN KINDS ---
    // It's conventional in Rowan to include token kinds in the same enum
    // for a unified SyntaxKind type used by the tree.
    WHITESPACE,
    NEWLINE,
    #[static_text("#")]
    HASH, // '#' itself (distinct from Comment node/token text)
    #[static_text("#*")]
    HASH_STAR, // '#*' documentation comment marker
    COMMENT_TEXT,
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
    #[static_text("mod")]
    MOD_KW, // 'mod' keyword
    #[static_text("use")]
    USE_KW, // 'use' keyword
    #[static_text(":")]
    COLON,
    #[static_text("*")]
    STAR, // '*' for indirect addressing
    #[static_text("=")]
    EQUALS, // '=' for immediate addressing
    #[static_text("[")]
    LBRACKET, // '[' for array access
    #[static_text("]")]
    RBRACKET, // ']' for array access
    #[static_text("{")]
    LBRACE, // '{' for import specifiers
    #[static_text("}")]
    RBRACE, // '}' for import specifiers
    #[static_text(",")]
    COMMA, // ',' for separating import specifiers
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

    /// Returns true if this is a module-related keyword.
    #[inline]
    pub fn is_module_keyword(self) -> bool {
        matches!(self, SyntaxKind::MOD_KW | SyntaxKind::USE_KW)
    }
}
