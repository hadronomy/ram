//! Analysis pipeline for executing analysis passes in dependency order.
//!
//! This module provides the `AnalysisPipeline` which manages the registration and execution
//! of analysis passes. It builds a dependency graph and runs passes in topological order,
//! ensuring that each pass's dependencies are satisfied before it runs.
//!
//! # Example
//!
//! ```
//! use hir_analysis::pipeline::AnalysisPipeline;
//! use hir_analysis::pass::AnalysisPass;
//! use hir::body::Body;
//! use std::sync::Arc;
//!
//! // Create a new pipeline
//! let mut pipeline = AnalysisPipeline::new();
//!
//! // Register passes (assuming MyPass1 and MyPass2 implement AnalysisPass)
//! // pipeline.register::<MyPass1>().unwrap();
//! // pipeline.register::<MyPass2>().unwrap();
//!
//! // Run the analysis on a body
//! // let body = Arc::new(Body::new(...));
//! // let context = pipeline.analyze(body).unwrap();
//! ```

use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;

use petgraph::algo::toposort;
use petgraph::graph::{DiGraph, NodeIndex};
use tracing::{debug, error, info, instrument, warn};

use crate::context::AnalysisContext;
use crate::error::AnalysisError;
use crate::export::{ExportFormat, ExportOptions, PipelineExporter};
use crate::pass::AnalysisPass;

/// Manages the registration and execution of analysis passes.
/// Builds a dependency graph and runs passes in topological order.
pub struct AnalysisPipeline {
    /// Stores registered pass runners, keyed by their `TypeId`.
    passes: HashMap<TypeId, Box<dyn ErasedPassRunner>>,
    /// Maps TypeId to NodeIndex for graph operations.
    pass_nodes: HashMap<TypeId, NodeIndex>,
    /// The dependency graph. Node weight is TypeId.
    graph: DiGraph<TypeId, ()>,
}

impl AnalysisPipeline {
    /// Creates a new, empty `AnalysisPipeline`.
    ///
    /// Returns an `AnalysisPipeline` with no registered passes.
    ///
    /// # Examples
    ///
    /// ```
    /// use hir_analysis::pipeline::AnalysisPipeline;
    ///
    /// let pipeline = AnalysisPipeline::new();
    /// ```
    #[instrument]
    pub fn new() -> Self {
        debug!("Creating new AnalysisPipeline");
        Self { passes: HashMap::new(), pass_nodes: HashMap::new(), graph: DiGraph::new() }
    }

    /// Registers an analysis pass using its default implementation.
    ///
    /// This is a convenience method that creates a default instance of the pass
    /// and registers it with the pipeline.
    ///
    /// # Type Parameters
    ///
    /// * `P` - The type of the analysis pass to register, which must implement
    ///   `AnalysisPass`, `Default`, and have a `'static` lifetime.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the pass was registered successfully.
    /// * `Err(AnalysisError)` if the pass could not be registered, for example
    ///   if it was already registered or if one of its dependencies is not registered.
    ///
    /// # Examples
    ///
    /// ```
    /// use hir_analysis::pipeline::AnalysisPipeline;
    /// // Assuming MyPass implements AnalysisPass + Default
    /// // let mut pipeline = AnalysisPipeline::new();
    /// // pipeline.register::<MyPass>().unwrap();
    /// ```
    #[instrument(skip_all, fields(pass_type = std::any::type_name::<P>()))]
    pub fn register<P>(&mut self) -> Result<(), AnalysisError>
    where
        P: AnalysisPass + Default + 'static,
    {
        debug!("Registering pass using default implementation");
        self.register_pass(P::default())
    }

    /// Registers a specific instance of an analysis pass.
    ///
    /// This method registers a specific instance of an analysis pass with the pipeline.
    /// It adds the pass to the dependency graph and ensures that all of its dependencies
    /// are already registered.
    ///
    /// # Type Parameters
    ///
    /// * `P` - The type of the analysis pass to register, which must implement
    ///   `AnalysisPass` and have a `'static` lifetime.
    ///
    /// # Parameters
    ///
    /// * `pass_instance` - The instance of the analysis pass to register.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the pass was registered successfully.
    /// * `Err(AnalysisError::PassAlreadyRegistered)` if a pass of the same type is already registered.
    /// * `Err(AnalysisError::PassNotRegistered)` if one of the pass's dependencies is not registered.
    ///
    /// # Examples
    ///
    /// ```
    /// use hir_analysis::pipeline::AnalysisPipeline;
    /// // Assuming MyPass implements AnalysisPass
    /// // let mut pipeline = AnalysisPipeline::new();
    /// // let my_pass = MyPass::new();
    /// // pipeline.register_pass(my_pass).unwrap();
    /// ```
    #[instrument(skip(self, pass_instance), fields(pass_name = pass_instance.name(), pass_type = std::any::type_name::<P>()))]
    pub fn register_pass<P>(&mut self, pass_instance: P) -> Result<(), AnalysisError>
    where
        P: AnalysisPass + 'static,
    {
        let pass_id = TypeId::of::<P>();
        let pass_name = pass_instance.name();
        let dependencies = pass_instance.dependencies();

        debug!(?pass_id, "Registering pass instance");

        if self.passes.contains_key(&pass_id) {
            warn!("Pass already registered");
            return Err(AnalysisError::PassAlreadyRegistered {
                pass_name: pass_name.to_string(),
                pass_id,
            });
        }

        // Add node to graph *before* looking up dependencies
        let node_index = self.graph.add_node(pass_id);
        self.pass_nodes.insert(pass_id, node_index);
        debug!(?node_index, "Added node to dependency graph");

        // Add edges for dependencies
        debug!(dependency_count = dependencies.len(), "Adding dependency edges");
        for dep_id in &dependencies {
            if let Some(dep_node_index) = self.pass_nodes.get(dep_id) {
                // Add an edge from dependency *to* current pass (B depends on A -> A -> B)
                self.graph.add_edge(*dep_node_index, node_index, ());
                debug!(?dep_id, ?dep_node_index, "Added dependency edge");
            } else {
                // Dependency not registered yet. Remove the node we just added.
                debug!(?dep_id, "Dependency not registered, removing pass node");
                self.graph.remove_node(node_index);
                self.pass_nodes.remove(&pass_id);
                return Err(AnalysisError::PassNotRegistered {
                    dependency_name: format!(
                        "Unknown (TypeId: {:?}) needed by {}",
                        dep_id, pass_name
                    ),
                    dependency_id: *dep_id,
                });
            }
        }

        // Store the runner trait object
        let runner: Box<dyn ErasedPassRunner> = Box::new(pass_instance);
        self.passes.insert(pass_id, runner);
        debug!("Pass registered successfully");

        Ok(())
    }

    /// Runs all registered analysis passes on the given HIR body.
    ///
    /// Passes are executed in topological order based on their declared dependencies.
    /// This ensures that each pass has access to the results of its dependencies.
    ///
    /// # Parameters
    ///
    /// * `body` - The HIR body to analyze.
    ///
    /// # Returns
    ///
    /// * `Ok(AnalysisContext)` containing the results of all analysis passes.
    /// * `Err(AnalysisError)` if any error occurred during analysis.
    ///
    /// # Errors
    ///
    /// * `AnalysisError::DependencyCycle` if the dependency graph contains a cycle.
    /// * `AnalysisError::PassFailed` if any pass returns an error during execution.
    /// * Other `AnalysisError` variants for internal issues.
    ///
    /// # Examples
    ///
    /// ```
    /// use hir_analysis::pipeline::AnalysisPipeline;
    /// use hir::body::Body;
    /// use std::sync::Arc;
    ///
    /// // let mut pipeline = AnalysisPipeline::new();
    /// // pipeline.register::<MyPass>().unwrap();
    /// // let body = Arc::new(Body::new(...));
    /// // let context = pipeline.analyze(body).unwrap();
    /// // let result = context.get_result::<MyPass>().unwrap();
    /// ```
    #[instrument(skip(self, body))]
    pub fn analyze(&self, body: Arc<hir::body::Body>) -> Result<AnalysisContext, AnalysisError> {
        info!("Starting analysis run");
        let mut context = AnalysisContext::new(body);

        let sorted_nodes = toposort(&self.graph, None).map_err(|cycle| {
            let node_id = cycle.node_id();
            let type_id = self.graph.node_weight(node_id).cloned();
            error!(?node_id, ?type_id, "Dependency cycle detected in analysis passes");
            AnalysisError::DependencyCycle(format!(
                "Cycle detected involving node index {:?} (TypeId: {:?})",
                node_id, type_id
            ))
        })?;

        info!(pass_count = sorted_nodes.len(), "Executing passes in topological order");
        for node_index in sorted_nodes {
            let pass_id = self.graph[node_index];
            let runner = self
                .passes
                .get(&pass_id)
                .expect("Graph node TypeId should exist in passes map (internal error)");

            info!(pass = runner.name(), "Executing analysis pass");
            match runner.run_pass(&mut context) {
                Ok(_) => debug!(pass = runner.name(), "Pass completed successfully"),
                Err(e) => {
                    error!(pass = runner.name(), error = ?e, "Pass failed");
                    return Err(e);
                }
            }
        }

        info!("Analysis run finished successfully");
        Ok(context)
    }
}

// Default implementation for AnalysisPipeline

impl Default for AnalysisPipeline {
    /// Creates a new, empty `AnalysisPipeline` using the `new` method.
    ///
    /// This is equivalent to calling `AnalysisPipeline::new()`.
    #[instrument]
    fn default() -> Self {
        debug!("Creating default AnalysisPipeline");
        Self::new()
    }
}

impl AnalysisPipeline {
    /// Returns a reference to the dependency graph.
    ///
    /// This method provides access to the internal dependency graph of analysis passes.
    ///
    /// # Returns
    ///
    /// A reference to the dependency graph.
    pub fn dependency_graph(&self) -> &DiGraph<TypeId, ()> {
        &self.graph
    }

    /// Returns a reference to the pass nodes map.
    ///
    /// This method provides access to the internal map from TypeId to NodeIndex.
    ///
    /// # Returns
    ///
    /// A reference to the pass nodes map.
    pub fn pass_nodes(&self) -> &HashMap<TypeId, NodeIndex> {
        &self.pass_nodes
    }

    /// Returns a map of pass names.
    ///
    /// This method collects the names of all registered passes.
    ///
    /// # Returns
    ///
    /// A map from TypeId to pass name.
    pub fn pass_names(&self) -> HashMap<TypeId, &'static str> {
        let mut pass_names = HashMap::new();
        for (&pass_id, runner) in &self.passes {
            pass_names.insert(pass_id, runner.name());
        }
        pass_names
    }

    /// Exports the dependency graph of analysis passes.
    ///
    /// This method generates an export of the dependency graph in the specified format.
    /// The dependency graph shows the relationships between analysis passes, where an edge
    /// from pass A to pass B indicates that B depends on A.
    ///
    /// # Parameters
    ///
    /// * `format` - The format to generate.
    /// * `options` - Options for customizing the export.
    ///
    /// # Returns
    ///
    /// A string containing the export in the specified format.
    ///
    /// # Examples
    ///
    /// ```
    /// use hir_analysis::pipeline::AnalysisPipeline;
    /// use hir_analysis::export::{ExportFormat, ExportOptions};
    ///
    /// let pipeline = AnalysisPipeline::new();
    /// // Generate a DOT export of the dependency graph
    /// let dot = pipeline.export_dependency_graph(ExportFormat::Dot, &Default::default());
    /// ```
    #[instrument(skip(self, options))]
    pub fn export_dependency_graph(&self, format: ExportFormat, options: &ExportOptions) -> String {
        debug!("Exporting dependency graph in {} format", format);

        let pass_names = self.pass_names();
        let exporter = PipelineExporter::new(&self.graph, &self.pass_nodes, pass_names);
        exporter.export_dependency_graph(format, options)
    }

    /// Exports the execution order of analysis passes.
    ///
    /// This method generates an export of the execution order in the specified format.
    /// The execution order is determined by topologically sorting the dependency graph.
    ///
    /// # Parameters
    ///
    /// * `format` - The format to generate.
    /// * `options` - Options for customizing the export.
    ///
    /// # Returns
    ///
    /// * `Ok(String)` containing the export in the specified format.
    /// * `Err(AnalysisError::DependencyCycle)` if the dependency graph contains a cycle.
    ///
    /// # Examples
    ///
    /// ```
    /// use hir_analysis::pipeline::AnalysisPipeline;
    /// use hir_analysis::export::{ExportFormat, ExportOptions};
    ///
    /// let pipeline = AnalysisPipeline::new();
    /// // Generate a Mermaid export of the execution order
    /// let mermaid = pipeline.export_execution_order(ExportFormat::Mermaid, &Default::default()).unwrap();
    /// ```
    #[instrument(skip(self, options))]
    pub fn export_execution_order(
        &self,
        format: ExportFormat,
        options: &ExportOptions,
    ) -> Result<String, AnalysisError> {
        debug!("Exporting execution order in {} format", format);

        // Perform topological sort to get execution order
        let sorted_nodes = toposort(&self.graph, None).map_err(|cycle| {
            let node_id = cycle.node_id();
            let type_id = self.graph.node_weight(node_id).cloned();
            error!(?node_id, ?type_id, "Dependency cycle detected in analysis passes");
            AnalysisError::DependencyCycle(format!(
                "Cycle detected involving node index {:?} (TypeId: {:?})",
                node_id, type_id
            ))
        })?;

        // Create a new graph with the execution order
        let mut order_graph = DiGraph::new();
        let mut pass_node_map = HashMap::new();
        let mut prev_node = None;

        let pass_names = self.pass_names();

        // Add nodes in execution order
        for node_index in &sorted_nodes {
            let pass_id = self.graph[*node_index];
            let new_idx = order_graph.add_node(pass_id);
            pass_node_map.insert(pass_id, new_idx);

            // Add edge from previous node to current node
            if let Some(prev) = prev_node {
                order_graph.add_edge(prev, new_idx, ());
            }

            prev_node = Some(new_idx);
        }

        let exporter = PipelineExporter::new(&order_graph, &pass_node_map, pass_names);
        Ok(exporter.export_dependency_graph(format, options))
    }
}

/// Type-erased trait object for running analysis passes.
///
/// This trait is used internally by the `AnalysisPipeline` to store and run
/// analysis passes of different types in a uniform way.
#[allow(dead_code)]
trait ErasedPassRunner: Send + Sync {
    /// Runs the analysis pass on the given context.
    ///
    /// # Parameters
    ///
    /// * `ctx` - The analysis context to run the pass on.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the pass ran successfully.
    /// * `Err(AnalysisError)` if the pass failed.
    fn run_pass(&self, ctx: &mut AnalysisContext) -> Result<(), AnalysisError>;

    /// Returns the name of the analysis pass.
    fn name(&self) -> &'static str;

    /// Returns the dependencies of the analysis pass.
    fn dependencies(&self) -> Vec<TypeId>;

    /// Returns the type ID of the analysis pass.
    fn type_id(&self) -> TypeId;
}

impl<P: AnalysisPass + 'static> ErasedPassRunner for P {
    #[instrument(skip(self, ctx), fields(pass_name = self.name()))]
    fn run_pass(&self, ctx: &mut AnalysisContext) -> Result<(), AnalysisError> {
        debug!("Running pass");
        match self.run(ctx) {
            Ok(output) => {
                debug!("Pass produced output, inserting into context");
                ctx.insert_result::<P>(output)?;
                debug!("Pass finished successfully");
                Ok(())
            }
            Err(source_err) => {
                error!(error = ?source_err, "Pass failed");
                Err(AnalysisError::PassFailed {
                    pass_name: self.name().to_string(),
                    source: source_err,
                })
            }
        }
    }

    fn name(&self) -> &'static str {
        P::name(self)
    }

    fn dependencies(&self) -> Vec<TypeId> {
        P::dependencies(self)
    }

    fn type_id(&self) -> TypeId {
        TypeId::of::<P>()
    }
}
