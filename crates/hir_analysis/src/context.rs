//! Analysis context for storing and retrieving analysis results.
//!
//! This module provides the [`AnalysisContext`] which is used to store and retrieve
//! the results of analysis passes. It is passed to each analysis pass and allows
//! passes to access the results of their dependencies.
//!
//! # Example
//!
//! ```
//! use hir_analysis::context::AnalysisContext;
//! use hir_analysis::pass::AnalysisPass;
//! use hir::body::Body;
//! use std::sync::Arc;
//!
//! // Create a new context
//! let body = Arc::new(Body::default());
//! // Note: AnalysisContext::new is pub(crate), so we'd normally use the pipeline
//! // let context = AnalysisContext::from(Body::default());
//!
//! // Insert a result (typically done by the AnalysisPipeline)
//! // context.insert_result::<MyPass>(my_result).unwrap();
//!
//! // Get a result
//! // let result = context.get_result::<MyPass>().unwrap();
//! ```

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use hir::body::Body;
use miette::*;
use ram_diagnostics::{Diagnostic, DiagnosticCollection};
use tracing::{debug, error, instrument};

use crate::error::AnalysisError;
use crate::pass::AnalysisPass;

/// Context for storing and retrieving analysis results.
///
/// The [`AnalysisContext`] is passed to each analysis pass and allows passes to
/// access the results of their dependencies. It also provides access to the
/// HIR body being analyzed. It also collects diagnostics reported by analysis passes.
pub struct AnalysisContext {
    /// The HIR body being analyzed.
    body: Arc<hir::body::Body>,
    /// Map from pass TypeId to analysis results.
    results: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    /// Collection of diagnostics reported by analysis passes.
    diagnostics: DiagnosticCollection,
}

impl AnalysisContext {
    /// Creates a new [`AnalysisContext`] for the given HIR body.
    ///
    /// # Parameters
    ///
    /// * `body` - The HIR body to analyze.
    #[instrument(skip(body))]
    pub(crate) fn new(body: Arc<Body>) -> Self {
        debug!("Creating new AnalysisContext");
        AnalysisContext { body, results: HashMap::new(), diagnostics: DiagnosticCollection::new() }
    }

    /// Add a diagnostic to the context.
    ///
    /// This method allows analysis passes to report diagnostics during analysis.
    ///
    /// # Parameters
    ///
    /// * `diagnostic` - The diagnostic to add.
    #[instrument(skip(self, diagnostic))]
    pub fn add_diagnostic(&mut self, diagnostic: Diagnostic) {
        debug!("Adding diagnostic: {:?}", diagnostic);
        self.diagnostics.add(diagnostic);
    }

    /// Add an error diagnostic to the context.
    ///
    /// This is a convenience method for adding an error diagnostic.
    ///
    /// # Parameters
    ///
    /// * `message` - The error message.
    /// * `help` - Additional help text.
    /// * `span` - The source span for the error, if available.
    /// * `node_id` - The ID of the HIR node that caused the error, if available.
    #[instrument(skip(self, message, help))]
    pub fn error(
        &mut self,
        message: impl Into<String>,
        help: impl Into<String>,
        span: Option<std::ops::Range<usize>>,
    ) {
        debug!("Adding error diagnostic");
        self.diagnostics.error(message, help, span);
    }

    /// Add an error diagnostic to the context using an instruction ID.
    ///
    /// This method extracts the span from the instruction and uses it for the diagnostic.
    ///
    /// # Parameters
    ///
    /// * `message` - The error message.
    /// * `help` - Additional help text.
    /// * `instr_id` - The ID of the instruction that caused the error.
    #[instrument(skip(self, message, help))]
    pub fn error_at_instruction(
        &mut self,
        message: impl Into<String>,
        help: impl Into<String>,
        instr_id: hir::ids::LocalDefId,
    ) {
        debug!("Adding error diagnostic at instruction");
        let span = self.get_instruction_span(instr_id);
        self.diagnostics.error(message, help, Some(span));
    }

    /// Add an error diagnostic to the context using an expression ID.
    ///
    /// This method extracts the span from the expression and uses it for the diagnostic.
    ///
    /// # Parameters
    ///
    /// * `message` - The error message.
    /// * `help` - Additional help text.
    /// * `expr_id` - The ID of the expression that caused the error.
    #[instrument(skip(self, message, help))]
    pub fn error_at_expr(
        &mut self,
        message: impl Into<String>,
        help: impl Into<String>,
        expr_id: hir::expr::ExprId,
    ) {
        debug!("Adding error diagnostic at expression");
        let span = self.get_expr_span(expr_id);
        self.diagnostics.error(message, help, Some(span));
    }

    /// Add an error diagnostic to the context using a label ID.
    ///
    /// This method extracts the span from the label and uses it for the diagnostic.
    ///
    /// # Parameters
    ///
    /// * `message` - The error message.
    /// * `help` - Additional help text.
    /// * `label_id` - The ID of the label that caused the error.
    #[instrument(skip(self, message, help))]
    pub fn error_at_label(
        &mut self,
        message: impl Into<String>,
        help: impl Into<String>,
        label_id: hir::ids::LocalDefId,
    ) {
        debug!("Adding error diagnostic at label");
        let span = self.get_label_span(label_id);
        self.diagnostics.error(message, help, Some(span));
    }

    /// Add a warning diagnostic to the context.
    ///
    /// This is a convenience method for adding a warning diagnostic.
    ///
    /// # Parameters
    ///
    /// * `message` - The warning message.
    /// * `help` - Additional help text.
    /// * `span` - The source span for the warning, if available.
    #[instrument(skip(self, message, help))]
    pub fn warning(
        &mut self,
        message: impl Into<String>,
        help: impl Into<String>,
        span: Option<std::ops::Range<usize>>,
    ) {
        debug!("Adding warning diagnostic");
        self.diagnostics.warning(message, help, span);
    }

    /// Add a warning diagnostic to the context using an instruction ID.
    ///
    /// This method extracts the span from the instruction and uses it for the diagnostic.
    ///
    /// # Parameters
    ///
    /// * `message` - The warning message.
    /// * `help` - Additional help text.
    /// * `instr_id` - The ID of the instruction that caused the warning.
    #[instrument(skip(self, message, help))]
    pub fn warning_at_instruction(
        &mut self,
        message: impl Into<String>,
        help: impl Into<String>,
        instr_id: hir::ids::LocalDefId,
    ) {
        debug!("Adding warning diagnostic at instruction");
        let span = self.get_instruction_span(instr_id);
        self.diagnostics.warning(message, help, Some(span));
    }

    /// Add a warning diagnostic to the context using an expression ID.
    ///
    /// This method extracts the span from the expression and uses it for the diagnostic.
    ///
    /// # Parameters
    ///
    /// * `message` - The warning message.
    /// * `help` - Additional help text.
    /// * `expr_id` - The ID of the expression that caused the warning.
    #[instrument(skip(self, message, help))]
    pub fn warning_at_expr(
        &mut self,
        message: impl Into<String>,
        help: impl Into<String>,
        expr_id: hir::expr::ExprId,
    ) {
        debug!("Adding warning diagnostic at expression");
        let span = self.get_expr_span(expr_id);
        self.diagnostics.warning(message, help, Some(span));
    }

    /// Add a warning diagnostic to the context using a label ID.
    ///
    /// This method extracts the span from the label and uses it for the diagnostic.
    ///
    /// # Parameters
    ///
    /// * `message` - The warning message.
    /// * `help` - Additional help text.
    /// * `label_id` - The ID of the label that caused the warning.
    #[instrument(skip(self, message, help))]
    pub fn warning_at_label(
        &mut self,
        message: impl Into<String>,
        help: impl Into<String>,
        label_id: hir::ids::LocalDefId,
    ) {
        debug!("Adding warning diagnostic at label");
        let span = self.get_label_span(label_id);
        self.diagnostics.warning(message, help, Some(span));
    }

    /// Add an info diagnostic to the context.
    ///
    /// This is a convenience method for adding an info diagnostic.
    ///
    /// # Parameters
    ///
    /// * `message` - The info message.
    /// * `help` - Additional help text.
    /// * `span` - The source span for the info, if available.
    #[instrument(skip(self, message, help))]
    pub fn info(
        &mut self,
        message: impl Into<String>,
        help: impl Into<String>,
        span: Option<std::ops::Range<usize>>,
    ) {
        debug!("Adding info diagnostic");
        self.diagnostics.info(message, help, span);
    }

    /// Add an info diagnostic to the context using an instruction ID.
    ///
    /// This method extracts the span from the instruction and uses it for the diagnostic.
    ///
    /// # Parameters
    ///
    /// * `message` - The info message.
    /// * `help` - Additional help text.
    /// * `instr_id` - The ID of the instruction that the info is about.
    #[instrument(skip(self, message, help))]
    pub fn info_at_instruction(
        &mut self,
        message: impl Into<String>,
        help: impl Into<String>,
        instr_id: hir::ids::LocalDefId,
    ) {
        debug!("Adding info diagnostic at instruction");
        let span = self.get_instruction_span(instr_id);
        self.diagnostics.info(message, help, Some(span));
    }

    /// Add an info diagnostic to the context using an expression ID.
    ///
    /// This method extracts the span from the expression and uses it for the diagnostic.
    ///
    /// # Parameters
    ///
    /// * `message` - The info message.
    /// * `help` - Additional help text.
    /// * `expr_id` - The ID of the expression that the info is about.
    #[instrument(skip(self, message, help))]
    pub fn info_at_expr(
        &mut self,
        message: impl Into<String>,
        help: impl Into<String>,
        expr_id: hir::expr::ExprId,
    ) {
        debug!("Adding info diagnostic at expression");
        let span = self.get_expr_span(expr_id);
        self.diagnostics.info(message, help, Some(span));
    }

    /// Add an info diagnostic to the context using a label ID.
    ///
    /// This method extracts the span from the label and uses it for the diagnostic.
    ///
    /// # Parameters
    ///
    /// * `message` - The info message.
    /// * `help` - Additional help text.
    /// * `label_id` - The ID of the label that the info is about.
    #[instrument(skip(self, message, help))]
    pub fn info_at_label(
        &mut self,
        message: impl Into<String>,
        help: impl Into<String>,
        label_id: hir::ids::LocalDefId,
    ) {
        debug!("Adding info diagnostic at label");
        let span = self.get_label_span(label_id);
        self.diagnostics.info(message, help, Some(span));
    }

    /// Get all diagnostics collected during analysis.
    ///
    /// # Returns
    ///
    /// A reference to the diagnostic collection.
    #[instrument(skip(self))]
    pub fn diagnostics(&self) -> &DiagnosticCollection {
        debug!("Accessing diagnostics from AnalysisContext");
        &self.diagnostics
    }

    /// Check if there are any error diagnostics.
    ///
    /// # Returns
    ///
    /// `true` if there are any error diagnostics, `false` otherwise.
    #[instrument(skip(self))]
    pub fn has_errors(&self) -> bool {
        debug!("Checking for errors in AnalysisContext");
        self.diagnostics.has_errors()
    }

    /// Returns a reference to the HIR body being analyzed.
    ///
    /// # Returns
    ///
    /// A reference to the HIR body.
    #[instrument(skip(self))]
    pub fn body(&self) -> &Arc<Body> {
        debug!("Accessing body from AnalysisContext");
        &self.body
    }

    /// Get the span for an instruction.
    ///
    /// # Parameters
    ///
    /// * `instr_id` - The ID of the instruction.
    ///
    /// # Returns
    ///
    /// The span of the instruction, or an empty span if the instruction is not found.
    #[instrument(skip(self))]
    pub fn get_instruction_span(&self, instr_id: hir::ids::LocalDefId) -> std::ops::Range<usize> {
        debug!("Getting instruction span");
        for instr in &self.body.instructions {
            if instr.id == instr_id {
                return instr.span.clone();
            }
        }
        // Return an empty span if the instruction is not found
        0..0
    }

    /// Get the span for an expression.
    ///
    /// # Parameters
    ///
    /// * `expr_id` - The ID of the expression.
    ///
    /// # Returns
    ///
    /// The span of the expression, or an empty span if the expression is not found.
    #[instrument(skip(self))]
    pub fn get_expr_span(&self, expr_id: hir::expr::ExprId) -> std::ops::Range<usize> {
        debug!("Getting expression span");
        for expr in &self.body.exprs {
            if expr.id == expr_id {
                return expr.span.clone();
            }
        }
        // Return an empty span if the expression is not found
        0..0
    }

    /// Get the span for a label.
    ///
    /// # Parameters
    ///
    /// * `label_id` - The ID of the label.
    ///
    /// # Returns
    ///
    /// The span of the label, or an empty span if the label is not found.
    #[instrument(skip(self))]
    pub fn get_label_span(&self, label_id: hir::ids::LocalDefId) -> std::ops::Range<usize> {
        debug!("Getting label span");
        for label in &self.body.labels {
            if label.id == label_id {
                return label.span.clone();
            }
        }
        // Return an empty span if the label is not found
        0..0
    }

    /// Store the result of an analysis pass.
    ///
    /// This method is used to store the result of an analysis pass in the context.
    /// It is typically called by the analysis manager, but can also be called
    /// directly in tests.
    ///
    /// # Type Parameters
    ///
    /// * `P` - The type of the analysis pass.
    ///
    /// # Parameters
    ///
    /// * `result` - The result of the analysis pass.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // let mut context = /* ... */;
    /// // let result = /* ... */;
    /// // context.store_result::<MyPass>(result);
    /// ```
    #[instrument(skip(self, result), fields(pass_type = std::any::type_name::<P>()))]
    pub fn store_result<P>(&mut self, result: P::Output)
    where
        P: AnalysisPass + 'static,
    {
        debug!("Storing result for pass");
        let pass_id = TypeId::of::<P>();
        let result = Arc::new(result);
        self.results.insert(pass_id, Box::new(result));
    }

    /// Gets the result of an analysis pass.
    ///
    /// # Type Parameters
    ///
    /// * `P` - The type of the analysis pass whose result to get.
    ///
    /// # Returns
    ///
    /// * `Ok(Arc<P::Output>)` - The result of the analysis pass.
    /// * `Err(AnalysisError::ResultNotAvailable)` - If the pass has not been run.
    /// * `Err(AnalysisError::DowncastError)` - If the result could not be downcast to the expected type.
    ///
    /// # Examples
    ///
    /// ```
    /// use hir_analysis::context::AnalysisContext;
    /// // Assuming MyPass is an AnalysisPass
    /// // let context = /* ... */;
    /// // let result = context.get_result::<MyPass>().unwrap();
    /// ```
    #[instrument(skip(self), fields(pass_type = std::any::type_name::<P>()))]
    pub fn get_result<P>(&self) -> Result<Arc<P::Output>, AnalysisError>
    where
        P: AnalysisPass + 'static,
    {
        let pass_id = TypeId::of::<P>();
        let pass_name = std::any::type_name::<P>();
        debug!(?pass_id, pass_name, "Getting result for pass");

        match self.results.get(&pass_id) {
            Some(result) => {
                debug!("Result found, attempting to downcast");
                let downcasted = result.downcast_ref::<Arc<P::Output>>().ok_or_else(|| {
                    error!("Failed to downcast result");
                    AnalysisError::DowncastError { pass_name: pass_name.to_string(), pass_id }
                })?;
                debug!("Result successfully downcast");
                Ok(Arc::clone(downcasted))
            }
            None => {
                error!("Result not available for pass");
                Err(AnalysisError::ResultNotAvailable { pass_name: pass_name.to_string(), pass_id })
            }
        }
    }

    /// Inserts the result of an analysis pass into the context.
    ///
    /// This is typically called by the `AnalysisPipeline` after running a pass.
    ///
    /// # Type Parameters
    ///
    /// * `P` - The type of the analysis pass whose result to insert.
    ///
    /// # Parameters
    ///
    /// * `result` - The result of the analysis pass.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the result was inserted successfully.
    /// * `Err(AnalysisError)` - If an error occurred.
    #[instrument(skip(self, result), fields(pass_type = std::any::type_name::<P>()))]
    pub(crate) fn insert_result<P>(&mut self, result: P::Output) -> Result<(), AnalysisError>
    where
        P: AnalysisPass + 'static,
    {
        let pass_id = TypeId::of::<P>();
        debug!(?pass_id, "Inserting result for pass");

        self.results.insert(pass_id, Box::new(Arc::new(result)));
        debug!("Result inserted successfully");
        Ok(())
    }
}

/// Implements the [`From<Body>`] trait for [`AnalysisContext`].
///
/// This allows creating an [`AnalysisContext`] directly from a [`hir::body::Body`].
///
/// # Examples
///
/// ```
/// use hir_analysis::context::AnalysisContext;
/// use hir::body::Body;
///
/// // let body = Body::new(/* ... */);
/// // let context = AnalysisContext::from(body);
/// ```
impl From<hir::body::Body> for AnalysisContext {
    #[instrument(skip(body))]
    fn from(body: hir::body::Body) -> Self {
        debug!("Creating AnalysisContext from Body");
        AnalysisContext::new(Arc::new(body))
    }
}

/// Implements the `Debug` trait for [`AnalysisContext`].
///
/// This allows printing the context for debugging purposes.
impl fmt::Debug for AnalysisContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AnalysisContext")
            .field("body", &self.body)
            .field("results", &format!("{} results", self.results.len()))
            .field("diagnostics", &format!("{} diagnostics", self.diagnostics.len()))
            .finish()
    }
}
