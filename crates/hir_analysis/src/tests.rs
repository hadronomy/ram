use std::any::TypeId;
use std::sync::Arc;

use hir::body::Body;
use miette::Diagnostic;

use crate::{AnalysisContext, AnalysisError, AnalysisPass, AnalysisPipeline};

// --- Dummy Passes ---
#[derive(Default)]
struct PassA;
impl AnalysisPass for PassA {
    type Output = String;
    fn name(&self) -> &'static str {
        "PassA"
    }
    fn dependencies(&self) -> Vec<TypeId> {
        vec![]
    }
    fn run(&self, ctx: &mut AnalysisContext) -> Result<Self::Output, Box<dyn Diagnostic>> {
        println!("Running PassA on body: {:?}", ctx.body());
        Ok("Result from PassA".to_string())
    }
}

#[derive(Default)]
struct PassB;
impl AnalysisPass for PassB {
    type Output = u32;
    fn name(&self) -> &'static str {
        "PassB"
    }
    fn dependencies(&self) -> Vec<TypeId> {
        vec![TypeId::of::<PassA>()]
    }
    fn run(&self, ctx: &mut AnalysisContext) -> Result<Self::Output, Box<dyn Diagnostic>> {
        println!("Running PassB on body: {:?}", ctx.body());
        let pass_a_result = match ctx.get_result::<PassA>() {
            Ok(result) => result,
            Err(e) => return Err(Box::new(e)),
        };
        println!("PassB received from PassA: '{}'", pass_a_result);
        Ok(pass_a_result.len() as u32)
    }
}

#[derive(Default)]
struct PassC;
impl AnalysisPass for PassC {
    type Output = f64;
    fn name(&self) -> &'static str {
        "PassC"
    }
    fn dependencies(&self) -> Vec<TypeId> {
        vec![TypeId::of::<PassB>()]
    }
    fn run(&self, ctx: &mut AnalysisContext) -> Result<Self::Output, Box<dyn Diagnostic>> {
        println!("Running PassC on body: {:?}", ctx.body());
        let pass_b_result = match ctx.get_result::<PassB>() {
            Ok(result) => result,
            Err(e) => return Err(Box::new(e)),
        };
        println!("PassC received from PassB: '{}'", pass_b_result);
        Ok(*pass_b_result as f64 * 1.5)
    }
}

#[test]
fn test_analysis_pipeline() -> Result<(), AnalysisError> {
    // Use AnalysisPipeline
    let mut pipeline = AnalysisPipeline::new();

    // Register passes - order matters if dependencies aren't registered first
    pipeline.register::<PassA>()?;
    pipeline.register::<PassB>()?; // Depends on A
    pipeline.register::<PassC>()?; // Depends on B

    // Create dummy input
    let dummy_body = Arc::new(Body::default());

    // Run the analysis
    let final_context = pipeline.analyze(dummy_body)?;

    // Check results
    let result_a = final_context.get_result::<PassA>()?;
    assert_eq!(*result_a, "Result from PassA");

    let result_b = final_context.get_result::<PassB>()?;
    assert_eq!(*result_b, 17); // Length of "Result from PassA"

    let result_c = final_context.get_result::<PassC>()?;
    assert_eq!(*result_c, 25.5); // 17.0 * 1.5

    println!("Analysis finished. Final context: {:?}", final_context);

    Ok(())
}

#[test]
fn test_dependency_error() {
    // Use AnalysisPipeline
    let mut pipeline = AnalysisPipeline::new();
    // Try to register B before A
    let result = pipeline.register::<PassB>();
    assert!(matches!(result, Err(AnalysisError::PassNotRegistered { .. })));
    println!("Correctly failed to register PassB before PassA: {:?}", result);
}

// Since we can't easily create a cycle in the test (the registration process prevents it),
// let's test that we can detect dependency errors instead
#[test]
fn test_missing_dependency() {
    // Define a pass that depends on a non-existent pass
    struct MissingDependencyPass;
    impl AnalysisPass for MissingDependencyPass {
        type Output = ();
        fn name(&self) -> &'static str {
            "MissingDependencyPass"
        }
        fn dependencies(&self) -> Vec<TypeId> {
            // This TypeId doesn't correspond to any registered pass
            vec![TypeId::of::<String>()]
        }
        fn run(&self, _ctx: &mut AnalysisContext) -> Result<Self::Output, Box<dyn Diagnostic>> {
            Ok(())
        }
    }

    let mut pipeline = AnalysisPipeline::new();
    let result = pipeline.register_pass(MissingDependencyPass);

    println!("Missing dependency result: {:?}", result);
    assert!(matches!(result, Err(AnalysisError::PassNotRegistered { .. })));
}
