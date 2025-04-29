//! Macros for analysis passes
//!
//! This module provides macros for defining analysis passes and their dependencies.
//! When the `macros` feature is enabled, these are implemented as procedural macros.
//! Otherwise, they are implemented as regular macros.

#[cfg(feature = "macros")]
pub use hir_analysis_macros::{analysis_pass, analysis_pass_with_deps, depends_on};

#[cfg(not(feature = "macros"))]
mod macros {
    /// Placeholder for the analysis_pass attribute macro
    ///
    /// In a real implementation, this would be a procedural macro that generates
    /// the implementation of the AnalysisPass trait for a struct.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// #[analysis_pass(output = "ControlFlowGraph")]
    /// struct ControlFlowAnalysisPass;
    /// ```
    #[macro_export]
    macro_rules! analysis_pass {
        ($pass:ident, $output:ty) => {
            impl $crate::analysis::AnalysisPass for $pass {
                type Output = $output;

                fn run<'db, 'body>(
                    &self,
                    ctx: &mut $crate::AnalysisContext<'db, 'body>,
                    dependencies: &$crate::analysis::results::AnalysisResultsCache,
                ) -> Self::Output {
                    // This is where the actual implementation would go
                    // For now, we just call the old run method for compatibility
                    self.run_impl(ctx, dependencies)
                }
            }
        };
    }

    /// Placeholder for the depends_on attribute macro
    ///
    /// In a real implementation, this would be a procedural macro that generates
    /// the implementation of the dependencies method for an analysis pass.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// #[depends_on(ControlFlowAnalysisPass, DataFlowAnalysisPass)]
    /// struct OptimizationAnalysisPass;
    /// ```
    #[macro_export]
    macro_rules! depends_on {
        ($pass:ident, $($dependency:ident),*) => {
            impl $pass {
                fn dependencies() -> Vec<std::any::TypeId> {
                    vec![$(std::any::TypeId::of::<$dependency>()),*]
                }
            }
        };
    }

    /// Placeholder for a combined analysis_pass and depends_on macro
    ///
    /// This macro combines the functionality of analysis_pass and depends_on.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// #[analysis_pass_with_deps(output = "OptimizationResults", deps = [ControlFlowAnalysisPass, DataFlowAnalysisPass])]
    /// struct OptimizationAnalysisPass;
    /// ```
    #[macro_export]
    macro_rules! analysis_pass_with_deps {
        ($pass:ident, $output:ty, $($dependency:ident),*) => {
            $crate::analysis_pass!($pass, $output);
            $crate::depends_on!($pass, $($dependency),*);
        };
    }
}
