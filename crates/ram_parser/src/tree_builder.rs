use rowan::{GreenNode, GreenNodeBuilder, Language};

use crate::SyntaxKind;
use crate::event::Event;
use crate::language::RamLang;

/// Builds a Rowan Green Tree from a list of events
///
/// This function takes the events produced by the parser and transforms them
/// into a proper syntax tree structure. It handles several phases of processing:
///
/// 1. Cleaning events (removing tombstones, converting placeholders)
/// 2. Processing hierarchical node relationships (StartNodeBefore events)
/// 3. Ensuring the event stream is properly balanced
/// 4. Building the final green tree
pub fn build_tree(events: Vec<Event>) -> GreenNode {
    // Handle the empty events case by creating a minimal valid tree
    if events.is_empty() {
        return create_minimal_tree();
    }

    // Process the events in multiple passes to build a proper tree
    let cleaned_events = clean_events(events);
    let processed_events = process_start_node_before(&cleaned_events);
    let balanced_events = balance_events(&processed_events);
    build_from_balanced_events(&balanced_events)
}

/// Creates a minimal valid syntax tree with just a root node
fn create_minimal_tree() -> GreenNode {
    let mut builder = GreenNodeBuilder::new();
    builder.start_node(RamLang::kind_to_raw(SyntaxKind::ROOT));
    builder.finish_node();
    builder.finish()
}

/// Cleans the event stream by removing tombstones and converting placeholders
fn clean_events(events: Vec<Event>) -> Vec<Event> {
    events
        .into_iter()
        .filter_map(|event| match event {
            Event::Tombstone => None, // Skip tombstones
            Event::Placeholder { kind_slot } => Some(Event::StartNode { kind: kind_slot }),
            _ => Some(event), // Keep all other events as-is
        })
        .collect()
}

/// Process StartNodeBefore events and convert them to regular StartNode/FinishNode pairs
///
/// Handles the conversion of StartNodeBefore events (which represent nodes that should
/// "wrap around" previously created nodes) into proper StartNode/FinishNode pairs.
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
                let events_to_include = &events[i + 1..before_pos.min(events.len())];
                result.extend_from_slice(events_to_include);

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
///
/// Guarantees that every StartNode event has a corresponding FinishNode,
/// and that the stream starts with a ROOT node.
fn balance_events(events: &[Event]) -> Vec<Event> {
    let mut result = Vec::with_capacity(events.len() * 2);
    let mut node_stack = Vec::new();

    // Check if we need to insert a root node at the beginning
    let has_root = events
        .iter()
        .any(|event| matches!(event, Event::StartNode { kind } if *kind == SyntaxKind::ROOT));

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
    while node_stack.pop().is_some() {
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
                // Other events should have been processed in earlier passes
            }
        }
    }

    builder.finish()
}
