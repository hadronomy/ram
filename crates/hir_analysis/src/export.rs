//! Export utilities for the analysis pipeline.
//!
//! This module provides utilities for exporting the analysis pipeline data,
//! including the dependency graph of analysis passes and the execution order.
//!
//! # Example
//!
//! ```
//! use hir_analysis::pipeline::AnalysisPipeline;
//! use hir_analysis::export::{ExportFormat, ExportOptions};
//!
//! let pipeline = AnalysisPipeline::new();
//! // Generate a DOT export of the dependency graph
//! let dot = pipeline.export_dependency_graph(ExportFormat::Dot, &Default::default());
//! ```

use std::any::TypeId;
use std::collections::HashMap;
use std::fmt;

use petgraph::dot::{Config, Dot};
use petgraph::graph::{DiGraph, NodeIndex};
use serde_json::{self, Map, Value};

/// Supported export formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// DOT format for use with Graphviz.
    Dot,
    /// Mermaid format for embedding in Markdown.
    Mermaid,
    /// JSON format for programmatic consumption.
    Json,
}

impl fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExportFormat::Dot => write!(f, "dot"),
            ExportFormat::Mermaid => write!(f, "mermaid"),
            ExportFormat::Json => write!(f, "json"),
        }
    }
}

/// Options for customizing exports.
#[derive(Debug, Clone)]
pub struct ExportOptions {
    /// Whether to include pass names in the export.
    pub include_pass_names: bool,
    /// Whether to include pass type IDs in the export.
    pub include_type_ids: bool,
    /// Whether to include edge labels in the export.
    pub include_edge_labels: bool,
    /// Whether to use a compact representation.
    pub compact: bool,
    /// Custom node labels, keyed by TypeId.
    pub node_labels: HashMap<TypeId, String>,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            include_pass_names: true,
            include_type_ids: false,
            include_edge_labels: false,
            compact: false,
            node_labels: HashMap::new(),
        }
    }
}

/// Export utilities for the analysis pipeline.
pub struct PipelineExporter<'a> {
    /// The dependency graph to export.
    graph: &'a DiGraph<TypeId, ()>,
    /// Maps TypeId to NodeIndex for graph operations.
    pass_nodes: &'a HashMap<TypeId, NodeIndex>,
    /// Maps TypeId to pass name.
    pass_names: HashMap<TypeId, &'static str>,
}

impl<'a> PipelineExporter<'a> {
    /// Creates a new `PipelineExporter` for the given graph and pass nodes.
    ///
    /// # Parameters
    ///
    /// * `graph` - The dependency graph to export.
    /// * `pass_nodes` - Maps TypeId to NodeIndex for graph operations.
    /// * `pass_names` - Maps TypeId to pass name.
    pub fn new(
        graph: &'a DiGraph<TypeId, ()>,
        pass_nodes: &'a HashMap<TypeId, NodeIndex>,
        pass_names: HashMap<TypeId, &'static str>,
    ) -> Self {
        Self { graph, pass_nodes, pass_names }
    }

    /// Generates an export of the dependency graph in the specified format.
    ///
    /// # Parameters
    ///
    /// * `format` - The format to generate.
    /// * `options` - Options for customizing the export.
    ///
    /// # Returns
    ///
    /// A string containing the export in the specified format.
    pub fn export_dependency_graph(&self, format: ExportFormat, options: &ExportOptions) -> String {
        match format {
            ExportFormat::Dot => self.to_dot(options),
            ExportFormat::Mermaid => self.to_mermaid(options),
            ExportFormat::Json => self.to_json(options),
        }
    }

    /// Generates a DOT representation of the dependency graph.
    ///
    /// # Parameters
    ///
    /// * `options` - Options for customizing the export.
    ///
    /// # Returns
    ///
    /// A string containing the DOT representation.
    fn to_dot(&self, options: &ExportOptions) -> String {
        let mut dot_config = vec![];

        if !options.include_edge_labels {
            dot_config.push(Config::EdgeNoLabel);
        }

        if options.compact {
            dot_config.push(Config::NodeNoLabel);
        }

        // Create a labeled graph for export
        let mut labeled_graph = DiGraph::new();
        let mut node_map = HashMap::new();

        // Add nodes with labels
        for (&type_id, &node_idx) in self.pass_nodes {
            let label = if let Some(custom_label) = options.node_labels.get(&type_id) {
                custom_label.clone()
            } else {
                let mut label = String::new();

                if options.include_pass_names {
                    if let Some(&name) = self.pass_names.get(&type_id) {
                        label.push_str(name);
                    } else {
                        label.push_str("Unknown");
                    }
                }

                if options.include_type_ids {
                    if !label.is_empty() {
                        label.push('\n');
                    }
                    label.push_str(&format!("{:?}", type_id));
                }

                label
            };

            let new_idx = labeled_graph.add_node(label);
            node_map.insert(node_idx, new_idx);
        }

        // Add edges
        for edge in self.graph.edge_indices() {
            let (source, target) = self.graph.edge_endpoints(edge).unwrap();
            if let (Some(&source_idx), Some(&target_idx)) =
                (node_map.get(&source), node_map.get(&target))
            {
                labeled_graph.add_edge(source_idx, target_idx, ());
            }
        }

        format!("{:?}", Dot::with_config(&labeled_graph, &dot_config))
    }

    /// Generates a Mermaid representation of the dependency graph.
    ///
    /// # Parameters
    ///
    /// * `options` - Options for customizing the export.
    ///
    /// # Returns
    ///
    /// A string containing the Mermaid representation.
    fn to_mermaid(&self, options: &ExportOptions) -> String {
        let mut result = String::from("graph TD\n");

        // Add nodes
        for (&type_id, &node_idx) in self.pass_nodes {
            let node_id = format!("N{}", node_idx.index());

            let label = if let Some(custom_label) = options.node_labels.get(&type_id) {
                custom_label.clone()
            } else {
                let mut label = String::new();

                if options.include_pass_names {
                    if let Some(&name) = self.pass_names.get(&type_id) {
                        label.push_str(name);
                    } else {
                        label.push_str("Unknown");
                    }
                }

                if options.include_type_ids {
                    if !label.is_empty() {
                        label.push_str("<br>");
                    }
                    label.push_str(&format!("{:?}", type_id));
                }

                label
            };

            if options.compact {
                result.push_str(&format!("    {}\n", node_id));
            } else {
                result.push_str(&format!("    {}[\"{}\"]\n", node_id, label));
            }
        }

        // Add edges
        for edge in self.graph.edge_indices() {
            let (source, target) = self.graph.edge_endpoints(edge).unwrap();
            let source_id = format!("N{}", source.index());
            let target_id = format!("N{}", target.index());

            result.push_str(&format!("    {} --> {}\n", source_id, target_id));
        }

        result
    }

    /// Generates a JSON representation of the dependency graph.
    ///
    /// # Parameters
    ///
    /// * `options` - Options for customizing the export.
    ///
    /// # Returns
    ///
    /// A string containing the JSON representation.
    fn to_json(&self, options: &ExportOptions) -> String {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        // Add nodes
        for (&type_id, &node_idx) in self.pass_nodes {
            let mut node = Map::new();
            node.insert("id".to_string(), Value::String(format!("N{}", node_idx.index())));

            if options.include_pass_names {
                if let Some(&name) = self.pass_names.get(&type_id) {
                    node.insert("name".to_string(), Value::String(name.to_string()));
                }
            }

            if options.include_type_ids {
                node.insert("type_id".to_string(), Value::String(format!("{:?}", type_id)));
            }

            if let Some(custom_label) = options.node_labels.get(&type_id) {
                node.insert("label".to_string(), Value::String(custom_label.clone()));
            }

            nodes.push(Value::Object(node));
        }

        // Add edges
        for edge in self.graph.edge_indices() {
            let (source, target) = self.graph.edge_endpoints(edge).unwrap();

            let mut edge_obj = Map::new();
            edge_obj.insert("source".to_string(), Value::String(format!("N{}", source.index())));
            edge_obj.insert("target".to_string(), Value::String(format!("N{}", target.index())));

            edges.push(Value::Object(edge_obj));
        }

        // Create the final JSON object
        let mut json = Map::new();
        json.insert("nodes".to_string(), Value::Array(nodes));
        json.insert("edges".to_string(), Value::Array(edges));

        serde_json::to_string_pretty(&Value::Object(json)).unwrap_or_else(|_| "{}".to_string())
    }
}
