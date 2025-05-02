//! Semantic analysis for RAM HIR
//!
//! This crate provides semantic analysis capabilities for the RAM High-level
//! Intermediate Representation (HIR). It includes type checking, control flow
//! analysis, data flow analysis, and other semantic validations.
//!
//! The main components of this crate are:
//!
//! * [`AnalysisContext`] - Stores and provides access to analysis results.
//! * [`AnalysisPass`] - Trait for implementing analysis passes.
//! * [`AnalysisPipeline`] - Manages the registration and execution of analysis passes.
//! * [`AnalysisError`] - Error types for the HIR analysis.
//!
//! # Example
//!
//! ```rust
//! use hir_analysis::{AnalysisPipeline, AnalysisPass, AnalysisContext, AnalysisError};
//! use hir::body::Body;
//! use std::sync::Arc;
//! use std::any::TypeId;
//! use miette::Diagnostic;
//!
//! // Define an analysis pass
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
//!
//! // Create a pipeline and register the pass
//! // let mut pipeline = AnalysisPipeline::new();
//! // pipeline.register::<MyPass>().unwrap();
//!
//! // Run the analysis on a body
//! // let body = Arc::new(Body::default());
//! // let context = pipeline.analyze(body).unwrap();
//!
//! // Get the result
//! // let result = context.get_result::<MyPass>().unwrap();
//! ```

pub mod analyzers;
pub mod context;
pub mod error;
pub mod export;
pub mod pass;
pub mod pipeline;
pub mod visitors;

// Re-export main components
pub use analyzers::constant_propagation::{
    BranchTaken, ConstantPropagationAnalysis, ConstantPropagationResult,
};
pub use analyzers::control_flow::{ControlFlowAnalysis, ControlFlowGraph};
pub use analyzers::control_flow_optimizer::{ControlFlowOptimizer, OptimizedControlFlowGraph};
pub use analyzers::data_flow::{DataFlowAnalysis, DataFlowGraph};
pub use analyzers::instruction_validation::InstructionValidationAnalysis;
pub use context::AnalysisContext;
pub use error::AnalysisError;
pub use export::{ExportFormat, ExportOptions};
pub use pass::AnalysisPass;
pub use pipeline::AnalysisPipeline;

#[cfg(test)]
mod tests;
