//! Database traits for semantic analysis
//!
//! This module defines the database traits used for incremental computation
//! in semantic analysis. It builds on the salsa crate to provide efficient
//! caching of analysis results.

use std::sync::Arc;

use hir::expr::ExprId;
use hir::ids::{DefId, LocalDefId};
use ram_diagnostics::DiagnosticCollection;

use crate::analysis::control_flow::ControlFlowGraph;
use crate::analysis::data_flow::DataFlowResults;
use crate::analysis::optimization::Optimization;
use crate::types::TypeInfo;

/// Database trait for semantic analysis
///
/// This trait extends the HIR database with methods for semantic analysis.
/// It provides access to the HIR and caches analysis results.
pub trait AnalysisDatabase: hir::db::HirDatabase {
    /// Get the type information for a body
    fn type_info(&self, def_id: DefId) -> Arc<TypeInfo>;

    /// Get the control flow graph for a body
    fn control_flow_graph(&self, def_id: DefId) -> Arc<ControlFlowGraph>;

    /// Get the data flow analysis results for a body
    fn data_flow_results(&self, def_id: DefId) -> Arc<DataFlowResults>;

    /// Get the optimization opportunities for a body
    fn optimizations(&self, def_id: DefId) -> Arc<Vec<Optimization>>;

    /// Get the diagnostics for a body
    fn diagnostics(&self, def_id: DefId) -> Arc<DiagnosticCollection>;

    /// Check if an instruction is reachable
    fn is_instruction_reachable(&self, def_id: DefId, instr_id: LocalDefId) -> bool;

    /// Get the type of an expression
    fn expr_type(&self, def_id: DefId, expr_id: ExprId) -> crate::types::TypeId;
}

/// Implementation of the AnalysisDatabase trait for any type that implements HirDatabase
impl<DB> AnalysisDatabase for DB
where
    DB: hir::db::HirDatabase + salsa::Database,
{
    fn type_info(&self, def_id: DefId) -> Arc<TypeInfo> {
        crate::types::type_check_query(self, def_id)
    }

    fn control_flow_graph(&self, def_id: DefId) -> Arc<ControlFlowGraph> {
        crate::analysis::control_flow::control_flow_query(self, def_id)
    }

    fn data_flow_results(&self, def_id: DefId) -> Arc<DataFlowResults> {
        crate::analysis::data_flow::data_flow_query(self, def_id)
    }

    fn optimizations(&self, def_id: DefId) -> Arc<Vec<Optimization>> {
        crate::analysis::optimization::optimization_query(self, def_id)
    }

    fn diagnostics(&self, def_id: DefId) -> Arc<DiagnosticCollection> {
        crate::diagnostics_query::diagnostics_query(self, def_id)
    }

    fn is_instruction_reachable(&self, def_id: DefId, instr_id: LocalDefId) -> bool {
        crate::analysis::control_flow::is_reachable_query(self, def_id, instr_id)
    }

    fn expr_type(&self, def_id: DefId, expr_id: ExprId) -> crate::types::TypeId {
        crate::types::expr_type_query(self, def_id, expr_id)
    }
}
