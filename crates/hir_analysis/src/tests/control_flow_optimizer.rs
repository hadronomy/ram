//! Tests for the control flow optimizer

use base_db::input::FileId;
use hir::body::{Body, Expr, ExprKind, Instruction, Label, Literal};
use hir::expr::ExprId;
use hir::ids::{DefId, LocalDefId};

use crate::analyzers::constant_propagation::ConstantPropagationAnalysis;
use crate::analyzers::control_flow::ControlFlowAnalysis;
use crate::analyzers::control_flow_optimizer::ControlFlowOptimizer;
use crate::analyzers::data_flow::DataFlowAnalysis;
use crate::context::AnalysisContext;
use crate::pass::AnalysisPass;

#[allow(clippy::field_reassign_with_default)]
/// Create a test body with a conditional jump that will always take the same branch
fn create_test_body_with_constant_condition() -> Body {
    let mut body = Body::default();
    body.owner = DefId { file_id: FileId(0), local_id: LocalDefId(0) };

    // Add instructions
    // LOAD =10  ; Load constant 10 into accumulator
    // JGTZ loop ; Jump to loop if accumulator > 0 (always true)
    // LOAD =20  ; This will never be executed
    // HALT      ; This will never be executed
    // loop: LOAD =30 ; This will always be executed
    // HALT      ; This will always be executed

    body.instructions.push(Instruction {
        id: LocalDefId(0),
        opcode: "LOAD".to_string(),
        operand: Some(ExprId(0)),
        label_name: None,
        span: 0..0, // Default span
    });

    body.instructions.push(Instruction {
        id: LocalDefId(1),
        opcode: "JGTZ".to_string(),
        operand: Some(ExprId(1)),
        label_name: None,
        span: 0..0, // Default span
    });

    body.instructions.push(Instruction {
        id: LocalDefId(2),
        opcode: "LOAD".to_string(),
        operand: Some(ExprId(2)),
        label_name: None,
        span: 0..0, // Default span
    });

    body.instructions.push(Instruction {
        id: LocalDefId(3),
        opcode: "HALT".to_string(),
        operand: None,
        label_name: None,
        span: 0..0, // Default span
    });

    body.instructions.push(Instruction {
        id: LocalDefId(4),
        opcode: "LOAD".to_string(),
        operand: Some(ExprId(3)),
        label_name: Some("loop".to_string()),
        span: 0..0, // Default span
    });

    body.instructions.push(Instruction {
        id: LocalDefId(5),
        opcode: "HALT".to_string(),
        operand: None,
        label_name: None,
        span: 0..0, // Default span
    });

    // Add a label
    body.labels.push(Label {
        id: LocalDefId(6),
        name: "loop".to_string(),
        instruction_id: Some(LocalDefId(4)),
        span: 0..0, // Default span
    });

    // Add expressions
    body.exprs.push(Expr {
        id: ExprId(0),
        kind: ExprKind::Literal(Literal::Int(10)),
        span: 0..0, // Default span
    });

    body.exprs.push(Expr {
        id: ExprId(1),
        kind: ExprKind::Literal(Literal::Label("loop".to_string())),
        span: 0..0, // Default span
    });

    body.exprs.push(Expr {
        id: ExprId(2),
        kind: ExprKind::Literal(Literal::Int(20)),
        span: 0..0, // Default span
    });

    body.exprs.push(Expr {
        id: ExprId(3),
        kind: ExprKind::Literal(Literal::Int(30)),
        span: 0..0, // Default span
    });

    body
}

#[test]
fn test_control_flow_optimizer() {
    // Create a new context with the test body
    let body = create_test_body_with_constant_condition();
    let mut context = AnalysisContext::from(body);

    // Run the control flow analysis first (dependency)
    let cf_analysis = ControlFlowAnalysis;
    let cf_result = cf_analysis.run(&mut context).unwrap();
    context.store_result::<ControlFlowAnalysis>(cf_result);

    // Run the data flow analysis (dependency for constant propagation)
    let df_analysis = DataFlowAnalysis;
    let df_result = df_analysis.run(&mut context).unwrap();
    context.store_result::<DataFlowAnalysis>(df_result);

    // Run the constant propagation analysis (dependency)
    let const_prop_analysis = ConstantPropagationAnalysis;
    let const_prop_result = const_prop_analysis.run(&mut context).unwrap();
    context.store_result::<ConstantPropagationAnalysis>(const_prop_result);

    // Run the control flow optimizer
    let optimizer = ControlFlowOptimizer;
    let result = optimizer.run(&mut context).unwrap();

    // Check that the optimizer identified the conditional jump
    assert!(!result.optimized_edges.is_empty());

    // Get the optimized CFG
    let optimized_cfg = result.cfg.lock().unwrap();

    // Check that the unreachable instructions are marked as such
    let unreachable = optimized_cfg.find_unreachable_nodes();
    assert!(!unreachable.is_empty());

    // Check that there's a path from the entry to the loop label
    let entry_node = optimized_cfg.entry_node().unwrap();
    let loop_instr_id = LocalDefId(4); // The instruction with the loop label
    let loop_node = optimized_cfg.get_node_by_instruction(loop_instr_id).unwrap();

    assert!(optimized_cfg.has_path(entry_node, loop_node));

    // Check that there's no path to the unreachable instructions
    let unreachable_instr_id = LocalDefId(2); // The LOAD =20 instruction
    let unreachable_node = optimized_cfg.get_node_by_instruction(unreachable_instr_id).unwrap();

    assert!(!optimized_cfg.has_path(entry_node, unreachable_node));
}
