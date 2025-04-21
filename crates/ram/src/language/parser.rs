use ram_parser::{AstNode, Program, SyntaxNode, build_tree, convert_errors, parse};

/// Create a parser for RAM assembly language.
///
/// This function returns a parser that can be used to parse RAM assembly code.
pub fn parser() -> impl FnOnce(&str) -> (Program, Vec<miette::Error>) {
    |source| parse_program(source)
}

/// Parse RAM assembly code into a syntax tree.
///
/// This function uses the recursive descent parser to parse the input string
/// and returns a syntax tree and any errors encountered during parsing.
pub fn parse_program(source: &str) -> (Program, Vec<miette::Error>) {
    // Parse the source text using our recursive descent parser
    let (events, errors) = parse(source);

    // Convert the events into a syntax tree
    let (tree, cache) = build_tree(events);
    let syntax_node = SyntaxNode::new_root_with_resolver(tree, cache);
    let program = Program::cast(syntax_node).expect("Failed to cast root node to Program");

    // Convert the errors into miette errors
    let miette_errors = if errors.is_empty() {
        // No errors, return an empty vector
        Vec::new()
    } else {
        // Convert the errors to miette errors
        let parser_error = convert_errors(source, errors);
        vec![miette::Error::new(parser_error)]
    };

    (program, miette_errors)
}
