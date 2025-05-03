use base_db::input::FileId;
use hir::ids::DefId;
use hir::lower::lower_program;
use hir_def::item_tree::ItemTree;
use ram_syntax::{AstNode, ast};

#[test]
fn test_array_access_lowering() {
    // Create a simple program with array access
    let source = "LOAD 2[3]\n";

    // Parse the source into an AST
    let (events, errors) = ram_parser::parse(source);
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    // Convert events to a syntax tree
    let (tree, cache) = ram_parser::build_tree(events);
    let syntax_node = ram_syntax::SyntaxNode::new_root_with_resolver(tree, cache);

    // Extract the AST program
    let program = ast::Program::cast(syntax_node).unwrap();

    // Create a dummy ItemTree
    let item_tree = ItemTree::default();

    // Create a dummy DefId and FileId
    let owner = DefId { file_id: FileId(0), local_id: hir::ids::LocalDefId(0) };
    let file_id = FileId(0);

    // Lower the program to HIR
    let body = lower_program(&program, owner, file_id, &item_tree).unwrap();

    // Verify that the array access was lowered correctly
    assert_eq!(
        body.instructions.len(),
        1,
        "Expected 1 instruction, got {}",
        body.instructions.len()
    );

    // Get the instruction
    let instruction = &body.instructions[0];
    assert_eq!(instruction.opcode, "LOAD", "Expected LOAD instruction, got {}", instruction.opcode);

    // Print all expressions for debugging
    for (i, expr) in body.exprs.iter().enumerate() {
        println!("Expr {}: {:?}", i, expr);
    }

    // Verify that the operand exists
    let operand_id = instruction.operand.unwrap();
    let operand = &body.exprs[operand_id.0 as usize];
    println!("Operand: {:?}", operand);
}

#[test]
fn test_indirect_array_access_lowering() {
    // Create a simple program with indirect array access
    let source = "LOAD *2[3]\n";

    // Parse the source into an AST
    let (events, errors) = ram_parser::parse(source);
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    // Convert events to a syntax tree
    let (tree, cache) = ram_parser::build_tree(events);
    let syntax_node = ram_syntax::SyntaxNode::new_root_with_resolver(tree, cache);

    // Extract the AST program
    let program = ast::Program::cast(syntax_node).unwrap();

    // Create a dummy ItemTree
    let item_tree = ItemTree::default();

    // Create a dummy DefId and FileId
    let owner = DefId { file_id: FileId(0), local_id: hir::ids::LocalDefId(0) };
    let file_id = FileId(0);

    // Lower the program to HIR
    let body = lower_program(&program, owner, file_id, &item_tree).unwrap();

    // Verify that the array access was lowered correctly
    assert_eq!(
        body.instructions.len(),
        1,
        "Expected 1 instruction, got {}",
        body.instructions.len()
    );

    // Get the instruction
    let instruction = &body.instructions[0];
    assert_eq!(instruction.opcode, "LOAD", "Expected LOAD instruction, got {}", instruction.opcode);

    // Verify that the operand is an indirect memory reference with an array access
    let operand_id = instruction.operand.unwrap();
    let operand = &body.exprs[operand_id.0 as usize];

    // Print the operand for debugging
    println!("Operand: {:?}", operand);

    // Print all expressions for debugging
    for (i, expr) in body.exprs.iter().enumerate() {
        println!("Expr {}: {:?}", i, expr);
    }
}

#[test]
fn test_immediate_array_access_lowering() {
    // Create a simple program with immediate array access
    let source = "LOAD #2[3]\n";

    // Parse the source into an AST
    let (events, errors) = ram_parser::parse(source);
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    // Convert events to a syntax tree
    let (tree, cache) = ram_parser::build_tree(events);
    let syntax_node = ram_syntax::SyntaxNode::new_root_with_resolver(tree, cache);

    // Extract the AST program
    let program = ast::Program::cast(syntax_node).unwrap();

    // Create a dummy ItemTree
    let item_tree = ItemTree::default();

    // Create a dummy DefId and FileId
    let owner = DefId { file_id: FileId(0), local_id: hir::ids::LocalDefId(0) };
    let file_id = FileId(0);

    // Lower the program to HIR
    let body = lower_program(&program, owner, file_id, &item_tree).unwrap();

    // Verify that the array access was lowered correctly
    assert_eq!(
        body.instructions.len(),
        1,
        "Expected 1 instruction, got {}",
        body.instructions.len()
    );

    // Get the instruction
    let instruction = &body.instructions[0];
    assert_eq!(instruction.opcode, "LOAD", "Expected LOAD instruction, got {}", instruction.opcode);

    // Print all expressions for debugging
    for (i, expr) in body.exprs.iter().enumerate() {
        println!("Expr {}: {:?}", i, expr);
    }

    // Verify that the operand exists
    if let Some(operand_id) = instruction.operand {
        let operand = &body.exprs[operand_id.0 as usize];
        println!("Operand: {:?}", operand);
    } else {
        println!("No operand found");
    }
}
