//! Base visitor traits and infrastructure for HIR traversal
//!
//! This module provides a comprehensive visitor pattern implementation for traversing
//! and analyzing HIR (High-level Intermediate Representation) structures. The visitor
//! pattern allows for separation of algorithms from the data structures they operate on,
//! enabling clean, modular code that can be easily extended.
//!
//! # Design
//!
//! The visitor system is designed around several key components:
//!
//! - **Visitor Traits**: Define the interface for traversing different node types
//! - **Visit Results**: Control the traversal flow
//! - **Visitor Context**: Provides access to analysis data and configuration
//! - **Traversal Strategies**: Support for different traversal orders and patterns
//!
//! # Examples
//!
//! ```
//! use hir_analysis::visitors::{Visitor, VisitorContext, VisitResult};
//! use hir_analysis::AnalysisContext;
//! use hir::body::{Body, Instruction};
//!
//! struct MyVisitor;
//!
//! impl Visitor for MyVisitor {
//!     fn visit_instruction(&mut self, ctx: &mut VisitorContext, instr: &Instruction) -> VisitResult {
//!         println!("Visiting instruction: {}", instr.opcode);
//!         VisitResult::Continue
//!     }
//! }
//!
//! // Usage:
//! // let mut ctx = AnalysisContext::new(db, body);
//! // let mut visitor_ctx = VisitorContext::new(&mut ctx);
//! // let mut visitor = MyVisitor;
//! // visitor.visit_body(&mut visitor_ctx, body);
//! ```

use std::marker::PhantomData;

use hir::body::{Body, Expr, Instruction};
use hir::expr::ExprId;
use miette::Severity;
use ram_diagnostics::{Diagnostic, DiagnosticCollection};

/// Result of visiting a node, controlling traversal flow
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisitResult {
    /// Continue traversal normally, visiting children
    Continue,

    /// Skip visiting children of the current node
    SkipChildren,

    /// Skip the current node's siblings (remaining nodes at the same level)
    SkipSiblings,

    /// Stop traversal entirely
    Stop,

    /// Skip the current node but continue with its children
    SkipSelf,
}

impl VisitResult {
    /// Returns true if traversal should continue to children
    #[inline]
    pub fn should_visit_children(&self) -> bool {
        matches!(self, Self::Continue | Self::SkipSelf)
    }

    /// Returns true if traversal should stop entirely
    #[inline]
    pub fn should_stop(&self) -> bool {
        matches!(self, Self::Stop)
    }

    /// Returns true if traversal should skip siblings
    #[inline]
    pub fn should_skip_siblings(&self) -> bool {
        matches!(self, Self::SkipSiblings | Self::Stop)
    }
}

/// Configuration for visitor traversal
#[derive(Debug, Clone)]
pub struct VisitorConfig {
    /// Maximum depth to traverse
    pub max_depth: Option<usize>,

    /// Whether to collect diagnostics during traversal
    pub collect_diagnostics: bool,

    /// Whether to traverse in pre-order (visit node before children)
    pub pre_order: bool,

    /// Whether to traverse in post-order (visit node after children)
    pub post_order: bool,
}

impl Default for VisitorConfig {
    fn default() -> Self {
        Self { max_depth: None, collect_diagnostics: true, pre_order: true, post_order: false }
    }
}

/// Context for visitors, providing access to analysis data and configuration
pub struct VisitorContext<'a, 'db, 'body> {
    /// Analysis context
    pub analysis_ctx: &'a mut crate::AnalysisContext<'db, 'body>,

    /// Current traversal depth
    pub depth: usize,

    /// Configuration for the visitor
    pub config: VisitorConfig,

    /// Diagnostics collected during traversal
    pub diagnostics: DiagnosticCollection,
}

impl<'a, 'db, 'body> VisitorContext<'a, 'db, 'body> {
    /// Create a new visitor context with default configuration
    pub fn new(analysis_ctx: &'a mut crate::AnalysisContext<'db, 'body>) -> Self {
        Self {
            analysis_ctx,
            depth: 0,
            config: VisitorConfig::default(),
            diagnostics: DiagnosticCollection::new(),
        }
    }

    /// Create a new visitor context with custom configuration
    pub fn with_config(
        analysis_ctx: &'a mut crate::AnalysisContext<'db, 'body>,
        config: VisitorConfig,
    ) -> Self {
        Self { analysis_ctx, depth: 0, config, diagnostics: DiagnosticCollection::new() }
    }

    /// Get the body being analyzed
    #[inline]
    pub fn body(&self) -> &Body {
        self.analysis_ctx.body()
    }

    /// Get a reference to the database
    #[inline]
    pub fn db(&self) -> &'db dyn crate::AnalysisDatabase {
        self.analysis_ctx.db()
    }

    /// Add a diagnostic
    pub fn add_diagnostic(&mut self, diagnostic: Diagnostic) {
        if self.config.collect_diagnostics {
            self.diagnostics.add(diagnostic);
        }
    }

    /// Add an error diagnostic
    pub fn error(&mut self, message: impl Into<String>, _span: Option<()>) {
        if self.config.collect_diagnostics {
            self.diagnostics.error(message, None, miette::Severity::Error);
        }
    }

    /// Add a warning diagnostic
    pub fn warning(&mut self, message: impl Into<String>, _span: Option<()>) {
        if self.config.collect_diagnostics {
            self.diagnostics.warning(message, None, miette::Severity::Warning);
        }
    }

    /// Add an info diagnostic
    pub fn info(&mut self, message: impl Into<String>, _span: Option<()>) {
        if self.config.collect_diagnostics {
            self.diagnostics.info(message, None, miette::Severity::Advice);
        }
    }

    /// Enter a new scope, increasing the depth
    #[inline]
    pub fn enter_scope(&mut self) -> bool {
        self.depth += 1;
        if let Some(max_depth) = self.config.max_depth { self.depth <= max_depth } else { true }
    }

    /// Exit the current scope, decreasing the depth
    #[inline]
    pub fn exit_scope(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }

    /// Check if we should continue traversal based on depth
    #[inline]
    pub fn should_continue(&self) -> bool {
        if let Some(max_depth) = self.config.max_depth { self.depth <= max_depth } else { true }
    }
}

/// Core visitor trait for traversing HIR nodes
///
/// This trait defines the interface for visiting different types of HIR nodes.
/// Implementors can override specific visit methods to customize behavior.
pub trait Visitor: Sized {
    /// Visit an instruction
    fn visit_instruction(
        &mut self,
        ctx: &mut VisitorContext,
        instruction: &Instruction,
    ) -> VisitResult {
        // Default implementation visits the operand
        if let Some(operand) = instruction.operand {
            if ctx.config.pre_order {
                let result = self.visit_expr(ctx, operand);
                if result.should_stop() {
                    return result;
                }
            }
        }

        VisitResult::Continue
    }

    /// Visit an expression
    fn visit_expr(&mut self, ctx: &mut VisitorContext, expr_id: ExprId) -> VisitResult {
        let body = ctx.analysis_ctx.body();

        if let Some(expr) = body.expr(expr_id) {
            if !ctx.enter_scope() {
                return VisitResult::SkipChildren;
            }

            let result = match expr {
                Expr::Literal(_) => self.visit_literal(ctx, expr_id),
                Expr::Name(_) => self.visit_name(ctx, expr_id),
                // Add other expression types as needed
                _ => VisitResult::Continue,
            };

            ctx.exit_scope();
            result
        } else {
            VisitResult::Continue
        }
    }

    /// Visit a literal expression
    fn visit_literal(&mut self, _ctx: &mut VisitorContext, _expr_id: ExprId) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit a name expression
    fn visit_name(&mut self, _ctx: &mut VisitorContext, _expr_id: ExprId) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit a body
    fn visit_body(&mut self, ctx: &mut VisitorContext, body: &Body) {
        // Visit all instructions
        for instruction in &body.instructions {
            if ctx.config.pre_order {
                let result = self.visit_instruction(ctx, instruction);

                if result.should_stop() {
                    return;
                }

                if result.should_skip_siblings() {
                    break;
                }
            }

            // Visit children if needed
            if ctx.config.post_order {
                let result = self.visit_instruction(ctx, instruction);

                if result.should_stop() {
                    return;
                }

                if result.should_skip_siblings() {
                    break;
                }
            }
        }
    }

    /// Run this visitor on a body
    fn run(mut self, analysis_ctx: &mut crate::AnalysisContext) -> DiagnosticCollection {
        let mut ctx = VisitorContext::new(analysis_ctx);
        self.visit_body(&mut ctx, analysis_ctx.body());
        ctx.diagnostics
    }

    /// Run this visitor with a custom configuration
    fn run_with_config(
        mut self,
        analysis_ctx: &mut crate::AnalysisContext,
        config: VisitorConfig,
    ) -> DiagnosticCollection {
        let mut ctx = VisitorContext::with_config(analysis_ctx, config);
        self.visit_body(&mut ctx, analysis_ctx.body());
        ctx.diagnostics
    }
}

/// A visitor that can be chained with another visitor
pub struct ChainVisitor<V1, V2> {
    first: V1,
    second: V2,
}

impl<V1, V2> ChainVisitor<V1, V2> {
    /// Create a new chain visitor
    pub fn new(first: V1, second: V2) -> Self {
        Self { first, second }
    }
}

impl<V1: Visitor, V2: Visitor> Visitor for ChainVisitor<V1, V2> {
    fn visit_instruction(
        &mut self,
        ctx: &mut VisitorContext,
        instruction: &Instruction,
    ) -> VisitResult {
        let result1 = self.first.visit_instruction(ctx, instruction);
        if result1.should_stop() {
            return result1;
        }

        let result2 = self.second.visit_instruction(ctx, instruction);
        if result1.should_skip_children() || result2.should_skip_children() {
            VisitResult::SkipChildren
        } else if result1.should_skip_siblings() || result2.should_skip_siblings() {
            VisitResult::SkipSiblings
        } else {
            VisitResult::Continue
        }
    }

    fn visit_expr(&mut self, ctx: &mut VisitorContext, expr_id: ExprId) -> VisitResult {
        let result1 = self.first.visit_expr(ctx, expr_id);
        if result1.should_stop() {
            return result1;
        }

        let result2 = self.second.visit_expr(ctx, expr_id);
        if result1.should_skip_children() || result2.should_skip_children() {
            VisitResult::SkipChildren
        } else if result1.should_skip_siblings() || result2.should_skip_siblings() {
            VisitResult::SkipSiblings
        } else {
            VisitResult::Continue
        }
    }
}

/// Extension trait for visitors
pub trait VisitorExt: Visitor + Sized {
    /// Chain this visitor with another visitor
    fn chain<V: Visitor>(self, other: V) -> ChainVisitor<Self, V> {
        ChainVisitor::new(self, other)
    }

    /// Run this visitor with a filter
    fn with_filter<F>(self, filter: F) -> FilterVisitor<Self, F>
    where
        F: Fn(&Instruction) -> bool,
    {
        FilterVisitor::new(self, filter)
    }
}

impl<T: Visitor> VisitorExt for T {}

/// A visitor that filters nodes based on a predicate
pub struct FilterVisitor<V, F> {
    visitor: V,
    filter: F,
}

impl<V, F> FilterVisitor<V, F>
where
    F: Fn(&Instruction) -> bool,
{
    /// Create a new filter visitor
    pub fn new(visitor: V, filter: F) -> Self {
        Self { visitor, filter }
    }
}

impl<V: Visitor, F: Fn(&Instruction) -> bool> Visitor for FilterVisitor<V, F> {
    fn visit_instruction(
        &mut self,
        ctx: &mut VisitorContext,
        instruction: &Instruction,
    ) -> VisitResult {
        if (self.filter)(instruction) {
            self.visitor.visit_instruction(ctx, instruction)
        } else {
            VisitResult::SkipChildren
        }
    }

    fn visit_expr(&mut self, ctx: &mut VisitorContext, expr_id: ExprId) -> VisitResult {
        self.visitor.visit_expr(ctx, expr_id)
    }
}

/// A visitor that collects nodes matching a predicate
pub struct CollectVisitor<T, F> {
    collected: Vec<T>,
    filter: F,
    _marker: PhantomData<T>,
}

impl<T, F> CollectVisitor<T, F>
where
    F: Fn(&Instruction) -> Option<T>,
    T: Clone,
{
    /// Create a new collect visitor
    pub fn new(filter: F) -> Self {
        Self { collected: Vec::new(), filter, _marker: PhantomData }
    }

    /// Get the collected items
    pub fn items(&self) -> &[T] {
        &self.collected
    }

    /// Take ownership of the collected items
    pub fn take_items(self) -> Vec<T> {
        self.collected
    }
}

impl<T: Clone, F: Fn(&Instruction) -> Option<T>> Visitor for CollectVisitor<T, F> {
    fn visit_instruction(
        &mut self,
        _ctx: &mut VisitorContext,
        instruction: &Instruction,
    ) -> VisitResult {
        if let Some(item) = (self.filter)(instruction) {
            self.collected.push(item);
        }
        VisitResult::Continue
    }
}
