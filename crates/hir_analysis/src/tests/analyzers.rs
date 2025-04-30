//! Tests for the analyzers module
//!
//! This module contains tests for the various analyzers in the analyzers module.

use std::sync::Arc;

use hir::body::{Body, Expr, ExprKind, Instruction, Label, Literal};
use hir::expr::ExprId;
use hir::ids::LocalDefId;

use crate::AnalysisPipeline;
use crate::analyzers::control_flow::ControlFlowAnalysis;
use crate::analyzers::data_flow::DataFlowAnalysis;
use crate::analyzers::instruction_validation::InstructionValidationAnalysis;

/// Create a test body with some instructions
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

    body.instructions.push(Instruction {
        id: LocalDefId(3),
        opcode: "JUMP".to_string(),
        operand: Some(ExprId(3)),
        label_name: None,
    });

    // Add a label
    body.labels.push(Label {
        id: LocalDefId(4),
        name: "LOOP".to_string(),
        instruction_id: Some(LocalDefId(0)),
    });

    // Add some expressions
    body.exprs.push(Expr { id: ExprId(0), kind: ExprKind::Literal(Literal::Int(10)) });

    body.exprs.push(Expr { id: ExprId(1), kind: ExprKind::Literal(Literal::Int(20)) });

    body.exprs.push(Expr { id: ExprId(2), kind: ExprKind::Literal(Literal::Int(30)) });

    body.exprs
        .push(Expr { id: ExprId(3), kind: ExprKind::Literal(Literal::Label("LOOP".to_string())) });

    body
}

#[test]
fn test_control_flow_analysis() {
    // Create a pipeline and register the control flow analysis pass
    let mut pipeline = AnalysisPipeline::new();
    pipeline.register::<ControlFlowAnalysis>().unwrap();

    // Run the analysis on a test body
    let body = Arc::new(create_test_body());
    let context = pipeline.analyze(body).unwrap();

    // Get the control flow graph
    let cfg = context.get_result::<ControlFlowAnalysis>().unwrap();

    // Check that the graph has the correct number of nodes
    assert_eq!(cfg.get_node(0).instruction_id, Some(LocalDefId(0)));
    assert_eq!(cfg.get_node(1).instruction_id, Some(LocalDefId(1)));
    assert_eq!(cfg.get_node(2).instruction_id, Some(LocalDefId(2)));
    assert_eq!(cfg.get_node(3).instruction_id, Some(LocalDefId(3)));

    // Check that there's an edge from JUMP to LOAD (the label target)
    let jump_node = 3;
    let load_node = 0;
    let outgoing_edges = cfg.get_outgoing_edges(jump_node);

    assert!(outgoing_edges.iter().any(|edge| edge.target == load_node));
}

#[test]
fn test_data_flow_analysis() {
    // Create a pipeline and register the control flow and data flow analysis passes
    let mut pipeline = AnalysisPipeline::new();
    pipeline.register::<ControlFlowAnalysis>().unwrap();
    pipeline.register::<DataFlowAnalysis>().unwrap();

    // Run the analysis on a test body
    let body = Arc::new(create_test_body());
    let context = pipeline.analyze(body).unwrap();

    // Get the data flow graph
    let dfg = context.get_result::<DataFlowAnalysis>().unwrap();

    // Check that the graph has nodes for all instructions
    assert!(dfg.get_node_by_instruction(LocalDefId(0)).is_some());
    assert!(dfg.get_node_by_instruction(LocalDefId(1)).is_some());
    assert!(dfg.get_node_by_instruction(LocalDefId(2)).is_some());
    assert!(dfg.get_node_by_instruction(LocalDefId(3)).is_some());
}

#[test]
fn test_instruction_validation() {
    // Create a pipeline and register the instruction validation pass
    let mut pipeline = AnalysisPipeline::new();
    pipeline.register::<InstructionValidationAnalysis>().unwrap();

    // Run the analysis on a test body
    let body = Arc::new(create_test_body());
    let context = pipeline.analyze(body).unwrap();

    // Check that there are no errors
    assert!(!context.has_errors());

    // Create a body with an invalid instruction
    let mut invalid_body = create_test_body();
    invalid_body.instructions.push(Instruction {
        id: LocalDefId(5),
        opcode: "INVALID".to_string(),
        operand: None,
        label_name: None,
    });

    // Run the analysis on the invalid body
    let invalid_body = Arc::new(invalid_body);
    let invalid_context = pipeline.analyze(invalid_body).unwrap();

    // Check that there are errors
    assert!(invalid_context.has_errors());
    assert!(
        invalid_context
            .diagnostics()
            .diagnostics()
            .iter()
            .any(|d| d.message.contains("Unknown instruction"))
    );
}
