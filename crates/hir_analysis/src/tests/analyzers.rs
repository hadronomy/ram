use std::collections::HashSet;

use hir::body::{Body, Expr, ExprKind, Instruction, Label, Literal};
use hir::expr::ExprId;
use hir::ids::LocalDefId;

use crate::analyzers::control_flow::ControlFlowAnalysis;
use crate::analyzers::data_flow::DataFlowAnalysis;
use crate::analyzers::instruction_validation::InstructionValidationAnalysis;
use crate::context::AnalysisContext;
use crate::pass::AnalysisPass;

/// Create a test body for analyzer tests
fn create_test_body() -> Body {
    let mut body = Body::default();

    // Add some instructions
    body.instructions.push(Instruction {
        id: LocalDefId(0),
        opcode: "LOAD".to_string(),
        operand: Some(ExprId(0)),
        label_name: None,
        span: 0..0, // Default span
    });

    body.instructions.push(Instruction {
        id: LocalDefId(1),
        opcode: "ADD".to_string(),
        operand: Some(ExprId(1)),
        label_name: None,
        span: 0..0, // Default span
    });

    body.instructions.push(Instruction {
        id: LocalDefId(2),
        opcode: "STORE".to_string(),
        operand: Some(ExprId(2)),
        label_name: None,
        span: 0..0, // Default span
    });

    body.instructions.push(Instruction {
        id: LocalDefId(3),
        opcode: "JUMP".to_string(),
        operand: Some(ExprId(3)),
        label_name: None,
        span: 0..0, // Default span
    });

    // Add a label
    body.labels.push(Label {
        id: LocalDefId(4),
        name: "LOOP".to_string(),
        instruction_id: Some(LocalDefId(0)),
        span: 0..0, // Default span
    });

    // Add some expressions
    body.exprs.push(Expr {
        id: ExprId(0),
        kind: ExprKind::Literal(Literal::Int(10)),
        span: 0..0, // Default span
    });

    body.exprs.push(Expr {
        id: ExprId(1),
        kind: ExprKind::Literal(Literal::Int(20)),
        span: 0..0, // Default span
    });

    body.exprs.push(Expr {
        id: ExprId(2),
        kind: ExprKind::Literal(Literal::Int(30)),
        span: 0..0, // Default span
    });

    body.exprs.push(Expr {
        id: ExprId(3),
        kind: ExprKind::Literal(Literal::Label("LOOP".to_string())),
        span: 0..0, // Default span
    });

    body
}

#[test]
fn test_control_flow_analysis() {
    let body = create_test_body();
    let mut context = AnalysisContext::from(body);

    // Run the control flow analysis
    let analysis = ControlFlowAnalysis;
    let result = analysis.run(&mut context).unwrap();

    // Check that the graph has nodes for all instructions
    let node_indices = result.node_indices();
    assert_eq!(node_indices.len(), 4);

    // Check that the nodes have the correct instruction IDs
    let mut found_instrs = HashSet::new();
    for node_idx in node_indices {
        if let Some(instr_id) = result.get_node(node_idx).instruction_id {
            found_instrs.insert(instr_id);
        }
    }

    assert!(found_instrs.contains(&LocalDefId(0)));
    assert!(found_instrs.contains(&LocalDefId(1)));
    assert!(found_instrs.contains(&LocalDefId(2)));
    assert!(found_instrs.contains(&LocalDefId(3)));

    // Check that there's a path from JUMP to LOAD (the label target)
    let jump_node_idx = result.get_node_by_instruction(LocalDefId(3)).unwrap();
    let load_node_idx = result.get_node_by_instruction(LocalDefId(0)).unwrap();

    assert!(result.has_path(jump_node_idx, load_node_idx));
}

#[test]
fn test_data_flow_analysis() {
    // Create a new context with the test body
    let mut context = AnalysisContext::from(create_test_body());

    // Run the control flow analysis first (dependency)
    let cf_analysis = ControlFlowAnalysis;
    let cf_result = cf_analysis.run(&mut context).unwrap();

    // Store the result in the context
    context.store_result::<ControlFlowAnalysis>(cf_result);

    // Run the data flow analysis
    let analysis = DataFlowAnalysis;
    let result = analysis.run(&mut context).unwrap();

    // Check that the graph has nodes for all instructions
    assert!(result.node_count() > 0);

    // Check that there are no uninitialized reads
    let uninitialized = result.find_uninitialized_reads();
    assert!(uninitialized.is_empty());
}

#[test]
fn test_instruction_validation() {
    let body = create_test_body();
    let mut context = AnalysisContext::from(body);

    // Run the instruction validation analysis
    let analysis = InstructionValidationAnalysis;
    analysis.run(&mut context).unwrap();

    // Check that there are no errors
    assert!(!context.has_errors());

    // Create a body with an invalid instruction
    let mut invalid_body = Body::default();
    invalid_body.instructions.push(Instruction {
        id: LocalDefId(0),
        opcode: "INVALID".to_string(),
        operand: None,
        label_name: None,
        span: 0..0, // Default span
    });

    let mut invalid_context = AnalysisContext::from(invalid_body);

    // Run the instruction validation analysis
    let analysis = InstructionValidationAnalysis;
    let _ = analysis.run(&mut invalid_context);

    // Check that there are errors
    assert!(invalid_context.has_errors());
}
