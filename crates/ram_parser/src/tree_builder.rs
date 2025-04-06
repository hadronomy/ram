use rowan::{GreenNode, GreenNodeBuilder, Language};

use crate::event::Event;
use crate::language::RamLang;

/// Builds a Rowan Green Tree from a list of events
pub fn build_tree(events: Vec<Event>) -> GreenNode {
    let mut builder = GreenNodeBuilder::new();

    for event in events {
        match event {
            Event::StartNode { kind } => {
                // Convert our SyntaxKind into rowan's raw kind
                builder.start_node(RamLang::kind_to_raw(kind));
            }
            Event::AddToken { kind, text, span: _ } => {
                // Convert our SyntaxKind into rowan's raw kind
                // Span info isn't stored in Green Tree
                builder.token(RamLang::kind_to_raw(kind), &text);
            }
            Event::FinishNode => {
                builder.finish_node();
            }
            Event::Error { message: _, pos: _ } => {
                // Errors reported by the parser are not directly part of the
                // Rowan Green Tree structure itself.
                // Errors should ideally be represented by the parser emitting
                // StartNode/FinishNode pairs with SyntaxKind::ErrorNode
                // around the problematic tokens/nodes *before* tree building.
                // We simply ignore Event::Error during tree construction.
            }
        }
    }

    builder.finish()
}
