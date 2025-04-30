use ram_syntax::{ResolvedNode, SyntaxKind, cstree};
use tower_lsp::lsp_types::{
    SemanticToken, SemanticTokenModifier, SemanticTokenType, SemanticTokens, SemanticTokensLegend,
};

/// Get the semantic tokens legend for RAM
pub fn semantic_tokens_legend() -> SemanticTokensLegend {
    SemanticTokensLegend {
        token_types: vec![
            SemanticTokenType::KEYWORD,     // 0: Instructions
            SemanticTokenType::FUNCTION,    // 1: Labels
            SemanticTokenType::NUMBER,      // 2: Numbers
            SemanticTokenType::OPERATOR,    // 3: Operators
            SemanticTokenType::COMMENT,     // 4: Comments
            SemanticTokenType::STRING,      // 5: Strings
            SemanticTokenType::VARIABLE,    // 6: Variables
            SemanticTokenType::PARAMETER,   // 7: Parameters
            SemanticTokenType::TYPE,        // 8: Types
            SemanticTokenType::ENUM_MEMBER, // 9: Enum members
        ],
        token_modifiers: vec![
            SemanticTokenModifier::DECLARATION, // 0: Declaration
            SemanticTokenModifier::DEFINITION,  // 1: Definition
            SemanticTokenModifier::READONLY,    // 2: Readonly
            SemanticTokenModifier::STATIC,      // 3: Static
            SemanticTokenModifier::DEPRECATED,  // 4: Deprecated
        ],
    }
}

/// Get semantic tokens for a syntax tree
pub fn semantic_tokens_for_tree(syntax_tree: &ResolvedNode) -> Vec<SemanticToken> {
    // Get the text of the entire file
    let text = syntax_tree.text();
    let text_str = text.to_string();

    // Split the text into lines for position calculation
    let lines: Vec<&str> = text_str.split('\n').collect();
    let mut line_starts = Vec::with_capacity(lines.len());
    let mut current_start = 0;

    // Calculate the start offset of each line
    for line in &lines {
        line_starts.push(current_start);
        current_start += line.len() + 1; // +1 for the newline character
    }

    // Create a structure to hold token information before converting to LSP format
    #[derive(Debug)]
    struct TokenInfo {
        line: usize,
        character: usize,
        length: usize,
        token_type: usize,
    }

    // Collect all tokens first
    let mut token_infos = Vec::new();

    // Process all tokens in the tree
    for element in syntax_tree.descendants_with_tokens() {
        // We only want to process tokens in this pass
        let token = match element {
            cstree::util::NodeOrToken::Token(token) => token,
            _ => continue,
        };

        // Skip tokens we don't want to highlight
        let token_type = token_type_for_syntax_kind(token.kind());
        if token_type.is_none() {
            continue;
        }

        // Get token range
        let token_range = token.text_range();
        let token_start = usize::from(token_range.start());
        let token_end = usize::from(token_range.end());
        let token_len = token_end - token_start;

        // Find the line and character position
        let mut token_line = 0;
        let mut token_character = 0;

        for (i, start) in line_starts.iter().enumerate() {
            if *start <= token_start
                && (i == line_starts.len() - 1 || line_starts[i + 1] > token_start)
            {
                token_line = i;
                token_character = token_start - start;
                break;
            }
        }

        // Add token info
        token_infos.push(TokenInfo {
            line: token_line,
            character: token_character,
            length: token_len,
            token_type: token_type.unwrap(),
        });
    }

    // Process specific parent nodes that need highlighting but don't have tokens
    // This is for nodes like LABEL_DEF, INSTRUCTION, etc.
    for element in syntax_tree.descendants() {
        let kind = element.kind();

        // Only process specific node types that need highlighting
        // and don't have corresponding tokens
        match kind {
            SyntaxKind::LABEL_DEF
            | SyntaxKind::INSTRUCTION
            | SyntaxKind::OPERAND
            | SyntaxKind::DIRECT_OPERAND
            | SyntaxKind::INDIRECT_OPERAND
            | SyntaxKind::IMMEDIATE_OPERAND
            | SyntaxKind::OPERAND_VALUE
            | SyntaxKind::ARRAY_ACCESSOR
            | SyntaxKind::MOD_STMT
            | SyntaxKind::USE_STMT
            | SyntaxKind::MODULE_PATH => {
                // Get the token type
                let node_type = token_type_for_syntax_kind(kind);
                if node_type.is_none() {
                    continue;
                }

                // Check if this node has any identifier tokens
                let has_identifier = element.descendants_with_tokens()
                    .any(|e| matches!(e, cstree::util::NodeOrToken::Token(t) if t.kind() == SyntaxKind::IDENTIFIER));

                // Skip if it has identifiers - they'll be highlighted separately
                if has_identifier {
                    continue;
                }

                // Get node range
                let node_range = element.text_range();
                let node_start = usize::from(node_range.start());
                let node_end = usize::from(node_range.end());
                let node_len = node_end - node_start;

                // Find the line and character position
                let mut node_line = 0;
                let mut node_character = 0;

                for (i, start) in line_starts.iter().enumerate() {
                    if *start <= node_start
                        && (i == line_starts.len() - 1 || line_starts[i + 1] > node_start)
                    {
                        node_line = i;
                        node_character = node_start - start;
                        break;
                    }
                }

                // Add token info
                token_infos.push(TokenInfo {
                    line: node_line,
                    character: node_character,
                    length: node_len,
                    token_type: node_type.unwrap(),
                });
            }
            _ => continue,
        }
    }

    // Sort tokens by position
    token_infos.sort_by_key(|info| (info.line, info.character));

    // Convert to LSP semantic tokens with delta encoding
    let mut tokens = Vec::with_capacity(token_infos.len());
    let mut prev_line = 0;
    let mut prev_character = 0;

    for info in token_infos {
        let delta_line = info.line as u32 - prev_line;
        let delta_start = if delta_line == 0 {
            info.character as u32 - prev_character
        } else {
            info.character as u32
        };

        tokens.push(SemanticToken {
            delta_line,
            delta_start,
            length: info.length as u32,
            token_type: info.token_type as u32,
            token_modifiers_bitset: 0,
        });

        prev_line = info.line as u32;
        prev_character = info.character as u32;
    }

    tokens
}

/// Get the token type for a syntax kind
fn token_type_for_syntax_kind(kind: SyntaxKind) -> Option<usize> {
    match kind {
        // Instructions and keywords
        SyntaxKind::INSTRUCTION => Some(0), // KEYWORD (keyword.control.instruction.ram)
        SyntaxKind::MOD_KW => Some(0),      // KEYWORD
        SyntaxKind::USE_KW => Some(0),      // KEYWORD

        // Functions and labels
        SyntaxKind::LABEL_DEF => Some(1), // FUNCTION (entity.name.function.ram)

        // Literals
        SyntaxKind::NUMBER => Some(2), // NUMBER (constant.numeric.integer.ram)

        // Operators and punctuation
        SyntaxKind::COLON => Some(3), // OPERATOR (punctuation.definition.label.ram)
        SyntaxKind::COMMA => Some(3), // OPERATOR
        SyntaxKind::LBRACE => Some(3), // OPERATOR
        SyntaxKind::RBRACE => Some(3), // OPERATOR
        SyntaxKind::LBRACKET => Some(3), // OPERATOR (punctuation.section.brackets.begin.ram)
        SyntaxKind::RBRACKET => Some(3), // OPERATOR (punctuation.section.brackets.end.ram)
        SyntaxKind::STAR => Some(3),  // OPERATOR (keyword.operator.indirect.ram)
        SyntaxKind::EQUALS => Some(3), // OPERATOR (keyword.operator.immediate.ram)

        // Comments
        SyntaxKind::COMMENT => Some(4), // COMMENT (comment.line.number-sign.ram)
        SyntaxKind::DOC_COMMENT => Some(4), // COMMENT
        SyntaxKind::COMMENT_GROUP => Some(4), // COMMENT
        SyntaxKind::HASH => Some(4),    // COMMENT
        SyntaxKind::HASH_STAR => Some(4), // COMMENT
        SyntaxKind::COMMENT_TEXT => Some(4), // COMMENT

        // Strings
        SyntaxKind::STRING => Some(5), // STRING

        // Variables and identifiers
        SyntaxKind::IDENTIFIER => None,

        // Parameters and operands
        SyntaxKind::OPERAND => Some(7),           // PARAMETER
        SyntaxKind::DIRECT_OPERAND => Some(7),    // PARAMETER
        SyntaxKind::INDIRECT_OPERAND => Some(7),  // PARAMETER
        SyntaxKind::IMMEDIATE_OPERAND => Some(7), // PARAMETER
        SyntaxKind::OPERAND_VALUE => Some(7),     // PARAMETER
        SyntaxKind::ARRAY_ACCESSOR => Some(7),    // PARAMETER

        // Types and modules
        SyntaxKind::MODULE_PATH => Some(8), // TYPE
        SyntaxKind::MOD_STMT => Some(8),    // TYPE
        SyntaxKind::USE_STMT => Some(8),    // TYPE

        // Skip these
        SyntaxKind::WHITESPACE => None,  // Skip whitespace
        SyntaxKind::NEWLINE => None,     // Skip newlines
        SyntaxKind::ERROR => None,       // Skip errors
        SyntaxKind::ERROR_NODE => None,  // Skip error nodes
        SyntaxKind::ERROR_TOKEN => None, // Skip error tokens

        // Default for anything else
        _ => None,
    }
}

/// Convert semantic tokens to LSP semantic tokens
pub fn to_lsp_semantic_tokens(tokens: Vec<SemanticToken>) -> SemanticTokens {
    SemanticTokens { result_id: None, data: tokens }
}
