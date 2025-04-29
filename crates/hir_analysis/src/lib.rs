//! Semantic analysis for RAM HIR
//!
//! This crate provides semantic analysis capabilities for the RAM High-level
//! Intermediate Representation (HIR). It includes type checking, control flow
//! analysis, data flow analysis, and other semantic validations.
//!
//! The main components of this crate are:
//!
//! * [`AnalysisContext`](context::AnalysisContext) - Stores and provides access to analysis results.
//! * [`AnalysisPass`](pass::AnalysisPass) - Trait for implementing analysis passes.
//! * [`AnalysisPipeline`](pipeline::AnalysisPipeline) - Manages the registration and execution of analysis passes.
//! * [`AnalysisError`](error::AnalysisError) - Error types for the HIR analysis.
//!
//! # Example
//!
//! ```rust
//! use hir_analysis::pipeline::AnalysisPipeline;
//! use hir_analysis::pass::AnalysisPass;
//! use hir_analysis::context::AnalysisContext;
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
//! // let body = Arc::new(Body::new(...));
//! // let context = pipeline.analyze(body).unwrap();
//!
//! // Get the result
//! // let result = context.get_result::<MyPass>().unwrap();
//! ```

pub mod context;
pub mod error;
pub mod pass;
pub mod pipeline;

// TODO: Re-export macros
