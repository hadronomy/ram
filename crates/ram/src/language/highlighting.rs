use std::sync::OnceLock;

use miette::highlighters::SyntectHighlighter;
use syntect::highlighting::ThemeSet;
use syntect::parsing::{SyntaxDefinition, SyntaxSet, SyntaxSetBuilder};

/// Holds the `.sublime-syntax` definition for RAM assembly language.
pub static SUBLIME_SYNTAX: &str = include_str!("ram.sublime-syntax");

/// Stores the loaded syntax definition for RAM assembly language.
pub static SYNTAX_DEFINITION: OnceLock<SyntaxDefinition> = OnceLock::new();

pub fn syntax_definition() -> &'static SyntaxDefinition {
    SYNTAX_DEFINITION.get_or_init(|| {
        SyntaxDefinition::load_from_str(SUBLIME_SYNTAX, false, Some("source.ram")).unwrap()
    })
}

pub fn syntax_set() -> SyntaxSet {
    let mut set = SyntaxSetBuilder::new();
    set.add(syntax_definition().to_owned());
    set.build()
}

pub fn highlighter() -> SyntectHighlighter {
    let theme = &ThemeSet::load_defaults().themes["base16-mocha.dark"];
    SyntectHighlighter::new(syntax_set(), theme.clone(), false)
}
