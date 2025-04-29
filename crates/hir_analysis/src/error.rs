//! Error types for the HIR analysis.
//!
//! This module provides error types that are used by the HIR analysis
//! to report errors that occur during analysis.
//!
//! # Examples
//!
//! ```
//! use hir_analysis::error::AnalysisError;
//! use std::any::TypeId;
//!
//! let error = AnalysisError::PassNotRegistered {
//!     dependency_name: "MyPass".to_string(),
//!     dependency_id: TypeId::of::<()>(), // Just an example
//! };
//! ```

use std::any::TypeId;

use miette::Diagnostic;
use thiserror::Error;

/// Errors that can occur during HIR analysis.
///
/// This enum represents the various errors that can occur during HIR analysis,
/// including errors related to pass registration, execution, and result retrieval.
#[derive(Error, Debug, Diagnostic)]
pub enum AnalysisError {
    /// An analysis pass failed during execution.
    ///
    /// This error occurs when an analysis pass returns an error from its `run` method.
    #[diagnostic(
        code(analysis::pass_failed),
        help("The analysis pass '{pass_name}' encountered an error during execution.")
    )]
    #[error("Pass '{pass_name}' failed")]
    PassFailed {
        /// The name of the pass that failed.
        pass_name: String,
        /// The error that caused the pass to fail.
        #[source]
        source: Box<dyn miette::Diagnostic>,
    },

    /// A dependency cycle was detected in the analysis passes.
    ///
    /// This error occurs when the dependency graph of analysis passes contains a cycle,
    /// which would make it impossible to determine a valid execution order.
    #[diagnostic(
        code(analysis::dependency_cycle),
        help(
            "Analysis passes cannot have circular dependencies. Check the dependencies declared by the passes involved."
        )
    )]
    #[error("Dependency cycle detected in analysis passes: {0}")]
    DependencyCycle(
        /// A description of the cycle, including the passes involved.
        String,
    ), // Keep petgraph's cycle details

    /// A required pass was not registered.
    ///
    /// This error occurs when a pass declares a dependency on another pass
    /// that has not been registered with the `AnalysisPipeline`.
    #[diagnostic(
        code(analysis::pass_not_registered),
        help(
            "Ensure the pass '{dependency_name}' (TypeId: {dependency_id:?}) is registered with the AnalysisHost before passes that depend on it."
        )
    )]
    #[error("Required pass '{dependency_name}' (TypeId: {dependency_id:?}) was not registered")]
    PassNotRegistered {
        /// The name of the dependency that was not registered.
        dependency_name: String,
        /// The type ID of the dependency.
        dependency_id: TypeId,
    },

    /// A result was not available in the analysis context.
    ///
    /// This error occurs when a pass tries to access the result of another pass
    /// that has not been run or whose result was not stored in the context.
    #[diagnostic(
        code(analysis::internal::result_not_available),
        severity(Error), // Should be an internal error
        help("This indicates an internal error in the analysis host's execution order or result caching.")
    )]
    #[error("Result for pass '{pass_name}' (TypeId: {pass_id:?}) not found in context")]
    ResultNotAvailable {
        /// The name of the pass whose result was not available.
        pass_name: String,
        /// The type ID of the pass.
        pass_id: TypeId,
    },

    /// A result could not be downcast to the expected type.
    ///
    /// This error occurs when a pass tries to access the result of another pass
    /// but the stored result has a different type than expected.
    #[diagnostic(
        code(analysis::internal::downcast_error),
        severity(Error), // Should be an internal error
        help("The type of the stored result does not match the expected output type for pass '{pass_name}'. This indicates an internal error.")
    )]
    #[error("Failed to downcast result for pass '{pass_name}' (TypeId: {pass_id:?})")]
    DowncastError {
        /// The name of the pass whose result could not be downcast.
        pass_name: String,
        /// The type ID of the pass.
        pass_id: TypeId,
    },

    /// A pass was already registered.
    ///
    /// This error occurs when a pass is registered with the `AnalysisPipeline`
    /// more than once.
    #[diagnostic(
        code(analysis::pass_already_registered),
        help(
            "The pass '{pass_name}' (TypeId: {pass_id:?}) is already registered with the AnalysisHost."
        )
    )]
    #[error("Pass '{pass_name}' (TypeId: {pass_id:?}) is already registered")]
    PassAlreadyRegistered {
        /// The name of the pass that was already registered.
        pass_name: String,
        /// The type ID of the pass.
        pass_id: TypeId,
    },
}
