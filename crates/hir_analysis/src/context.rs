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
//! let body = Arc::new(Body::new(/* ... */));
//! let mut context = AnalysisContext::new(body);
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
use tracing::{debug, error, instrument};

use crate::error::AnalysisError;
use crate::pass::AnalysisPass;

/// Context for storing and retrieving analysis results.
///
/// The [`AnalysisContext`] is passed to each analysis pass and allows passes to
/// access the results of their dependencies. It also provides access to the
/// HIR body being analyzed.
pub struct AnalysisContext {
    /// The HIR body being analyzed.
    body: Arc<hir::body::Body>,
    /// Map from pass TypeId to analysis results.
    results: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
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
        AnalysisContext { body, results: HashMap::new() }
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
            .finish()
    }
}
