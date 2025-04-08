//! Tests for the RAM parser.

use crate::SyntaxKind;
use crate::event::Event;
use crate::lexer::{Lexer, Token};
use crate::parser::{Input, Parser, SyntaxError};

/// Helper function to parse a string and return the events
fn parse_test(source: &str) -> (Vec<Event>, Vec<SyntaxError>) {
    crate::parse(source)
}

/// Helper function to check if parsing succeeded without errors
fn assert_no_errors(errors: &[SyntaxError]) {
    assert!(errors.is_empty(), "Expected no errors, got: {errors:?}");
}

#[test]
fn test_lexer() {
    let source = "LOAD 1 # Load value\nHALT\n";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();

    assert_eq!(tokens.len(), 9);
    assert_eq!(tokens[0].kind, SyntaxKind::LOAD_KW);
    assert_eq!(tokens[1].kind, SyntaxKind::WHITESPACE);
    assert_eq!(tokens[2].kind, SyntaxKind::NUMBER);
    assert_eq!(tokens[3].kind, SyntaxKind::WHITESPACE);
    assert_eq!(tokens[4].kind, SyntaxKind::HASH);
    assert_eq!(tokens[5].kind, SyntaxKind::COMMENT_TEXT);
    assert_eq!(tokens[6].kind, SyntaxKind::NEWLINE);
    assert_eq!(tokens[7].kind, SyntaxKind::HALT_KW);
    assert_eq!(tokens[8].kind, SyntaxKind::NEWLINE);
}

#[test]
fn test_basic_parser() {
    let source = "LOAD 1 # Load value\nHALT\n";
    let (events, errors) = parse_test(source);

    assert_no_errors(&errors);
    assert!(!events.is_empty(), "Expected events, got none");

    // Verify we have the expected structure
    let has_root = events
        .iter()
        .any(|e| matches!(e, Event::Placeholder { kind_slot } if *kind_slot == SyntaxKind::ROOT));
    assert!(has_root, "Missing Root node in events");
}

#[test]
fn test_complex_program() {
    let source = "# Simple test program
start:  LOAD =0    # Initialize accumulator
        STORE x    # Initialize x
loop:   LOAD x     # Load x
        SUB =10    # Check if x >= 10
        JGTZ end   # If so, end
        LOAD x     # Load x again
        ADD =1     # Increment
        STORE x    # Store back
        JUMP loop  # Repeat
end:    HALT       # Stop execution
x:      LOAD =0    # Data";

    let (events, errors) = parse_test(source);
    assert_no_errors(&errors);

    // Count how many instructions we have
    let instruction_count = events.iter().filter(|e| {
        matches!(e, Event::Placeholder { kind_slot } if *kind_slot == SyntaxKind::INSTRUCTION)
    }).count();

    assert_eq!(instruction_count, 11, "Expected 11 instructions");
}

#[test]
fn test_parse_label() {
    let source = "loop: LOAD 1\n";
    let (events, errors) = parse_test(source);

    assert_no_errors(&errors);

    // Check for LabelDef node
    let has_label = events.iter().any(
        |e| matches!(e, Event::Placeholder { kind_slot } if *kind_slot == SyntaxKind::LABEL_DEF),
    );
    assert!(has_label, "Missing LabelDef node in events");
}

#[test]
fn test_parse_indirect() {
    let source = "LOAD *1\n";
    let (events, errors) = parse_test(source);

    assert_no_errors(&errors);

    // Check for IndirectOperand node
    let has_indirect = events.iter().any(|e| matches!(e, Event::Placeholder { kind_slot } if *kind_slot == SyntaxKind::INDIRECT_OPERAND));
    assert!(has_indirect, "Missing IndirectOperand node in events");
}

#[test]
fn test_parse_immediate() {
    let source = "LOAD =1\n";
    let (events, errors) = parse_test(source);

    assert_no_errors(&errors);

    // Check for ImmediateOperand node
    let has_immediate = events.iter().any(|e| matches!(e, Event::Placeholder { kind_slot } if *kind_slot == SyntaxKind::IMMEDIATE_OPERAND));
    assert!(has_immediate, "Missing ImmediateOperand node in events");
}

#[test]
fn test_parse_error() {
    let source = "LOAD @1\n"; // @ is not a valid character
    let (_, errors) = parse_test(source);

    assert!(!errors.is_empty(), "Expected errors, got none");
    // The current parser produces an "Expected a number or identifier" error when it encounters @ in an operand
    assert!(
        errors[0].message.contains("Expected a number or identifier"),
        "Expected error about missing operand value"
    );
}

#[test]
fn test_unclosed_bracket() {
    let source = "LOAD x[5\n"; // Missing closing bracket
    let (_, errors) = parse_test(source);

    assert!(!errors.is_empty(), "Expected errors, got none");
    assert!(
        errors[0].message.contains("Unclosed bracket"),
        "Expected error about unclosed bracket"
    );
}

#[test]
fn test_extra_closing_bracket() {
    let source = "LOAD 5]\n"; // Extra closing bracket
    let (_, errors) = parse_test(source);

    assert!(!errors.is_empty(), "Expected errors, got none");
    assert!(
        errors[0].message.contains("Unexpected closing bracket"),
        "Expected error about unexpected closing bracket"
    );
}

#[test]
fn test_unexpected_opening_bracket() {
    let source = "LOAD [5]\n"; // Unexpected opening bracket
    let (_, errors) = parse_test(source);

    assert!(!errors.is_empty(), "Expected errors, got none");
    assert!(
        errors[0].message.contains("Array accessor to nowhere"),
        "Expected error about array accessor to nowhere"
    );
}

#[test]
fn test_valid_array_accessor() {
    let source = "LOAD x[5]\n"; // Valid array accessor
    let (_, errors) = parse_test(source);

    assert_no_errors(&errors);
}

#[test]
fn test_label_with_newline() {
    let source = "label:\nLOAD 1\n"; // Label followed by newline
    let (events, errors) = parse_test(source);

    assert_no_errors(&errors);

    // Check that we have at least one statement
    let stmt_count = events
        .iter()
        .filter(|e| matches!(e, Event::Placeholder { kind_slot } if *kind_slot == SyntaxKind::STMT))
        .count();

    assert!(stmt_count >= 1, "Expected at least 1 statement");

    // Check that we have a label definition
    let has_label = events.iter().any(
        |e| matches!(e, Event::Placeholder { kind_slot } if *kind_slot == SyntaxKind::LABEL_DEF),
    );
    assert!(has_label, "Missing LABEL_DEF node in events");

    // Check that we have an instruction
    let has_instruction = events.iter().any(
        |e| matches!(e, Event::Placeholder { kind_slot } if *kind_slot == SyntaxKind::INSTRUCTION),
    );
    assert!(has_instruction, "Missing INSTRUCTION node in events");
}

#[test]
fn test_comment_only() {
    let source = "# Just a comment\n";
    let (events, errors) = parse_test(source);

    assert_no_errors(&errors);

    // Check for Comment node
    let has_comment = events.iter().any(
        |e| matches!(e, Event::Placeholder { kind_slot } if *kind_slot == SyntaxKind::COMMENT),
    );
    assert!(has_comment, "Missing Comment node in events");
}

#[test]
fn test_empty_file() {
    let source = "";
    let (events, errors) = parse_test(source);

    assert_no_errors(&errors);
    assert!(!events.is_empty(), "Should have at least a Root node");
}

#[test]
fn test_marker_handling() {
    // This test verifies that our marker system works properly
    let input = Input::new(vec![
        Token { kind: SyntaxKind::LOAD_KW, text: "LOAD".to_string(), span: 0..4 },
        Token { kind: SyntaxKind::WHITESPACE, text: " ".to_string(), span: 4..5 },
        Token { kind: SyntaxKind::NUMBER, text: "42".to_string(), span: 5..7 },
    ]);

    let mut parser = Parser::new(&input);

    // Start outer node
    let outer = parser.start();
    parser.bump_any(); // LOAD
    parser.bump_any(); // whitespace

    // Start inner node
    let inner = parser.start();
    parser.bump_any(); // 42
    inner.complete(&mut parser, SyntaxKind::OPERAND_VALUE);

    // Complete outer node
    outer.complete(&mut parser, SyntaxKind::INSTRUCTION);

    // Verify events
    let (events, errors) = parser.finish();
    assert_no_errors(&errors);

    // Check event sequence
    assert!(
        matches!(events[0], Event::Placeholder { kind_slot } if kind_slot == SyntaxKind::INSTRUCTION)
    );
    assert!(
        matches!(events[3], Event::Placeholder { kind_slot } if kind_slot == SyntaxKind::OPERAND_VALUE)
    );
}

#[test]
fn test_precede_marker() {
    // This tests the marker.precede() functionality
    let input = Input::new(vec![
        Token { kind: SyntaxKind::LOAD_KW, text: "LOAD".to_string(), span: 0..4 },
        Token { kind: SyntaxKind::WHITESPACE, text: " ".to_string(), span: 4..5 },
        Token { kind: SyntaxKind::NUMBER, text: "42".to_string(), span: 5..7 },
    ]);

    let mut parser = Parser::new(&input);

    // Start value node first
    let value = parser.start();
    parser.bump_any(); // LOAD
    parser.bump_any(); // whitespace
    parser.bump_any(); // 42
    let completed = value.complete(&mut parser, SyntaxKind::OPERAND_VALUE);

    // Now precede it with an instruction node
    let instruction = completed.precede(&mut parser);
    instruction.complete(&mut parser, SyntaxKind::INSTRUCTION);

    // Check events
    let (events, errors) = parser.finish();
    assert_no_errors(&errors);

    // Verify we have a StartNodeBefore event
    let has_start_before = events.iter().any(|e| matches!(e, Event::StartNodeBefore { .. }));
    assert!(has_start_before, "Missing StartNodeBefore event");
}

#[test]
fn test_doc_comment() {
    let source = "#* Documentation comment\nLOAD 1\n";
    let (events, errors) = parse_test(source);

    assert_no_errors(&errors);

    // Check for DocComment node
    let has_doc_comment = events.iter().any(
        |e| matches!(e, Event::Placeholder { kind_slot } if *kind_slot == SyntaxKind::DOC_COMMENT),
    );
    assert!(has_doc_comment, "Missing DocComment node in events");

    // Verify the lexer correctly identified the #* token
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();

    // Find the HASH_STAR token
    let has_hash_star = tokens.iter().any(|t| t.kind == SyntaxKind::HASH_STAR);
    assert!(has_hash_star, "Missing HASH_STAR token");
}

#[test]
fn test_comment_group_same_type() {
    // Test with consecutive doc comments
    let source = "#* First doc comment\n#* Second doc comment\nLOAD 1\n";
    let (events, errors) = parse_test(source);

    assert_no_errors(&errors);

    // Check for CommentGroup node
    let has_comment_group = events.iter().any(
        |e| matches!(e, Event::Placeholder { kind_slot } if *kind_slot == SyntaxKind::COMMENT_GROUP),
    );
    assert!(has_comment_group, "Missing CommentGroup node in events");

    // Count doc comments - should be 2
    let doc_comment_count = events.iter().filter(
        |e| matches!(e, Event::Placeholder { kind_slot } if *kind_slot == SyntaxKind::DOC_COMMENT),
    ).count();
    assert_eq!(doc_comment_count, 2, "Expected exactly 2 doc comments");

    // Test with consecutive regular comments
    let source = "# First regular comment\n# Second regular comment\nLOAD 1\n";
    let (events, errors) = parse_test(source);

    assert_no_errors(&errors);

    // Check for CommentGroup node
    let has_comment_group = events.iter().any(
        |e| matches!(e, Event::Placeholder { kind_slot } if *kind_slot == SyntaxKind::COMMENT_GROUP),
    );
    assert!(has_comment_group, "Missing CommentGroup node in events");

    // Count regular comments - should be 2
    let regular_comment_count = events
        .iter()
        .filter(
            |e| matches!(e, Event::Placeholder { kind_slot } if *kind_slot == SyntaxKind::COMMENT),
        )
        .count();
    assert_eq!(regular_comment_count, 2, "Expected exactly 2 regular comments");
}

#[test]
fn test_comment_groups_different_types() {
    // Test with mixed comment types - should create separate groups
    let source = "#* Doc comment\n# Regular comment\n#* Another doc comment\nLOAD 1\n";
    let (events, errors) = parse_test(source);

    assert_no_errors(&errors);

    // Count CommentGroup nodes - should be at least 2 (one for each type)
    let comment_group_count = events.iter().filter(
        |e| matches!(e, Event::Placeholder { kind_slot } if *kind_slot == SyntaxKind::COMMENT_GROUP),
    ).count();
    assert!(
        comment_group_count >= 2,
        "Expected at least 2 comment groups for different comment types"
    );

    // Check that we have both types of comments
    let has_doc_comment = events.iter().any(
        |e| matches!(e, Event::Placeholder { kind_slot } if *kind_slot == SyntaxKind::DOC_COMMENT),
    );
    assert!(has_doc_comment, "Missing DocComment node in events");

    let has_regular_comment = events.iter().any(
        |e| matches!(e, Event::Placeholder { kind_slot } if *kind_slot == SyntaxKind::COMMENT),
    );
    assert!(has_regular_comment, "Missing Comment node in events");
}

#[test]
fn test_label_requires_instruction() {
    // Test with a label followed by an instruction on the same line
    let source = "label: LOAD 1\n";
    let (_events, errors) = parse_test(source);
    assert_no_errors(&errors);

    // Test with a label followed by an instruction on the next line
    let source = "label:\nLOAD 1\n";
    let (_events, errors) = parse_test(source);
    assert_no_errors(&errors);

    // Test with a label not followed by an instruction
    let source = "label:\n";
    let (_events, errors) = parse_test(source);

    // Test with a label not followed by an instruction, but with an instruction later in the file
    let source = "label:\n\n\nLOAD 1\n";
    let (_events, errors_with_later_instr) = parse_test(source);

    // Should have an error
    assert!(!errors.is_empty(), "Expected an error for label without instruction");
    assert!(
        !errors_with_later_instr.is_empty(),
        "Expected an error for label without immediate instruction"
    );

    // Check that the error with a later instruction has multiple labeled spans
    let error_with_later = errors_with_later_instr
        .iter()
        .find(|e| e.message.contains("Label must be followed by an instruction"))
        .unwrap();

    // Should have at least two labeled spans (one for the label, one for the instruction)
    assert!(
        error_with_later.labeled_spans.len() >= 2,
        "Error should have at least two labeled spans when there's an instruction later in the file"
    );

    // The second span should mention the instruction
    let has_instruction_span = error_with_later
        .labeled_spans
        .iter()
        .any(|(_, label)| label.contains("instruction found here"));
    assert!(has_instruction_span, "Error should have a span pointing to the later instruction");

    // Check that the error message is correct
    let has_correct_error =
        errors.iter().any(|e| e.message.contains("Label must be followed by an instruction"));
    assert!(has_correct_error, "Expected error message about label requiring instruction");

    // Check that the error span points to the label, not the end of the file
    let error = errors
        .iter()
        .find(|e| e.message.contains("Label must be followed by an instruction"))
        .unwrap();

    // The error should have at least one labeled span
    assert!(!error.labeled_spans.is_empty(), "Error should have at least one labeled span");

    // The first labeled span should be the label
    let span = &error.labeled_spans[0].0;

    // The error span should start at the beginning of the line (where "label:" is)
    assert_eq!(span.start, 0, "Error span should start at the beginning of the line");

    // The error span should not be empty
    assert!(span.end > 0, "Error span should not be empty");
}

#[test]
fn test_multiline_comment_grouping() {
    // Test with multiple consecutive comments of the same type across multiple lines
    let source = "# First comment\n# Second comment\n# Third comment\nLOAD 1\n";
    let (events, errors) = parse_test(source);

    assert_no_errors(&errors);

    // Count CommentGroup nodes - should be exactly 1 for all regular comments
    let comment_group_count = events.iter().filter(
        |e| matches!(e, Event::Placeholder { kind_slot } if *kind_slot == SyntaxKind::COMMENT_GROUP),
    ).count();
    assert_eq!(comment_group_count, 1, "Expected exactly 1 comment group for all regular comments");

    // Count regular comments - should be 3
    let regular_comment_count = events
        .iter()
        .filter(
            |e| matches!(e, Event::Placeholder { kind_slot } if *kind_slot == SyntaxKind::COMMENT),
        )
        .count();
    assert_eq!(regular_comment_count, 3, "Expected exactly 3 regular comments");

    // Test with multiple consecutive doc comments across multiple lines
    let source = "#* First doc comment\n#* Second doc comment\n#* Third doc comment\nLOAD 1\n";
    let (events, errors) = parse_test(source);

    assert_no_errors(&errors);

    // Count CommentGroup nodes - should be exactly 1 for all doc comments
    let comment_group_count = events.iter().filter(
        |e| matches!(e, Event::Placeholder { kind_slot } if *kind_slot == SyntaxKind::COMMENT_GROUP),
    ).count();
    assert_eq!(comment_group_count, 1, "Expected exactly 1 comment group for all doc comments");

    // Count doc comments - should be 3
    let doc_comment_count = events.iter().filter(
        |e| matches!(e, Event::Placeholder { kind_slot } if *kind_slot == SyntaxKind::DOC_COMMENT),
    ).count();
    assert_eq!(doc_comment_count, 3, "Expected exactly 3 doc comments");
}
