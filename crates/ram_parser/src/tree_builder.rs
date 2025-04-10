use cstree::interning::Interner;
use cstree::prelude::*;

use crate::SyntaxKind;
use crate::event::Event;
use crate::syntax_kind::Ram;

/// A builder for creating syntax trees from parser events
///
/// This struct encapsulates the process of transforming parser events
/// into a proper syntax tree structure. It handles several phases:
///
/// 1. Cleaning events (removing tombstones, converting placeholders)
/// 2. Processing hierarchical node relationships (StartNodeBefore events)
/// 3. Ensuring the event stream is properly balanced
/// 4. Building the final green tree
pub struct TreeBuilder {
    events: Vec<Event>,
    builder: GreenNodeBuilder<'static, 'static, Ram>,
}

impl TreeBuilder {
    /// Create a new TreeBuilder from a list of parser events
    pub fn new(events: Vec<Event>) -> Self {
        Self { events, builder: GreenNodeBuilder::new() }
    }

    /// Build the tree from the events
    ///
    /// This is the main entry point that processes all events and builds
    /// the final syntax tree.
    pub fn build(mut self) -> (GreenNode, impl Interner) {
        // Process the events in multiple passes to build a proper tree
        if !self.events.is_empty() {
            self.clean_events();
            self.process_start_node_before();
            self.balance_events();
        }

        self.build_tree()
    }

    /// Cleans the event stream by removing tombstones and converting placeholders
    fn clean_events(&mut self) {
        self.events = self
            .events
            .drain(..)
            .filter_map(|event| match event {
                Event::Tombstone => None, // Skip tombstones
                Event::Placeholder { kind_slot } => Some(Event::StartNode { kind: kind_slot }),
                _ => Some(event), // Keep all other events as-is
            })
            .collect();
    }

    /// Process StartNodeBefore events and convert them to regular StartNode/FinishNode pairs
    ///
    /// Handles the conversion of StartNodeBefore events (which represent nodes that should
    /// "wrap around" previously created nodes) into proper StartNode/FinishNode pairs.
    fn process_start_node_before(&mut self) {
        let mut result = Vec::with_capacity(self.events.len() * 2); // Allocate extra space
        let mut i = 0;
        let events = std::mem::take(&mut self.events);

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

        self.events = result;
    }

    /// Ensure the event stream is balanced with matching StartNode and FinishNode events
    ///
    /// Guarantees that every StartNode event has a corresponding FinishNode,
    /// and that the stream starts with a ROOT node.
    fn balance_events(&mut self) {
        let mut result = Vec::with_capacity(self.events.len() * 2);
        let mut node_stack = Vec::new();

        // Check if we need to insert a root node at the beginning
        let has_root = self
            .events
            .iter()
            .any(|event| matches!(event, Event::StartNode { kind } if *kind == SyntaxKind::ROOT));

        if !has_root {
            result.push(Event::StartNode { kind: SyntaxKind::ROOT });
            node_stack.push(SyntaxKind::ROOT);
        }

        // Process all events
        for event in &self.events {
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

        self.events = result;
    }

    /// Build the tree from the processed events
    fn build_tree(mut self) -> (GreenNode, impl Interner) {
        // Handle empty events by creating a minimal valid tree
        if self.events.is_empty() {
            self.builder.start_node(SyntaxKind::ROOT);
            self.builder.finish_node();
        } else {
            for event in &self.events {
                match event {
                    Event::StartNode { kind } => {
                        self.builder.start_node(*kind);
                    }
                    Event::FinishNode => {
                        self.builder.finish_node();
                    }
                    Event::AddToken { kind, text, span: _ } => {
                        self.builder.token(*kind, text);
                    }
                    Event::Error { msg } => {
                        // Create an error node with the error message
                        self.builder.start_node(Ram::ERROR);
                        self.builder.token(Ram::ERROR_TOKEN, msg);
                        self.builder.finish_node();
                    }
                    _ => {
                        // Other events should have been processed in earlier passes
                    }
                }
            }
        }

        let (tree, cache) = self.builder.finish();
        (tree, cache.unwrap().into_interner().unwrap())
    }
}

/// Builds a [GreenNode](`cstree::green::node::GreenNode`) from a list of events
///
/// This function takes the events produced by the parser and transforms them
/// into a proper syntax tree structure by using the TreeBuilder struct.
///
/// **NOTE:** This is a convenience function
pub fn build_tree(events: Vec<Event>) -> (GreenNode, impl Interner) {
    TreeBuilder::new(events).build()
}
