use rowan::{GreenNode, GreenNodeBuilder, Language};

use crate::SyntaxKind;
use crate::event::Event;
use crate::language::RamLang;

/// Builds a Rowan Green Tree from a list of events
pub fn build_tree(events: Vec<Event>) -> GreenNode {
    // Make sure we have at least one node to work with
    if events.is_empty() {
        // Create a minimal valid tree with just a root node
        let mut builder = GreenNodeBuilder::new();
        builder.start_node(RamLang::kind_to_raw(SyntaxKind::ROOT));
        builder.finish_node();
        return builder.finish();
    }

    // Remove any Tombstones and convert Placeholders to proper StartNode events
    let mut cleaned_events = Vec::with_capacity(events.len());

    for event in events {
        match event {
            Event::Tombstone => {
                // Skip tombstones
                continue;
            }
            Event::Placeholder { kind_slot } => {
                // Convert placeholder to proper StartNode
                cleaned_events.push(Event::StartNode { kind: kind_slot });
            }
            Event::StartNodeBefore { .. } => {
                // StartNodeBefore needs special handling, which we'll do later
                cleaned_events.push(event);
            }
            _ => {
                // Keep all other events as-is
                cleaned_events.push(event);
            }
        }
    }

    // First pass: Process StartNodeBefore events and build a properly balanced event stream
    let processed_events = process_start_node_before(&cleaned_events);

    // Second pass: Ensure the event stream is balanced
    let balanced_events = balance_events(&processed_events);

    // Third pass: Build the tree from the balanced events
    build_from_balanced_events(&balanced_events)
}

/// Process StartNodeBefore events and convert them to regular StartNode/FinishNode pairs
fn process_start_node_before(events: &[Event]) -> Vec<Event> {
    let mut result = Vec::with_capacity(events.len() * 2); // Allocate extra space
    let mut i = 0;

    while i < events.len() {
        match &events[i] {
            Event::StartNodeBefore { kind, before_pos } => {
                let before_pos = *before_pos;
                let kind = *kind;

                // Insert a StartNode at the current position
                result.push(Event::StartNode { kind });

                // Copy all subsequent events until we reach before_pos
                let mut j = i + 1;
                while j < events.len() && j < before_pos {
                    result.push(events[j].clone());
                    j += 1;
                }

                // Add the FinishNode for our StartNode
                result.push(Event::FinishNode);

                // Skip ahead to before_pos
                i = before_pos;
            }
            _ => {
                // Copy the event as-is
                result.push(events[i].clone());
                i += 1;
            }
        }
    }

    result
}

/// Ensure the event stream is balanced with matching StartNode and FinishNode events
fn balance_events(events: &[Event]) -> Vec<Event> {
    let mut result = Vec::with_capacity(events.len() * 2); // Allocate extra space
    let mut node_stack = Vec::new();

    // Ensure the event stream starts with a root node
    let mut has_root = false;
    for event in events {
        if let Event::StartNode { kind } = event {
            if *kind == SyntaxKind::ROOT {
                has_root = true;
                break;
            }
        }
    }

    if !has_root {
        result.push(Event::StartNode { kind: SyntaxKind::ROOT });
        node_stack.push(SyntaxKind::ROOT);
    }

    // Process all events
    for event in events {
        match event {
            Event::StartNode { kind } => {
                result.push(Event::StartNode { kind: *kind });
                node_stack.push(*kind);
            }
            Event::FinishNode => {
                if !node_stack.is_empty() {
                    result.push(Event::FinishNode);
                    node_stack.pop();
                }
                // Skip unmatched FinishNode events
            }
            _ => {
                result.push(event.clone());
            }
        }
    }

    // Close any remaining open nodes
    while let Some(_kind) = node_stack.pop() {
        result.push(Event::FinishNode);
    }

    result
}

/// Build the tree from the balanced events
fn build_from_balanced_events(events: &[Event]) -> GreenNode {
    let mut builder = GreenNodeBuilder::new();

    for event in events {
        match event {
            Event::StartNode { kind } => {
                builder.start_node(RamLang::kind_to_raw(*kind));
            }
            Event::FinishNode => {
                builder.finish_node();
            }
            Event::AddToken { kind, text, span: _ } => {
                builder.token(RamLang::kind_to_raw(*kind), text);
            }
            Event::Error { msg } => {
                // Create an error node with the error message
                builder.start_node(RamLang::kind_to_raw(SyntaxKind::ERROR));
                builder.token(RamLang::kind_to_raw(SyntaxKind::ERROR_TOKEN), msg);
                builder.finish_node();
            }
            _ => {
                // Ignore other events - they should have been processed already
            }
        }
    }

    builder.finish()
}
