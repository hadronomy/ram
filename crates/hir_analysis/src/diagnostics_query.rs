//! Query function for diagnostics

use std::sync::Arc;

use hir::ids::DefId;
use ram_diagnostics::DiagnosticCollection;

/// Query function for getting diagnostics for a body
pub(crate) fn diagnostics_query(db: &dyn crate::AnalysisDatabase, def_id: DefId) -> Arc<DiagnosticCollection> {
    let analysis = crate::Analysis::new(db);
    let results = analysis.analyze_body(def_id);
    Arc::new(results.diagnostics)
}
