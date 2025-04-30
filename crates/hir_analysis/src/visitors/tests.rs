//! Tests for the visitor pattern
//!
//! This module contains tests for the visitor pattern implementation.

use hir::body::{Body, Expr, ExprKind, Instruction, Label, Literal};
use hir::expr::ExprId;
use hir::ids::LocalDefId;

use crate::visitors::examples::{collect_int_literals, count_instructions, find_instruction};

/// Create a test body with some instructions and literals
fn create_test_body() -> Body {
    let mut body = Body::default();

    // Add some instructions
    body.instructions.push(Instruction {
        id: LocalDefId(0),
        opcode: "LOAD".to_string(),
        operand: Some(ExprId(0)),
        label_name: None,
    });

    body.instructions.push(Instruction {
        id: LocalDefId(1),
        opcode: "ADD".to_string(),
        operand: Some(ExprId(1)),
        label_name: None,
    });

    body.instructions.push(Instruction {
        id: LocalDefId(2),
        opcode: "STORE".to_string(),
        operand: Some(ExprId(2)),
        label_name: None,
    });

    // Add some expressions
    body.exprs.push(Expr { id: ExprId(0), kind: ExprKind::Literal(Literal::Int(10)) });

    body.exprs.push(Expr { id: ExprId(1), kind: ExprKind::Literal(Literal::Int(20)) });

    body.exprs.push(Expr { id: ExprId(2), kind: ExprKind::Literal(Literal::Int(30)) });

    // Add a label
    body.labels.push(Label {
        id: LocalDefId(3),
        name: "LOOP".to_string(),
        instruction_id: Some(LocalDefId(0)),
    });

    body
}

#[test]
fn test_count_instructions() {
    let body = create_test_body();
    let count = count_instructions(&body);
    assert_eq!(count, 3);
}

#[test]
fn test_collect_int_literals() {
    let body = create_test_body();
    let literals = collect_int_literals(&body);
    assert_eq!(literals, vec![10, 20, 30]);
}

#[test]
fn test_find_instruction() {
    let body = create_test_body();

    // Find an existing instruction
    let instruction = find_instruction(&body, "ADD");
    assert!(instruction.is_some());
    assert_eq!(instruction.unwrap().opcode, "ADD");

    // Try to find a non-existent instruction
    let instruction = find_instruction(&body, "JUMP");
    assert!(instruction.is_none());
}
