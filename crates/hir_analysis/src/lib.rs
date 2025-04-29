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

mod analysis;
mod db;
mod diagnostics_query;
mod types;
mod validation;
mod visitors;

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

pub use db::AnalysisDatabase;
use hir::body::Body;
use hir::ids::DefId;
use ram_diagnostics::DiagnosticCollection;
pub use types::{Type, TypeId as TypeIdAlias, TypeSystem};
pub use validation::{ValidationContext, ValidationRule};

/// Main entry point for semantic analysis
#[derive(Debug, Clone)]
pub struct Analysis<'db> {
    db: &'db dyn AnalysisDatabase,
    manager: analysis::manager::AnalysisManager,
}

impl<'db> Analysis<'db> {
    /// Create a new analysis instance with default passes
    pub fn new(db: &'db dyn AnalysisDatabase) -> Self {
        let mut analysis = Self { db, manager: analysis::manager::AnalysisManager::new() };

        // Register default pass types
        analysis.register_default_passes();

        analysis
    }

    /// Create a new analysis instance without any passes
    pub fn new_empty(db: &'db dyn AnalysisDatabase) -> Self {
        Self { db, manager: analysis::manager::AnalysisManager::new() }
    }

    /// Register default analysis passes
    fn register_default_passes(&mut self) {
        // Register pass types directly
        self.manager.register_pass_type::<analysis::control_flow::ControlFlowAnalysis>();
        self.manager.register_pass_type::<analysis::data_flow::DataFlowAnalysis>();
        self.manager.register_pass_type::<analysis::optimization::OptimizationAnalysis>();
    }

    /// Add an analysis pass type
    pub fn add_pass_type<P: analysis::AnalysisPass + Default + 'static>(&mut self) -> &mut Self {
        self.manager.register_pass::<P>();
        self
    }

    /// Add an analysis pass instance
    pub fn add_pass<P: analysis::AnalysisPass + 'static>(&mut self, pass: P) -> &mut Self {
        self.manager.register_pass_instance(pass);
        self
    }

    /// Add an analysis pass with a pre-computed result
    ///
    /// This method adds a pass to the manager and immediately stores its result.
    /// This is useful for passes that don't need to be run, but whose results should be available.
    pub fn add_pass_with_result<P: analysis::AnalysisPass + 'static>(
        &mut self,
        pass: P,
        result: P::Output,
    ) -> &mut Self {
        self.manager.register_pass_with_result(pass, result);
        self
    }

    /// Analyze a body and return the analysis results
    pub fn analyze_body(&self, def_id: DefId) -> AnalysisResults {
        // Get the body from the database
        let body = self.db.body(def_id);

        // Create a context for the analysis
        let mut context = AnalysisContext::new(self.db, &body);

        // Create a new manager for this analysis
        // We need a new instance because we can't mutate self.manager
        let mut manager = analysis::manager::AnalysisManager::new();

        // Register the default passes
        manager.register_pass::<analysis::control_flow::ControlFlowAnalysis>();
        manager.register_pass::<analysis::data_flow::DataFlowAnalysis>();
        manager.register_pass::<analysis::optimization::OptimizationAnalysis>();

        // Run all passes
        if let Err(err) = manager.run_all_passes(&mut context) {
            // If there's an error, log it and continue with what we have
            context
                .diagnostics_mut()
                .error(format!("Error running analysis passes: {}", err), None);
        }

        // Convert the context to results
        let mut results = context.into_results();

        // Transfer all results from the manager to the results
        manager.transfer_all_results_to(&mut results);

        results
    }
}

/// Context for semantic analysis
///
/// This struct holds all the data needed for semantic analysis of a HIR body.
/// It's designed to be extensible through the use of traits and component storage.
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

    /// Custom extensions that can be attached to the context
    extensions: HashMap<TypeId, Box<dyn Any>>,
}

impl<'db, 'body> AnalysisContext<'db, 'body> {
    /// Create a new analysis context
    pub fn new(db: &'db dyn AnalysisDatabase, body: &'body Body) -> Self {
        Self {
            db,
            body,
            diagnostics: DiagnosticCollection::new(),
            type_info: types::TypeInfo::new(),
            extensions: HashMap::new(),
        }
    }

    /// Create a new context builder
    pub fn builder(
        db: &'db dyn AnalysisDatabase,
        body: &'body Body,
    ) -> AnalysisContextBuilder<'db, 'body> {
        AnalysisContextBuilder::new(db, body)
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

    /// Add an extension to the context
    pub fn add_extension<T: 'static>(&mut self, extension: T) {
        let type_id = TypeId::of::<T>();
        self.extensions.insert(type_id, Box::new(extension));
    }

    /// Get an extension from the context
    pub fn get_extension<T: 'static>(&self) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.extensions.get(&type_id).and_then(|c| c.downcast_ref())
    }

    /// Get a mutable extension from the context
    pub fn get_extension_mut<T: 'static>(&mut self) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.extensions.get_mut(&type_id).and_then(|c| c.downcast_mut())
    }

    /// Convert the context into analysis results
    pub fn into_results(self) -> AnalysisResults {
        // Create a new results object
        let mut results = AnalysisResults::new();

        // Set the diagnostics and type info
        results.diagnostics = self.diagnostics;
        results.type_info = Arc::new(self.type_info);

        // Add any extensions to the results
        for (type_id, extension) in self.extensions {
            // Store the extension by its type ID
            results.insert_boxed(type_id, extension);
        }

        results
    }
}

/// Builder for AnalysisContext
///
/// Provides a fluent interface for building an AnalysisContext with different components.
pub struct AnalysisContextBuilder<'db, 'body> {
    db: &'db dyn AnalysisDatabase,
    body: &'body Body,
    diagnostics: Option<DiagnosticCollection>,
    extensions: HashMap<TypeId, Box<dyn Any>>,
}

impl<'db, 'body> AnalysisContextBuilder<'db, 'body> {
    /// Create a new context builder
    pub fn new(db: &'db dyn AnalysisDatabase, body: &'body Body) -> Self {
        Self { db, body, diagnostics: None, extensions: HashMap::new() }
    }

    /// Set diagnostics collection
    pub fn with_diagnostics(mut self, diagnostics: DiagnosticCollection) -> Self {
        self.diagnostics = Some(diagnostics);
        self
    }

    /// Add an extension to the context
    pub fn with_extension<T: 'static>(mut self, extension: T) -> Self {
        let type_id = TypeId::of::<T>();
        self.extensions.insert(type_id, Box::new(extension));
        self
    }

    /// Build the analysis context
    pub fn build(self) -> AnalysisContext<'db, 'body> {
        let mut context = AnalysisContext::new(self.db, self.body);

        // Set diagnostics if provided
        if let Some(diagnostics) = self.diagnostics {
            context.diagnostics = diagnostics;
        }

        // Add all extensions
        context.extensions = self.extensions;

        context
    }
}

/// Results of semantic analysis
///
/// This struct holds the results of semantic analysis on a HIR body.
/// It provides access to diagnostics, type information, and the results
/// of various analysis passes.
#[derive(Debug, Clone)]
pub struct AnalysisResults {
    /// Diagnostics collected during analysis
    pub diagnostics: DiagnosticCollection,

    /// Type information
    pub type_info: Arc<types::TypeInfo>,

    /// Map from type ID to boxed result
    results: HashMap<TypeId, Box<dyn Any>>,

    /// Map from pass ID to boxed result
    pass_results: HashMap<TypeId, Box<dyn Any>>,
}

impl AnalysisResults {
    /// Create a new empty results object
    pub fn new() -> Self {
        Self {
            diagnostics: DiagnosticCollection::new(),
            type_info: Arc::new(types::TypeInfo::new()),
            results: HashMap::new(),
            pass_results: HashMap::new(),
        }
    }

    /// Check if the analysis found any errors
    pub fn has_errors(&self) -> bool {
        self.diagnostics.has_errors()
    }

    /// Get the number of errors found during analysis
    pub fn error_count(&self) -> usize {
        self.diagnostics.error_count()
    }

    /// Get the number of warnings found during analysis
    pub fn warning_count(&self) -> usize {
        self.diagnostics.warning_count()
    }

    /// Check if the body is valid (no errors)
    pub fn is_valid(&self) -> bool {
        !self.diagnostics.has_errors()
    }

    /// Get a reference to the diagnostics
    pub fn diagnostics(&self) -> &DiagnosticCollection {
        &self.diagnostics
    }

    /// Get a reference to the type information
    pub fn type_info(&self) -> &types::TypeInfo {
        &self.type_info
    }

    /// Get a reference to the control flow graph
    pub fn control_flow(&self) -> Option<&Arc<analysis::control_flow::ControlFlowGraph>> {
        self.get::<Arc<analysis::control_flow::ControlFlowGraph>>()
    }

    /// Get a reference to the data flow results
    pub fn data_flow(&self) -> Option<&Arc<analysis::data_flow::DataFlowResults>> {
        self.get::<Arc<analysis::data_flow::DataFlowResults>>()
    }

    /// Get a reference to the optimization opportunities
    pub fn optimizations(&self) -> Option<&Vec<analysis::optimization::Optimization>> {
        self.get::<Vec<analysis::optimization::Optimization>>()
    }

    /// Insert a result into the cache
    ///
    /// This method stores a result in the cache, keyed by its type.
    /// If a result of the same type already exists, it will be replaced.
    pub fn insert<T: Any>(&mut self, result: T) {
        let type_id = TypeId::of::<T>();
        self.results.insert(type_id, Box::new(result));
    }

    /// Insert a boxed result into the cache by pass ID
    ///
    /// This method stores a boxed result in the cache, keyed by the pass ID.
    /// If a result for the same pass already exists, it will be replaced.
    pub fn insert_boxed(&mut self, pass_id: TypeId, result: Box<dyn Any>) {
        self.pass_results.insert(pass_id, result);
    }

    /// Get a result from the cache by type
    ///
    /// This method retrieves a result from the cache by its type.
    /// If no result of the given type exists, it returns None.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    /// use hir_analysis::analysis::control_flow::ControlFlowGraph;
    ///
    /// let results = /* ... */;
    /// if let Some(cfg) = results.get::<Arc<ControlFlowGraph>>() {
    ///     // Use the control flow graph
    /// }
    /// ```
    pub fn get<T: std::any::Any>(&self) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.results.get(&type_id).and_then(|result| result.downcast_ref())
    }

    /// Check if a result of the given type exists
    ///
    /// This method checks if a result of the given type exists in the cache.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    /// use hir_analysis::analysis::control_flow::ControlFlowGraph;
    ///
    /// let results = /* ... */;
    /// if results.contains::<Arc<ControlFlowGraph>>() {
    ///     // The control flow graph exists
    /// }
    /// ```
    pub fn contains<T: std::any::Any>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        self.results.contains_key(&type_id)
    }

    /// Get a result from the cache by pass type
    ///
    /// This method retrieves a result from the cache by the type of the pass that produced it.
    /// If no result for the given pass type exists, it returns None.
    ///
    /// # Examples
    ///
    /// ```
    /// use hir_analysis::analysis::control_flow::ControlFlowAnalysis;
    ///
    /// let results = /* ... */;
    /// if let Some(cfg) = results.get_by_pass::<ControlFlowAnalysis>() {
    ///     // Use the control flow graph
    /// }
    /// ```
    pub fn get_by_pass<P: analysis::AnalysisPass + 'static>(&self) -> Option<&P::Output> {
        let pass_id = TypeId::of::<P>();
        self.get_boxed(&pass_id).and_then(|boxed| boxed.downcast_ref::<P::Output>())
    }

    /// Get a boxed result from the cache by pass ID
    ///
    /// This method retrieves a boxed result from the cache by the pass ID.
    /// If no result for the given pass ID exists, it returns None.
    pub fn get_boxed(&self, pass_id: &TypeId) -> Option<&Box<dyn std::any::Any>> {
        self.pass_results.get(pass_id)
    }

    /// Check if a result for the given pass ID exists
    ///
    /// This method checks if a result for the given pass ID exists in the cache.
    pub fn contains_boxed(&self, pass_id: &TypeId) -> bool {
        self.pass_results.contains_key(pass_id)
    }

    /// Clear all results
    ///
    /// This method removes all results from the cache.
    pub fn clear(&mut self) {
        self.results.clear();
        self.pass_results.clear();
    }
}

impl Default for AnalysisResults {
    fn default() -> Self {
        Self::new()
    }
}
