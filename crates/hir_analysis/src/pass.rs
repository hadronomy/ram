//! Analysis pass trait and related types.
//!
//! This module provides the `AnalysisPass` trait, which is implemented by
//! analysis passes that can be registered with the `AnalysisPipeline`.
//!
//! # Example
//!
//! ```
//! use std::any::TypeId;
//! use hir_analysis::pass::AnalysisPass;
//! use hir_analysis::context::AnalysisContext;
//! use miette::Diagnostic;
//!
//! struct MyPass;
//!
//! impl AnalysisPass for MyPass {
//!     type Output = Vec<String>;
//!
//!     fn name(&self) -> &'static str {
//!         "MyPass"
//!     }
//!
//!     fn dependencies(&self) -> Vec<TypeId> {
//!         vec![]
//!     }
//!
//!     fn run(&self, ctx: &mut AnalysisContext) -> Result<Self::Output, Box<dyn Diagnostic>> {
//!         // Perform analysis
//!         Ok(vec!["result".to_string()])
//!     }
//! }
//! ```

use std::any::{Any, TypeId};
use std::fmt;

use miette::*;

use crate::context::AnalysisContext;

/// Trait for analysis passes that can be registered with the `AnalysisPipeline`.
///
/// Analysis passes perform semantic analysis on a HIR body and produce results
/// that can be used by other passes. Each pass declares its dependencies, which
/// are used by the `AnalysisPipeline` to determine the order in which passes
/// should be run.
///
/// # Type Parameters
///
/// * `Output` - The type of the result produced by this pass.
///
/// # Examples
///
/// ```
/// use std::any::TypeId;
/// use hir_analysis::pass::AnalysisPass;
/// use hir_analysis::context::AnalysisContext;
/// use miette::Diagnostic;
///
/// struct MyPass;
///
/// impl AnalysisPass for MyPass {
///     type Output = Vec<String>;
///
///     fn name(&self) -> &'static str {
///         "MyPass"
///     }
///
///     fn dependencies(&self) -> Vec<TypeId> {
///         vec![]
///     }
///
///     fn run(&self, ctx: &mut AnalysisContext) -> Result<Self::Output, Box<dyn Diagnostic>> {
///         // Perform analysis
///         Ok(vec!["result".to_string()])
///     }
/// }
/// ```
pub trait AnalysisPass: Any + Send + Sync {
    /// The type of the result produced by this pass.
    ///
    /// This type must implement `Any + Send + Sync` so that it can be stored
    /// in the `AnalysisContext` and accessed by other passes.
    type Output: Any + Send + Sync;

    /// Returns the name of this pass.
    ///
    /// This name is used in error messages and logging.
    ///
    /// # Returns
    ///
    /// A static string slice containing the name of this pass.
    fn name(&self) -> &'static str;

    /// Returns the dependencies of this pass.
    ///
    /// The `AnalysisPipeline` uses this information to determine the order in
    /// which passes should be run. Each pass will only be run after all of its
    /// dependencies have been run.
    ///
    /// # Returns
    ///
    /// A vector of `TypeId`s representing the types of the passes that this
    /// pass depends on.
    fn dependencies(&self) -> Vec<TypeId>;

    /// Runs this pass on the given context.
    ///
    /// This method performs the actual analysis and returns the result.
    ///
    /// # Parameters
    ///
    /// * `ctx` - The analysis context, which provides access to the HIR body
    ///   and the results of other passes.
    ///
    /// # Returns
    ///
    /// * `Ok(Self::Output)` - The result of the analysis.
    /// * `Err(Box<dyn Diagnostic>)` - An error that occurred during analysis.
    fn run(&self, ctx: &mut AnalysisContext) -> Result<Self::Output, Box<dyn Diagnostic>>;
}

/// Implements the `Debug` trait for `Box<dyn AnalysisPass>`.
///
/// This allows printing boxed analysis passes for debugging purposes.
impl<T: Any + Send + Sync> fmt::Debug for Box<dyn AnalysisPass<Output = T>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Box<dyn AnalysisPass>")
            .field("name", &self.name())
            .field("type_id", &(**self).type_id())
            .field("output_type", &std::any::type_name::<T>())
            .finish()
    }
}
