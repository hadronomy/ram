//! Example of using the hir_analysis crate
//!
//! This example shows how to use the hir_analysis crate to analyze a RAM program.

use std::sync::Arc;

use hir::ids::DefId;
use hir::body::Body;
use hir::HirDatabase;

use hir_analysis::{Analysis, AnalysisDatabase, AnalysisStorage};
use ram_diagnostics::DiagnosticKind;

// A simple implementation of HirDatabase for testing
struct TestDatabase {
    body: Arc<Body>,
}

impl HirDatabase for TestDatabase {
    fn body(&self, _def_id: DefId) -> Arc<Body> {
        self.body.clone()
    }
}

// A simple implementation of AnalysisDatabase for testing
struct TestAnalysisDatabase {
    hir_db: TestDatabase,
    analysis_storage: AnalysisStorage,
}

impl HirDatabase for TestAnalysisDatabase {
    fn body(&self, def_id: DefId) -> Arc<Body> {
        self.hir_db.body(def_id)
    }
}

impl AnalysisDatabase for TestAnalysisDatabase {}

fn main() {
    // Create a simple body for testing
    let body = Body::default(); // In a real example, this would be a real body

    // Create the database
    let hir_db = TestDatabase {
        body: Arc::new(body),
    };

    let db = TestAnalysisDatabase {
        hir_db,
        analysis_storage: AnalysisStorage::default(),
    };

    // Create the analysis
    let analysis = Analysis::new(&db);

    // Analyze the body
    let results = analysis.analyze_body(DefId::ROOT);

    // Print the results
    println!("Analysis results:");
    println!("  Diagnostics: {} ({} errors)", results.diagnostics.len(), if results.diagnostics.has_errors() { "has errors" } else { "no errors" });

    // Print diagnostics
    for diagnostic in results.diagnostics.diagnostics() {
        match diagnostic.kind {
            DiagnosticKind::Error => println!("    Error: {}", diagnostic.message),
            DiagnosticKind::Warning => println!("    Warning: {}", diagnostic.message),
            DiagnosticKind::Advice => println!("    Advice: {}", diagnostic.message),
            DiagnosticKind::Custom(name) => println!("    {}: {}", name, diagnostic.message),
        }
    }

    if let Some(cfg) = &results.control_flow {
        println!("  Control flow graph: {} blocks", cfg.blocks.len());
        println!("  Unreachable blocks: {:?}", cfg.unreachable_blocks());
    }

    if let Some(data_flow) = &results.data_flow {
        println!("  Data flow analysis: {} blocks", data_flow.block_data_flow.len());
    }

    println!("  Optimization opportunities: {}", results.optimizations.len());
    for opt in &results.optimizations {
        match opt {
            hir_analysis::analysis::optimization::Optimization::DeadCode { instruction_id, description } => {
                println!("    Dead code: {} (instruction {:?})", description, instruction_id);
            },
            hir_analysis::analysis::optimization::Optimization::ConstantPropagation { instruction_id, value, description } => {
                println!("    Constant propagation: {} (instruction {:?}, value {})", description, instruction_id, value);
            },
            hir_analysis::analysis::optimization::Optimization::CommonSubexpression { instruction_id, equivalent_id, description } => {
                println!("    Common subexpression: {} (instruction {:?}, equivalent {:?})", description, instruction_id, equivalent_id);
            },
            hir_analysis::analysis::optimization::Optimization::StrengthReduction { instruction_id, new_opcode, description } => {
                println!("    Strength reduction: {} (instruction {:?}, new opcode {})", description, instruction_id, new_opcode);
            },
        }
    }
}
