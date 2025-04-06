use crate::SyntaxKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RamLang {}

impl rowan::Language for RamLang {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        SyntaxKind::from(raw)
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
    }
}

// Type aliases for convenience
pub type SyntaxNode = rowan::SyntaxNode<RamLang>;
pub type SyntaxToken = rowan::SyntaxToken<RamLang>;
pub type SyntaxElement = rowan::SyntaxElement<RamLang>;
pub type SyntaxNodeChildren = rowan::SyntaxNodeChildren<RamLang>;
pub type SyntaxElementChildren = rowan::SyntaxElementChildren<RamLang>;
