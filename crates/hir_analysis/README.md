# HIR Analysis

A modular and ergonomic semantic analysis framework for the RAM High-level Intermediate Representation (HIR).

## Overview

This crate provides a comprehensive set of tools for analyzing RAM programs at the HIR level. It includes:

- **Type checking**: Verify that operations have compatible types
- **Control flow analysis**: Build and analyze control flow graphs
- **Data flow analysis**: Track variable definitions and uses
- **Optimization analysis**: Identify optimization opportunities

## Architecture

The semantic analysis is built around a few key components:

- **Database**: Salsa-based incremental computation for caching analysis results
- **Visitors**: Traversal of the HIR for various analysis purposes
- **Diagnostics**: Collection and reporting of semantic errors and warnings (using the `ram_diagnostics` crate)
- **Analysis Passes**: Specific analysis algorithms that operate on the HIR

## Usage

### Basic Analysis

```rust
use hir_analysis::{Analysis, AnalysisDatabase};
use hir::ids::DefId;
use ram_diagnostics::DiagnosticKind;

fn analyze_program(db: &dyn AnalysisDatabase, def_id: DefId) {
    // Create an analysis instance
    let analysis = Analysis::new(db);

    // Analyze the body
    let results = analysis.analyze_body(def_id);

    // Check for errors
    if results.diagnostics.has_errors() {
        println!("Analysis found errors:");
        for diagnostic in results.diagnostics.diagnostics() {
            if diagnostic.kind == DiagnosticKind::Error {
                println!("  {}: {}", diagnostic.message, diagnostic.help);
            }
        }
    }

    // Use the analysis results
    if let Some(cfg) = &results.control_flow {
        println!("Control flow graph has {} blocks", cfg.blocks.len());
    }
}
```

### Querying Analysis Results

```rust
use hir_analysis::AnalysisDatabase;
use hir::ids::{DefId, LocalDefId};
use hir::ExprId;

fn query_analysis(db: &dyn AnalysisDatabase, def_id: DefId) {
    // Get the type of an expression
    let expr_id = ExprId(0);
    let type_id = db.expr_type(def_id, expr_id);

    // Check if an instruction is reachable
    let instr_id = LocalDefId(0);
    let is_reachable = db.is_instruction_reachable(def_id, instr_id);

    // Get all diagnostics
    let diagnostics = db.diagnostics(def_id);

    // Get optimization opportunities
    let optimizations = db.optimizations(def_id);
}
```

### Custom Analysis

```rust
use hir_analysis::visitors::{Visitor, VisitorContext, VisitResult};
use hir::body::Instruction;

struct MyVisitor;

impl Visitor for MyVisitor {
    fn visit_instruction(
        &self,
        ctx: &mut VisitorContext,
        instruction: &Instruction,
    ) -> VisitResult {
        // Custom analysis logic
        if instruction.opcode == "HALT" {
            ctx.analysis_ctx.diagnostics_mut().info(
                "Found HALT instruction",
                None,
            );
        }

        VisitResult::Continue
    }
}

fn run_custom_analysis(ctx: &mut AnalysisContext) {
    let visitor = MyVisitor;
    let mut visitor_ctx = VisitorContext { analysis_ctx: ctx };
    visitor.visit_body(&mut visitor_ctx, ctx.body());
}
```

## Features

### Type System

The type system is simple but extensible:

```rust
pub enum Type {
    Int,
    Address,
    Unknown,
    Error,
}
```

### Control Flow Analysis

Build and analyze control flow graphs:

```rust
let cfg = db.control_flow_graph(def_id);

// Check if an instruction is reachable
let is_reachable = cfg.is_instruction_reachable(instr_id);

// Find unreachable blocks
let unreachable = cfg.unreachable_blocks();
```

### Data Flow Analysis

Track variable definitions and uses:

```rust
let data_flow = db.data_flow_results(def_id);

// Check if a variable is defined at an instruction
let is_defined = data_flow.is_defined_at(instr_id, &var);

// Check if a variable is live before an instruction
let is_live = data_flow.is_live_before(instr_id, &var);
```

### Optimization Analysis

Identify optimization opportunities:

```rust
let optimizations = db.optimizations(def_id);

for opt in optimizations.iter() {
    match opt {
        Optimization::DeadCode { instruction_id, description } => {
            println!("Dead code: {}", description);
        },
        Optimization::ConstantPropagation { instruction_id, value, description } => {
            println!("Constant propagation: {}", description);
        },
        // ...
    }
}
```

## Extending the Framework

The framework is designed to be extensible:

- Add new validation rules by implementing the `ValidationRule` trait
- Create custom visitors by implementing the `Visitor` trait
- Add new analysis passes in the `analysis` module
- Extend the type system by adding new variants to the `Type` enum

## License

This project is licensed under the MIT License - see the LICENSE file for details.
