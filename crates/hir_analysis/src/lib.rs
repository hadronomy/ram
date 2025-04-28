//! Semantic analysis for RAM HIR
//!
//! This crate provides semantic analysis capabilities for the RAM High-level
//! Intermediate Representation (HIR). It includes type checking, control flow
//! analysis, data flow analysis, and other semantic validations.
//!
//! # Architecture
//!
//! The semantic analysis is built around a few key components:
//!
//! - **Database**: Salsa-based incremental computation for caching analysis results
//! - **Visitors**: Traversal of the HIR for various analysis purposes
//! - **Diagnostics**: Collection and reporting of semantic errors and warnings
//! - **Analysis Passes**: Specific analysis algorithms that operate on the HIR
//!
//! # Example
//!
//! ```rust,no_run
//! use hir_analysis::{AnalysisDatabase, Analysis, AnalysisResults};
//! use hir::ids::DefId;
//!
//! fn analyze_program(db: &dyn AnalysisDatabase, def_id: DefId) -> AnalysisResults {
//!     let analysis = Analysis::new(db);
//!     analysis.analyze_body(def_id)
//! }
//! ```

mod db;
mod types;
mod validation;
mod visitors;
mod analysis;
mod diagnostics_query;

use std::sync::Arc;

use hir::ids::DefId;
use hir::body::Body;
use ram_diagnostics::{Diagnostic, DiagnosticKind, DiagnosticCollection};

pub use db::{AnalysisDatabase, AnalysisStorage};
pub use types::{Type, TypeId, TypeSystem};
pub use validation::{ValidationRule, ValidationContext};

/// Main entry point for semantic analysis
#[derive(Debug, Clone)]
pub struct Analysis<'db> {
    db: &'db dyn AnalysisDatabase,
}

/// Results of semantic analysis
#[derive(Debug, Clone)]
pub struct AnalysisResults {
    /// Diagnostics collected during analysis
    pub diagnostics: DiagnosticCollection,

    /// Type information for expressions
    pub type_info: Arc<types::TypeInfo>,

    /// Control flow graph
    pub control_flow: Option<Arc<analysis::control_flow::ControlFlowGraph>>,

    /// Data flow analysis results
    pub data_flow: Option<Arc<analysis::data_flow::DataFlowResults>>,

    /// Optimization opportunities
    pub optimizations: Vec<analysis::optimization::Optimization>,
}

impl<'db> Analysis<'db> {
    /// Create a new analysis instance
    pub fn new(db: &'db dyn AnalysisDatabase) -> Self {
        Self { db }
    }

    /// Analyze a body and return the analysis results
    pub fn analyze_body(&self, def_id: DefId) -> AnalysisResults {
        // Get the body from the database
        let body = self.db.body(def_id);

        // Create a context for the analysis
        let mut context = AnalysisContext::new(self.db, &body);

        // Run the analysis passes
        self.run_analysis_passes(&mut context);

        // Return the results
        context.into_results()
    }

    /// Run all analysis passes on the given context
    fn run_analysis_passes(&self, context: &mut AnalysisContext) {
        // Type checking
        let type_visitor = visitors::type_check::TypeCheckVisitor::new(context);
        type_visitor.check();

        // Control flow analysis
        if context.diagnostics.has_errors() {
            // Skip further analysis if there are type errors
            return;
        }

        let cfg = analysis::control_flow::analyze(context);
        context.set_control_flow_graph(cfg);

        // Data flow analysis
        if let Some(cfg) = &context.control_flow {
            let data_flow = analysis::data_flow::analyze(context, cfg);
            context.set_data_flow_results(data_flow);
        }

        // Optimization analysis
        let optimizations = analysis::optimization::find_optimizations(context);
        context.set_optimizations(optimizations);
    }
}

/// Context for semantic analysis
#[derive(Debug)]
pub struct AnalysisContext<'db, 'body> {
    /// Database for queries
    db: &'db dyn AnalysisDatabase,

    /// Body being analyzed
    body: &'body Body,

    /// Diagnostics collected during analysis
    diagnostics: DiagnosticCollection,

    /// Type information
    type_info: types::TypeInfo,

    /// Control flow graph
    control_flow: Option<Arc<analysis::control_flow::ControlFlowGraph>>,

    /// Data flow analysis results
    data_flow: Option<Arc<analysis::data_flow::DataFlowResults>>,

    /// Optimization opportunities
    optimizations: Vec<analysis::optimization::Optimization>,
}

impl<'db, 'body> AnalysisContext<'db, 'body> {
    /// Create a new analysis context
    pub fn new(db: &'db dyn AnalysisDatabase, body: &'body Body) -> Self {
        Self {
            db,
            body,
            diagnostics: DiagnosticCollection::new(),
            type_info: types::TypeInfo::new(),
            control_flow: None,
            data_flow: None,
            optimizations: Vec::new(),
        }
    }

    /// Get the body being analyzed
    pub fn body(&self) -> &Body {
        self.body
    }

    /// Get the database
    pub fn db(&self) -> &dyn AnalysisDatabase {
        self.db
    }

    /// Get the diagnostics collection
    pub fn diagnostics(&self) -> &DiagnosticCollection {
        &self.diagnostics
    }

    /// Get mutable access to the diagnostics collection
    pub fn diagnostics_mut(&mut self) -> &mut DiagnosticCollection {
        &mut self.diagnostics
    }

    /// Get the type information
    pub fn type_info(&self) -> &types::TypeInfo {
        &self.type_info
    }

    /// Get mutable access to the type information
    pub fn type_info_mut(&mut self) -> &mut types::TypeInfo {
        &mut self.type_info
    }

    /// Set the control flow graph
    pub fn set_control_flow_graph(&mut self, cfg: Arc<analysis::control_flow::ControlFlowGraph>) {
        self.control_flow = Some(cfg);
    }

    /// Set the data flow analysis results
    pub fn set_data_flow_results(&mut self, data_flow: Arc<analysis::data_flow::DataFlowResults>) {
        self.data_flow = Some(data_flow);
    }

    /// Set the optimization opportunities
    pub fn set_optimizations(&mut self, optimizations: Vec<analysis::optimization::Optimization>) {
        self.optimizations = optimizations;
    }

    /// Convert the context into analysis results
    pub fn into_results(self) -> AnalysisResults {
        AnalysisResults {
            diagnostics: self.diagnostics,
            type_info: Arc::new(self.type_info),
            control_flow: self.control_flow,
            data_flow: self.data_flow,
            optimizations: self.optimizations,
        }
    }
}
