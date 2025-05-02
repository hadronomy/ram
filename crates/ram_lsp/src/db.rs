use std::sync::Arc;

use dashmap::DashMap;
use hir::body::Body;
use hir_analysis::analyzers::constant_propagation::ConstantPropagationAnalysis;
use hir_analysis::analyzers::control_flow_optimizer::ControlFlowOptimizer;
use hir_analysis::{
    AnalysisPipeline, ControlFlowAnalysis, DataFlowAnalysis, InstructionValidationAnalysis,
};
use ram_diagnostics::DiagnosticCollection;
use ram_parser::parse;
use ram_syntax::{AstNode, Program, ResolvedNode, SyntaxNode};
use tower_lsp::lsp_types::Url;

/// A file ID for the LSP database
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileId(pub u32);

/// LSP database for the RAM language server
#[derive(Debug, Default)]
pub struct LspDatabase {
    /// Map from FileId to file content
    files: DashMap<FileId, String>,
    /// Map from URL to FileId
    url_to_file: DashMap<Url, FileId>,
    /// Map from FileId to URL
    file_to_url: DashMap<FileId, Url>,
    /// Map from FileId to diagnostics
    diagnostics: DashMap<FileId, DiagnosticCollection>,
    /// Map from FileId to syntax tree
    syntax_trees: DashMap<FileId, ResolvedNode>,
}

#[allow(dead_code)]
impl LspDatabase {
    /// Create a new LSP database
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the file ID for a URL
    pub fn file_id_for_url(&self, url: &Url) -> Option<FileId> {
        self.url_to_file.get(url).map(|id| *id)
    }

    /// Get the URL for a file ID
    pub fn url_for_file_id(&self, file_id: FileId) -> Option<Url> {
        self.file_to_url.get(&file_id).map(|url| url.clone())
    }

    /// Get the text of a file
    pub fn file_text(&self, file_id: FileId) -> Option<String> {
        self.files.get(&file_id).map(|text| text.clone())
    }

    /// Add or update a file in the database
    pub fn add_file(&mut self, url: Url, text: &str) -> FileId {
        // Check if we already have this file
        if let Some(file_id) = self.file_id_for_url(&url) {
            // Update the file text
            self.files.insert(file_id, text.to_string());
            // Update the syntax tree and diagnostics
            self.update_file_analysis(file_id, text);
            file_id
        } else {
            // Create a new file ID
            let file_id = FileId(self.url_to_file.len() as u32);

            // Add the file to the database
            self.files.insert(file_id, text.to_string());

            // Add the URL mappings
            self.url_to_file.insert(url.clone(), file_id);
            self.file_to_url.insert(file_id, url);

            // Update the syntax tree and diagnostics
            self.update_file_analysis(file_id, text);

            file_id
        }
    }

    /// Remove a file from the database
    pub fn remove_file(&mut self, url: &Url) {
        if let Some(file_id) = self.file_id_for_url(url) {
            self.files.remove(&file_id);
            self.url_to_file.remove(url);
            self.file_to_url.remove(&file_id);
            self.diagnostics.remove(&file_id);
            self.syntax_trees.remove(&file_id);
        }
    }

    /// Update the syntax tree and diagnostics for a file
    fn update_file_analysis(&mut self, file_id: FileId, text: &str) {
        // Parse the file
        let (events, parser_diagnostics) = parse(text);

        // Build the syntax tree
        let (green_node, interner) = ram_parser::build_tree(events);
        let syntax_tree = SyntaxNode::new_root_with_resolver(green_node, interner);

        // Store the syntax tree
        self.syntax_trees.insert(file_id, syntax_tree.clone());

        // Create a diagnostic collection
        let mut diagnostic_collection = DiagnosticCollection::new();

        // Convert parser diagnostics to ram_diagnostics::Diagnostic
        for parser_diag in parser_diagnostics {
            let ram_diag = ram_diagnostics::Diagnostic {
                message: parser_diag.message,
                help: parser_diag.help,
                labeled_spans: parser_diag.labeled_spans,
                kind: match parser_diag.kind {
                    ram_parser::DiagnosticKind::Error => ram_diagnostics::DiagnosticKind::Error,
                    ram_parser::DiagnosticKind::Warning => ram_diagnostics::DiagnosticKind::Warning,
                    ram_parser::DiagnosticKind::Advice => ram_diagnostics::DiagnosticKind::Advice,
                    ram_parser::DiagnosticKind::Custom(name) => {
                        ram_diagnostics::DiagnosticKind::Custom(name)
                    }
                },
                code: parser_diag.code,
                notes: parser_diag.notes,
            };
            diagnostic_collection.add(ram_diag);
        }

        // Try to perform semantic analysis if the syntax is valid
        if !diagnostic_collection.has_errors() {
            // Convert syntax tree to AST Program
            if let Some(program) = Program::cast(syntax_tree) {
                // Create a dummy HIR body for analysis
                // In a real implementation, we would use the proper lowering logic
                let body = self.create_hir_body_from_program(&program);

                // Run HIR analysis
                let mut pipeline = AnalysisPipeline::new();

                // Register analysis passes
                pipeline.register::<InstructionValidationAnalysis>().ok();
                pipeline.register::<ControlFlowAnalysis>().ok();
                pipeline.register::<DataFlowAnalysis>().ok();
                pipeline.register::<ConstantPropagationAnalysis>().ok();
                pipeline.register::<ControlFlowOptimizer>().ok();

                // Run the analysis
                if let Ok(context) = pipeline.analyze(Arc::new(body)) {
                    // Add semantic diagnostics to our collection
                    diagnostic_collection.extend(context.diagnostics().clone());
                }
            }
        }

        // Store the diagnostics
        self.diagnostics.insert(file_id, diagnostic_collection);
    }

    /// Create a HIR body from an AST Program
    /// Uses the proper lowering logic from the hir crate
    fn create_hir_body_from_program(&self, program: &Program) -> Body {
        // Create a dummy file ID for this program
        let file_id = base_db::input::FileId(0);

        // Create a dummy DefId for the program
        let def_id = hir::ids::DefId { file_id, local_id: hir::ids::LocalDefId(0) };

        // Create an ItemTree for the program
        let item_tree = hir_def::item_tree::ItemTree::lower(program, file_id);

        // Lower the AST Program to a HIR Body
        match hir::lower::lower_program(program, def_id, file_id, &item_tree) {
            Ok(body) => body,
            Err(err) => {
                // Log the error
                tracing::error!("Failed to lower program to HIR: {:?}", err);
                // Return an empty body as fallback
                Body::default()
            }
        }
    }

    /// Get the diagnostics for a file
    pub fn diagnostics_for_file(&self, file_id: FileId) -> Option<DiagnosticCollection> {
        self.diagnostics.get(&file_id).map(|d| d.clone())
    }

    /// Get the syntax tree for a file
    pub fn syntax_tree_for_file(&self, file_id: FileId) -> Option<ResolvedNode> {
        self.syntax_trees.get(&file_id).map(|t| t.clone())
    }
}
