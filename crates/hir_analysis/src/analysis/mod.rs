//! Analysis passes for HIR
//!
//! This module provides various analysis passes for HIR.

use std::any::{Any, TypeId};

use crate::AnalysisContext;

pub mod control_flow;
pub mod data_flow;
pub mod macros;
pub mod manager;
pub mod optimization;

/// A trait for analysis passes that can be run on a body
///
/// This trait defines the interface for all analysis passes that can be
/// run as part of semantic analysis. Each pass has an associated output type
/// and can depend on other passes.
pub trait AnalysisPass: Clone + Default + 'static {
    /// The output type of this pass
    ///
    /// This associated type defines the result type of the pass.
    /// It must implement `Any` so it can be stored in the results cache.
    type Output: Any;

    /// Run the analysis pass on the given context and dependencies
    ///
    /// This method runs the analysis pass on the given context, using the
    /// results of any dependencies from the provided results.
    ///
    /// # Parameters
    ///
    /// * `ctx` - The analysis context to operate on
    /// * `dependencies` - The results from previous passes
    ///
    /// # Returns
    ///
    /// The result of the analysis pass, which will be stored in the results.
    fn run<'db, 'body>(
        &self,
        ctx: &mut AnalysisContext<'db, 'body>,
        dependencies: &crate::AnalysisResults,
    ) -> Self::Output;

    /// Get the type ID of this pass
    ///
    /// This method returns the TypeId of the pass's concrete type.
    /// It's used for identifying passes in the dependency graph.
    ///
    /// You should not need to override this method.
    fn id(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    /// Get the name of this pass
    ///
    /// This method returns a human-readable name for the pass.
    /// It's used for debugging and logging.
    ///
    /// The default implementation returns the type name of the pass.
    /// You can override this to provide a more descriptive name.
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    /// Get the description of this pass
    ///
    /// This method returns a human-readable description of what the pass does.
    /// It's used for documentation and logging.
    ///
    /// You should override this to provide a description of your pass.
    fn description(&self) -> &'static str {
        "No description provided"
    }

    /// Get the dependencies of this pass
    ///
    /// This method returns a list of TypeIds for the passes that this pass
    /// depends on. It's used for building the dependency graph.
    ///
    /// The default implementation returns an empty list, meaning the pass
    /// has no dependencies.
    ///
    /// You should override this to specify the dependencies of your pass.
    /// For example:
    ///
    /// ```
    /// fn dependencies(&self) -> Vec<TypeId> {
    ///     vec![
    ///         TypeId::of::<ControlFlowAnalysis>(),
    ///         TypeId::of::<DataFlowAnalysis>(),
    ///     ]
    /// }
    /// ```
    fn dependencies(&self) -> Vec<TypeId> {
        Vec::new()
    }

    /// Get the priority of this pass
    ///
    /// This method returns a priority value for the pass. Passes with lower
    /// priority values run earlier, all else being equal.
    ///
    /// The default priority is 100.
    ///
    /// You can override this to change the priority of your pass.
    fn priority(&self) -> u32 {
        100
    }

    /// Get a dependency result from the results
    ///
    /// This is a helper method for getting the result of a dependency pass.
    /// It's used in the `run` method to get the results of dependency passes.
    ///
    /// # Type Parameters
    ///
    /// * `P` - The type of the dependency pass
    ///
    /// # Parameters
    ///
    /// * `dependencies` - The results from previous passes
    ///
    /// # Returns
    ///
    /// The result of the dependency pass, or None if it's not in the results.
    fn get_dependency<'a, P: AnalysisPass>(
        &self,
        dependencies: &'a crate::AnalysisResults,
    ) -> Option<&'a P::Output> {
        // First try to get the result by pass type
        dependencies.get_by_pass::<P>().or_else(|| {
            // If that fails, try to get it by output type
            dependencies.get::<P::Output>()
        })
    }
}
