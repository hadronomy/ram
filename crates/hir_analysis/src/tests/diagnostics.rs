//! Tests for diagnostic collection in analysis passes.

use std::any::TypeId;
use std::sync::Arc;

use hir::body::Body;
use miette::Diagnostic;
use ram_diagnostics::DiagnosticKind;

use crate::{AnalysisContext, AnalysisPass, AnalysisPipeline};

// --- Test Passes with Diagnostics ---

/// A pass that reports an error diagnostic
#[derive(Default)]
struct ErrorReportingPass;

impl AnalysisPass for ErrorReportingPass {
    type Output = ();

    fn name(&self) -> &'static str {
        "ErrorReportingPass"
    }

    fn dependencies(&self) -> Vec<TypeId> {
        vec![]
    }

    fn run(&self, ctx: &mut AnalysisContext) -> Result<Self::Output, Box<dyn Diagnostic>> {
        // Report an error diagnostic
        ctx.error("This is an error message", "This is a help message for the error", Some(10..20));
        Ok(())
    }
}

/// A pass that reports a warning diagnostic
#[derive(Default)]
struct WarningReportingPass;

impl AnalysisPass for WarningReportingPass {
    type Output = ();

    fn name(&self) -> &'static str {
        "WarningReportingPass"
    }

    fn dependencies(&self) -> Vec<TypeId> {
        vec![]
    }

    fn run(&self, ctx: &mut AnalysisContext) -> Result<Self::Output, Box<dyn Diagnostic>> {
        // Report a warning diagnostic
        ctx.warning(
            "This is a warning message",
            "This is a help message for the warning",
            Some(30..40),
        );
        Ok(())
    }
}

/// A pass that reports an info diagnostic
#[derive(Default)]
struct InfoReportingPass;

impl AnalysisPass for InfoReportingPass {
    type Output = ();

    fn name(&self) -> &'static str {
        "InfoReportingPass"
    }

    fn dependencies(&self) -> Vec<TypeId> {
        vec![]
    }

    fn run(&self, ctx: &mut AnalysisContext) -> Result<Self::Output, Box<dyn Diagnostic>> {
        // Report an info diagnostic
        ctx.info("This is an info message", "This is a help message for the info", Some(50..60));
        Ok(())
    }
}

/// A pass that depends on other passes and checks their diagnostics
#[derive(Default)]
struct DiagnosticCheckingPass;

impl AnalysisPass for DiagnosticCheckingPass {
    type Output = usize;

    fn name(&self) -> &'static str {
        "DiagnosticCheckingPass"
    }

    fn dependencies(&self) -> Vec<TypeId> {
        vec![
            TypeId::of::<ErrorReportingPass>(),
            TypeId::of::<WarningReportingPass>(),
            TypeId::of::<InfoReportingPass>(),
        ]
    }

    fn run(&self, ctx: &mut AnalysisContext) -> Result<Self::Output, Box<dyn Diagnostic>> {
        // Return the total number of diagnostics
        Ok(ctx.diagnostics().len())
    }
}

#[test]
fn test_diagnostic_collection() {
    // Create a pipeline and register the passes
    let mut pipeline = AnalysisPipeline::new();
    pipeline.register::<ErrorReportingPass>().unwrap();
    pipeline.register::<WarningReportingPass>().unwrap();
    pipeline.register::<InfoReportingPass>().unwrap();
    pipeline.register::<DiagnosticCheckingPass>().unwrap();

    // Run the analysis on a dummy body
    let body = Arc::new(Body::default());
    let context = pipeline.analyze(body).unwrap();

    // Check that the diagnostics were collected
    assert_eq!(context.diagnostics().len(), 3);
    assert_eq!(context.diagnostics().error_count(), 1);
    assert_eq!(context.diagnostics().warning_count(), 1);

    // Check that the DiagnosticCheckingPass got the correct count
    let result = context.get_result::<DiagnosticCheckingPass>().unwrap();
    assert_eq!(*result, 3);

    // Check that has_errors returns true
    assert!(context.has_errors());

    // Check the specific diagnostics
    let diagnostics = context.diagnostics().diagnostics();

    // Find each diagnostic by its kind and check its properties
    let error = diagnostics
        .iter()
        .find(|d| d.kind == DiagnosticKind::Error)
        .expect("Error diagnostic not found");
    assert_eq!(error.message, "This is an error message");
    assert_eq!(error.help, "This is a help message for the error");
    assert_eq!(error.labeled_spans[0].0, 10..20);

    let warning = diagnostics
        .iter()
        .find(|d| d.kind == DiagnosticKind::Warning)
        .expect("Warning diagnostic not found");
    assert_eq!(warning.message, "This is a warning message");
    assert_eq!(warning.help, "This is a help message for the warning");
    assert_eq!(warning.labeled_spans[0].0, 30..40);

    let info = diagnostics
        .iter()
        .find(|d| d.kind == DiagnosticKind::Advice)
        .expect("Info diagnostic not found");
    assert_eq!(info.message, "This is an info message");
    assert_eq!(info.help, "This is a help message for the info");
    assert_eq!(info.labeled_spans[0].0, 50..60);
}
