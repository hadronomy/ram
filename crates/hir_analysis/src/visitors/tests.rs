use hir::body::{Body, Expr, ExprKind, Instruction, Label, Literal};
use hir::expr::ExprId;
use hir::ids::LocalDefId;

use crate::visitors::traits::Visitor;
use crate::visitors::walkers::walk_body;

/// Test visitor that counts instructions
struct InstructionCounter {
    count: usize,
}

/// Create a test body for visitor tests
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

    // Add a label
    body.labels.push(Label {
        id: LocalDefId(3),
        name: "LOOP".to_string(),
        instruction_id: Some(LocalDefId(0)),
        span: 0..0, // Default span
    });

    body
}

impl Visitor for InstructionCounter {
    type Result = usize;

    fn visit_instruction(
        &mut self,
        _instruction: &Instruction,
    ) -> std::ops::ControlFlow<Self::Result> {
        self.count += 1;
        std::ops::ControlFlow::Continue(())
    }

    fn finish(self) -> Self::Result {
        self.count
    }
}

/// Test visitor that finds a specific instruction
struct InstructionFinder {
    opcode: String,
    found: bool,
}

impl Visitor for InstructionFinder {
    type Result = bool;

    fn visit_instruction(
        &mut self,
        instruction: &Instruction,
    ) -> std::ops::ControlFlow<Self::Result> {
        if instruction.opcode == self.opcode {
            self.found = true;
            return std::ops::ControlFlow::Break(true);
        }
        std::ops::ControlFlow::Continue(())
    }

    fn finish(self) -> Self::Result {
        self.found
    }
}

/// Test visitor that collects all integer literals
struct IntLiteralCollector {
    literals: Vec<i64>,
}

impl Visitor for IntLiteralCollector {
    type Result = Vec<i64>;

    fn visit_expr(&mut self, expr: &Expr) -> std::ops::ControlFlow<Self::Result> {
        if let ExprKind::Literal(Literal::Int(value)) = &expr.kind {
            self.literals.push(*value);
        }
        std::ops::ControlFlow::Continue(())
    }

    fn visit_body(&mut self, _body: &Body) -> std::ops::ControlFlow<Self::Result> {
        // Clear the literals to avoid duplicates when testing
        self.literals.clear();
        std::ops::ControlFlow::Continue(())
    }

    fn finish(self) -> Self::Result {
        self.literals
    }
}

#[test]
fn test_count_instructions() {
    let body = create_test_body();
    let counter = InstructionCounter { count: 0 };
    let result = walk_body(counter, &body);
    assert_eq!(result, 3);
}

#[test]
fn test_find_instruction() {
    let body = create_test_body();

    // Test finding an instruction that exists
    let finder = InstructionFinder { opcode: "ADD".to_string(), found: false };
    let result = walk_body(finder, &body);
    assert!(result);

    // Test finding an instruction that doesn't exist
    let finder = InstructionFinder { opcode: "SUB".to_string(), found: false };
    let result = walk_body(finder, &body);
    assert!(!result);
}

#[test]
#[allow(clippy::field_reassign_with_default)]
fn test_collect_int_literals() {
    let body = create_test_body();
    // Create a new collector for each test
    let collector = IntLiteralCollector { literals: Vec::new() };
    // Create a new body with only the expressions we want to test
    let mut test_body = Body::default();
    test_body.exprs = body.exprs.clone();
    let result = walk_body(collector, &test_body);
    assert_eq!(result, vec![10, 20, 30]);
}
