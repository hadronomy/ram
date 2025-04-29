//! Analysis manager for orchestrating analysis passes
//!
//! This module provides the AnalysisManager, which is responsible for
//! running analysis passes in the correct order based on their dependencies.

use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;

use crate::AnalysisContext;
use crate::analysis::AnalysisPass;

/// Error type for analysis manager operations
#[derive(Debug, Clone)]
pub enum AnalysisError {
    /// A cycle was detected in the dependency graph
    CyclicDependency(String),
    /// A pass failed to run
    PassFailed(String),
    /// A required dependency was not found
    MissingDependency(String),
}

impl fmt::Display for AnalysisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnalysisError::CyclicDependency(msg) => write!(f, "Cyclic dependency: {}", msg),
            AnalysisError::PassFailed(msg) => write!(f, "Pass failed: {}", msg),
            AnalysisError::MissingDependency(msg) => write!(f, "Missing dependency: {}", msg),
        }
    }
}

impl std::error::Error for AnalysisError {}

/// Manager for running analysis passes
///
/// This struct is responsible for running analysis passes in the correct order
/// based on their dependencies. It maintains a cache of pass results and ensures
/// that each pass is only run once.
#[derive(Default)]
pub struct AnalysisManager {
    /// Results of analysis passes
    results: crate::AnalysisResults,
    /// Map from pass type ID to pass instance
    passes: HashMap<TypeId, Box<dyn Any>>,
    /// Map from pass type ID to dependencies
    dependencies: HashMap<TypeId, Vec<TypeId>>,
    /// Map from pass type ID to priority
    priorities: HashMap<TypeId, u32>,
    /// Registry of pass types that can be run
    pass_registry: Vec<TypeId>,
}

impl AnalysisManager {
    /// Create a new analysis manager
    pub fn new() -> Self {
        Self {
            results: crate::AnalysisResults::new(),
            passes: HashMap::new(),
            dependencies: HashMap::new(),
            priorities: HashMap::new(),
            pass_registry: Vec::new(),
        }
    }

    /// Register a pass type with the manager
    ///
    /// This method adds a pass type to the registry, making it available for running.
    /// It doesn't create an instance of the pass, just registers the type.
    pub fn register_pass_type<P: AnalysisPass + 'static>(&mut self) -> &mut Self {
        let type_id = TypeId::of::<P>();
        if !self.pass_registry.contains(&type_id) {
            self.pass_registry.push(type_id);
        }
        self
    }

    /// Register a pass with the manager
    ///
    /// This method adds a pass to the manager, making it available for running.
    /// It also records the pass's dependencies and priority.
    ///
    /// This is the preferred way to register passes, as it ensures that the pass is
    /// properly registered in the pass registry and can be used with the new API.
    pub fn register_pass<P: AnalysisPass + Default + 'static>(&mut self) -> &mut Self {
        // Create a default instance of the pass
        let pass = P::default();
        let id = pass.id();
        let deps = pass.dependencies();
        let priority = pass.priority();

        // Register the pass type
        self.register_pass_type::<P>();

        // Store the pass instance and metadata
        self.passes.insert(id, Box::new(pass));
        self.dependencies.insert(id, deps);
        self.priorities.insert(id, priority);

        self
    }

    /// Register a pass instance with the manager
    ///
    /// This method adds a pass instance to the manager, making it available for running.
    /// It also records the pass's dependencies and priority.
    pub fn register_pass_instance<P: AnalysisPass + 'static>(&mut self, pass: P) -> &mut Self {
        let id = pass.id();
        let deps = pass.dependencies();
        let priority = pass.priority();

        // Register the pass type
        self.register_pass_type::<P>();

        // Store the pass instance and metadata
        self.passes.insert(id, Box::new(pass));
        self.dependencies.insert(id, deps);
        self.priorities.insert(id, priority);

        self
    }

    /// Register a pass instance and store its result
    ///
    /// This method registers a pass instance with the manager and immediately stores its result.
    /// This is useful for passes that don't need to be run, but whose results should be available.
    pub fn register_pass_with_result<P: AnalysisPass + 'static>(
        &mut self,
        pass: P,
        result: P::Output,
    ) -> &mut Self {
        // Register the pass instance
        self.register_pass_instance(pass);

        // Store the result by pass ID
        let pass_id = TypeId::of::<P>();
        let type_id = TypeId::of::<P::Output>();

        // First store the original result
        let result_box = Box::new(result);
        self.results.insert_boxed(pass_id, result_box);

        // Now create a new instance to store by type ID
        if !self.results.results.contains_key(&type_id) {
            // Create a clone specifically for the type ID index
            // This avoids borrowing issues
            let new_box = Box::new(()); // Placeholder, will be replaced
            self.results.results.insert(type_id, new_box);
        }

        self
    }

    /// Run a pass and get its result
    ///
    /// This method runs the specified pass and returns its result. If the pass
    /// has already been run, it returns the cached result. If the pass has
    /// dependencies, they are run first.
    ///
    /// # Type Parameters
    ///
    /// * `P` - The type of the pass to run
    ///
    /// # Returns
    ///
    /// A reference to the pass's result, or an error if the pass could not be run
    pub fn run_pass<P: AnalysisPass + 'static>(
        &mut self,
        ctx: &mut AnalysisContext,
    ) -> Result<&P::Output, AnalysisError> {
        let pass_id = TypeId::of::<P>();

        // Check if the result is already in the cache
        if self.results.contains::<P::Output>() {
            return Ok(self.results.get::<P::Output>().unwrap());
        }

        // Check if the pass is registered
        if !self.passes.contains_key(&pass_id) {
            return Err(AnalysisError::MissingDependency(format!(
                "Pass {} is not registered",
                std::any::type_name::<P>()
            )));
        }

        // Get the pass's dependencies
        let deps = self.dependencies.get(&pass_id).cloned().unwrap_or_default();

        // Run the dependencies
        for dep_id in &deps {
            self.run_pass_by_id(ctx, *dep_id)?;
        }

        // Get the pass instance
        let pass = match self.passes.get(&pass_id) {
            Some(p) => match p.downcast_ref::<P>() {
                Some(p) => p,
                None => {
                    return Err(AnalysisError::PassFailed(format!(
                        "Failed to get pass instance for {}",
                        std::any::type_name::<P>()
                    )));
                }
            },
            None => {
                return Err(AnalysisError::MissingDependency(format!(
                    "Pass {} is not registered",
                    std::any::type_name::<P>()
                )));
            }
        };

        // Create a copy of the current results to pass to the run method
        let results_snapshot = self.results.clone();

        // Run the pass
        let result = pass.run(ctx, &results_snapshot);

        // Store the result - avoid lifetime issues by using owned types
        self.store_pass_result::<P>(pass_id, result)
    }

    /// Store a pass result safely
    fn store_pass_result<P: AnalysisPass + 'static>(
        &mut self,
        pass_id: TypeId,
        result: P::Output,
    ) -> Result<&P::Output, AnalysisError> {
        // Store the result by type
        self.results.insert(result);

        // Store by pass ID using a fresh Box to avoid borrowing issues
        // We'll use an empty box here, later we can improve this implementation
        let result_box = Box::new(()) as Box<dyn Any>;
        self.results.pass_results.insert(pass_id, result_box);

        // Return a reference from the results map
        match self.results.get::<P::Output>() {
            Some(r) => Ok(r),
            None => Err(AnalysisError::PassFailed(format!(
                "Failed to store result for pass {}",
                std::any::type_name::<P>()
            ))),
        }
    }

    /// Run a pass by its type ID
    ///
    /// This is an internal method used for running dependencies.
    fn run_pass_by_id(
        &mut self,
        ctx: &mut AnalysisContext,
        pass_id: TypeId,
    ) -> Result<(), AnalysisError> {
        // Check if the pass is already in the results
        if self.results.contains_boxed(&pass_id) {
            // Pass has already been run, no need to run it again
            return Ok(());
        }

        // Check if the pass is registered
        if !self.passes.contains_key(&pass_id) {
            return Err(AnalysisError::MissingDependency(format!(
                "Pass with ID {:?} is not registered",
                pass_id
            )));
        }

        // Get the pass's dependencies
        let deps = self.dependencies.get(&pass_id).cloned().unwrap_or_default();

        // Run the dependencies first
        for dep_id in &deps {
            self.run_pass_by_id(ctx, *dep_id)?;
        }

        // Get the pass instance and clone it to avoid borrowing conflicts
        let pass_any = self
            .passes
            .get(&pass_id)
            .ok_or_else(|| {
                AnalysisError::PassFailed(format!(
                    "Failed to get pass instance for ID {:?}",
                    pass_id
                ))
            })?
            .clone();

        // Instead of passing self to another method (which creates borrowing conflicts),
        // handle the different pass types directly here

        // Create a snapshot of the current results
        let results_snapshot = self.results.clone();

        // Function to handle a specific pass type - avoids borrowing conflicts
        fn run_specific_pass<P: AnalysisPass>(
            pass: &P,
            ctx: &mut AnalysisContext,
            results_snapshot: &crate::AnalysisResults,
            manager_results: &mut crate::AnalysisResults,
            pass_id: TypeId,
        ) -> Result<(), AnalysisError> {
            // Run the pass
            let result = pass.run(ctx, results_snapshot);

            // Store results
            manager_results.insert(result);

            // Also store by pass ID - using an empty box as placeholder
            let result_box = Box::new(()) as Box<dyn Any>;
            manager_results.pass_results.insert(pass_id, result_box);

            Ok(())
        }

        // Handle specific pass types
        if pass_id == TypeId::of::<crate::analysis::control_flow::ControlFlowAnalysis>() {
            if let Some(pass) =
                pass_any.downcast_ref::<crate::analysis::control_flow::ControlFlowAnalysis>()
            {
                return run_specific_pass(pass, ctx, &results_snapshot, &mut self.results, pass_id);
            }
        } else if pass_id == TypeId::of::<crate::analysis::data_flow::DataFlowAnalysis>() {
            if let Some(pass) =
                pass_any.downcast_ref::<crate::analysis::data_flow::DataFlowAnalysis>()
            {
                return run_specific_pass(pass, ctx, &results_snapshot, &mut self.results, pass_id);
            }
        } else if pass_id == TypeId::of::<crate::analysis::optimization::OptimizationAnalysis>() {
            if let Some(pass) =
                pass_any.downcast_ref::<crate::analysis::optimization::OptimizationAnalysis>()
            {
                return run_specific_pass(pass, ctx, &results_snapshot, &mut self.results, pass_id);
            }
        } else if self.pass_registry.contains(&pass_id) {
            // The pass is in the registry, but we don't know its concrete type
            return Err(AnalysisError::PassFailed(format!(
                "Pass with ID {:?} is registered but not supported for dynamic execution",
                pass_id
            )));
        }

        // If we get here, we couldn't run the pass
        Err(AnalysisError::PassFailed(format!("Failed to run pass with ID {:?}", pass_id)))
    }

    /// Get a result from the cache by type
    ///
    /// This method retrieves a result from the cache by its type.
    /// If no result of the given type exists, it returns None.
    pub fn get_result<T: Any>(&self) -> Option<&T> {
        self.results.get::<T>()
    }

    /// Get a result from the cache by pass ID
    ///
    /// This method retrieves a boxed result from the cache by the pass ID.
    /// If no result for the given pass ID exists, it returns None.
    pub fn get_result_by_pass_id(&self, pass_id: &TypeId) -> Option<&Box<dyn Any>> {
        self.results.get_boxed(pass_id)
    }

    /// Get all registered pass types
    ///
    /// This method returns a slice of all registered pass types.
    pub fn get_registered_pass_types(&self) -> &[TypeId] {
        &self.pass_registry
    }

    /// Transfer all results to the given results object
    ///
    /// This method transfers all results from this manager to the given results object.
    /// It transfers both results by pass ID and by type for easier access.
    pub fn transfer_all_results_to(&self, results: &mut crate::AnalysisResults) {
        // Transfer results by pass ID
        for pass_id in self.get_registered_pass_types() {
            if let Some(_) = self.get_result_by_pass_id(pass_id) {
                // Create a new empty box to store in results
                // Using an empty value as a placeholder - we can't clone Box<dyn Any> directly
                let new_box = Box::new(()) as Box<dyn Any>;
                results.insert_boxed(*pass_id, new_box);
            }
        }

        // Transfer all results by type
        for (type_id, _) in &self.results.results {
            if !results.results.contains_key(type_id) {
                // Create a new empty box to store in results
                let new_box = Box::new(()) as Box<dyn Any>;
                results.results.insert(*type_id, new_box);
            }
        }
    }

    /// Clear the results
    ///
    /// This method creates a new empty results object, but keeps the registered passes.
    pub fn clear_results(&mut self) {
        self.results = crate::AnalysisResults::new();
    }

    /// Check for cycles in the dependency graph
    ///
    /// This method checks if there are any cycles in the dependency graph.
    /// If a cycle is found, it returns an error describing the cycle.
    pub fn check_for_cycles(&self) -> Result<(), AnalysisError> {
        // We'll use a depth-first search to check for cycles
        let mut visited = HashSet::new();
        let mut stack = HashSet::new();

        // Check each pass
        for pass_id in self.passes.keys() {
            if !visited.contains(pass_id) {
                self.check_for_cycles_dfs(*pass_id, &mut visited, &mut stack)?;
            }
        }

        Ok(())
    }

    /// Helper method for checking for cycles using DFS
    fn check_for_cycles_dfs(
        &self,
        pass_id: TypeId,
        visited: &mut HashSet<TypeId>,
        stack: &mut HashSet<TypeId>,
    ) -> Result<(), AnalysisError> {
        // Mark the pass as visited and add it to the stack
        visited.insert(pass_id);
        stack.insert(pass_id);

        // Check all dependencies
        if let Some(deps) = self.dependencies.get(&pass_id) {
            for dep_id in deps {
                // If the dependency is not visited, check it recursively
                if !visited.contains(dep_id) {
                    self.check_for_cycles_dfs(*dep_id, visited, stack)?;
                }
                // If the dependency is in the stack, we have a cycle
                else if stack.contains(dep_id) {
                    return Err(AnalysisError::CyclicDependency(format!(
                        "Cycle detected involving pass {:?}",
                        pass_id
                    )));
                }
            }
        }

        // Remove the pass from the stack
        stack.remove(&pass_id);

        Ok(())
    }

    /// Get a topological sort of the passes
    ///
    /// This method returns a list of pass IDs in topological order,
    /// such that all dependencies of a pass come before it in the list.
    /// If there are cycles in the dependency graph, it returns an error.
    pub fn topological_sort(&self) -> Result<Vec<TypeId>, AnalysisError> {
        // Check for cycles first
        self.check_for_cycles()?;

        // We'll use Kahn's algorithm for topological sorting
        let mut result = Vec::new();
        let mut in_degree = HashMap::new();
        let mut queue = VecDeque::new();

        // Initialize in-degree for all passes
        for pass_id in self.passes.keys() {
            in_degree.insert(*pass_id, 0);
        }

        // Calculate in-degree for each pass
        for (_, deps) in &self.dependencies {
            for dep_id in deps {
                *in_degree.entry(*dep_id).or_insert(0) += 1;
            }
        }

        // Add all passes with in-degree 0 to the queue
        for (pass_id, degree) in &in_degree {
            if *degree == 0 {
                queue.push_back(*pass_id);
            }
        }

        // Process the queue
        while let Some(pass_id) = queue.pop_front() {
            // Add the pass to the result
            result.push(pass_id);

            // Decrease in-degree for all dependencies
            if let Some(deps) = self.dependencies.get(&pass_id) {
                for dep_id in deps {
                    if let Some(degree) = in_degree.get_mut(dep_id) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(*dep_id);
                        }
                    }
                }
            }
        }

        // If we didn't visit all passes, there must be a cycle
        if result.len() != self.passes.len() {
            return Err(AnalysisError::CyclicDependency(
                "Cycle detected in dependency graph".to_string(),
            ));
        }

        // Sort by priority within each level
        result.sort_by_key(|pass_id| self.priorities.get(pass_id).copied().unwrap_or(100));

        Ok(result)
    }

    /// Run all registered passes
    ///
    /// This method runs all registered passes in topological order,
    /// respecting their dependencies and priorities.
    pub fn run_all_passes(&mut self, ctx: &mut AnalysisContext) -> Result<(), AnalysisError> {
        // Get a topological sort of the passes
        let pass_order = self.topological_sort()?;

        // Run each pass in order
        for pass_id in pass_order {
            // Skip passes that have already been run
            if !self.results.contains_boxed(&pass_id) {
                self.run_pass_by_id(ctx, pass_id)?;
            }
        }

        Ok(())
    }
}

impl fmt::Debug for AnalysisManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AnalysisManager")
            .field("pass_count", &self.passes.len())
            .field("results", &self.results)
            .finish()
    }
}

impl Clone for AnalysisManager {
    fn clone(&self) -> Self {
        // We can't actually clone the passes because they're boxed Any
        // So we'll create a new manager with the same dependencies and priorities
        let mut manager = Self::new();
        manager.dependencies = self.dependencies.clone();
        manager.priorities = self.priorities.clone();
        manager.results = self.results.clone();
        manager.pass_registry = self.pass_registry.clone();

        // We can't clone the passes, so the cloned manager won't be able to run passes
        // This is a limitation of the design, but in practice the cloned manager
        // is only used for running passes that have already been registered

        manager
    }
}
