//! Database traits for semantic analysis
//!
//! This module defines the database traits used for incremental computation
//! in semantic analysis. It builds on the salsa crate to provide efficient
//! caching of analysis results.

use std::sync::Arc;

use hir::body::Body;
use hir::ids::{DefId, LocalDefId};
use hir::HirDatabase;

use ram_diagnostics::DiagnosticCollection;
use crate::types::TypeInfo;
use crate::analysis::control_flow::ControlFlowGraph;
use crate::analysis::data_flow::DataFlowResults;
use crate::analysis::optimization::Optimization;

/// Database trait for semantic analysis
///
/// This trait extends the HIR database with methods for semantic analysis.
/// It provides access to the HIR and caches analysis results.
#[salsa::query_group(AnalysisStorageStorage)]
pub trait AnalysisDatabase: HirDatabase {
    /// Get the type information for a body
    #[salsa::invoke(crate::types::type_check_query)]
    fn type_info(&self, def_id: DefId) -> Arc<TypeInfo>;

    /// Get the control flow graph for a body
    #[salsa::invoke(crate::analysis::control_flow::control_flow_query)]
    fn control_flow_graph(&self, def_id: DefId) -> Arc<ControlFlowGraph>;

    /// Get the data flow analysis results for a body
    #[salsa::invoke(crate::analysis::data_flow::data_flow_query)]
    fn data_flow_results(&self, def_id: DefId) -> Arc<DataFlowResults>;

    /// Get the optimization opportunities for a body
    #[salsa::invoke(crate::analysis::optimization::optimization_query)]
    fn optimizations(&self, def_id: DefId) -> Arc<Vec<Optimization>>;

    /// Get the diagnostics for a body
    #[salsa::invoke(crate::diagnostics_query::diagnostics_query)]
    fn diagnostics(&self, def_id: DefId) -> Arc<DiagnosticCollection>;

    /// Check if an instruction is reachable
    #[salsa::invoke(crate::analysis::control_flow::is_reachable_query)]
    fn is_instruction_reachable(&self, def_id: DefId, instr_id: LocalDefId) -> bool;

    /// Get the type of an expression
    #[salsa::invoke(crate::types::expr_type_query)]
    fn expr_type(&self, def_id: DefId, expr_id: hir::ExprId) -> crate::types::TypeId;
}

/// Storage for analysis database
#[salsa::database(AnalysisStorageStorage)]
#[derive(Default)]
pub struct AnalysisStorage {
    storage: salsa::Storage<Self>,
}

impl salsa::Database for AnalysisStorage {}

impl std::fmt::Debug for AnalysisStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnalysisStorage").finish()
    }
}

/// Implement HirDatabase for AnalysisStorage by delegating to the inner database
impl HirDatabase for AnalysisStorage {
    fn body(&self, def_id: DefId) -> Arc<Body> {
        // This would delegate to the inner HirDatabase implementation
        // For now, we'll just panic as this is just an example
        unimplemented!("AnalysisStorage::body not implemented")
    }

    // Implement other methods from HirDatabase...
}
