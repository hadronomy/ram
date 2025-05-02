use std::sync::Arc;

use hir_analysis::AnalysisPipeline;
use ram_parser::{AstNode, Program, SyntaxNode, build_tree, convert_errors, parse};

/// Create a parser for RAM assembly language.
///
/// This function returns a parser that can be used to parse RAM assembly code.
pub fn parser() -> impl FnOnce(&str) -> (Program, AnalysisPipeline, Vec<miette::Error>) {
    |source| parse_program(source)
}

/// Parse RAM assembly code into a syntax tree.
///
/// This function uses the recursive descent parser to parse the input string
/// and returns a syntax tree and any errors encountered during parsing.
pub fn parse_program(source: &str) -> (Program, AnalysisPipeline, Vec<miette::Error>) {
    // Parse the source text using our recursive descent parser
    let (events, errors) = parse(source);
    let mut errors = errors;

    // Convert the events into a syntax tree
    let (tree, cache) = build_tree(events);
    let syntax_node = SyntaxNode::new_root_with_resolver(tree, cache);
    let program = Program::cast(syntax_node).expect("Failed to cast root node to Program");
    // Create a file ID for this program
    let file_id = base_db::input::FileId(0);

    // Create an ItemTree for the program
    let item_tree = hir_def::item_tree::ItemTree::lower(&program, file_id);

    // Lower the program to HIR
    let body = hir::lower::lower_program(&program, hir::ids::DefId::default(), file_id, &item_tree)
        .unwrap();

    let mut pipeline = AnalysisPipeline::new();

    pipeline.register::<hir_analysis::analyzers::InstructionValidationAnalysis>().ok();
    pipeline.register::<hir_analysis::analyzers::ControlFlowAnalysis>().ok();
    pipeline.register::<hir_analysis::analyzers::DataFlowAnalysis>().ok();
    pipeline.register::<hir_analysis::analyzers::ConstantPropagationAnalysis>().ok();
    pipeline.register::<hir_analysis::analyzers::ControlFlowOptimizer>().ok();

    // Run the analysis pipeline
    match pipeline.analyze(Arc::new(body)) {
        Ok(result) => {
            // Add any diagnostics from the analysis to our errors
            errors.extend(result.diagnostics().clone());
        }
        Err(err) => {
            // If analysis fails, add a diagnostic about it
            let range = program.syntax().text_range();
            let span = range.start().into()..range.end().into();
            errors.push(ram_parser::Diagnostic::error(
                format!("Analysis failed: {}", err),
                "Check your program for semantic errors".to_string(),
                span,
            ));
        }
    }

    // Convert the errors into miette errors
    let miette_errors = if errors.is_empty() {
        // No errors, return an empty vector
        Vec::new()
    } else {
        // Convert the errors to miette errors
        let parser_error = convert_errors(source, errors);
        vec![miette::Error::new(parser_error)]
    };

    (program, pipeline, miette_errors)
}
