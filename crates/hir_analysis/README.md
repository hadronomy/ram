# HIR Analysis

This crate provides semantic analysis capabilities for the RAM High-level Intermediate Representation (HIR).

## Main Components

- `AnalysisContext` - Stores and provides access to analysis results and diagnostics
- `AnalysisPass` - Trait for implementing analysis passes
- `AnalysisPipeline` - Manages the registration and execution of analysis passes
- `AnalysisError` - Error types for the HIR analysis

## Diagnostic Collection

The `AnalysisContext` provides methods for collecting diagnostics during analysis:

```rust
// Add a diagnostic
ctx.add_diagnostic(Diagnostic::error("Error message", "Help message", 0..10));

// Add an error diagnostic
ctx.error("Error message", "Help message", Some(0..10));

// Add a warning diagnostic
ctx.warning("Warning message", "Help message", Some(10..20));

// Add an info diagnostic
ctx.info("Info message", "Help message", Some(20..30));

// Get all diagnostics
let diagnostics = ctx.diagnostics();

// Check if there are any error diagnostics
if ctx.has_errors() {
    // Handle errors
}
```

## Example Analysis Pass

Here's an example of an analysis pass that reports diagnostics:

```rust
struct MyPass;

impl AnalysisPass for MyPass {
    type Output = ();

    fn name(&self) -> &'static str {
        "MyPass"
    }

    fn dependencies(&self) -> Vec<TypeId> {
        vec![]
    }

    fn run(&self, ctx: &mut AnalysisContext) -> Result<Self::Output, Box<dyn Diagnostic>> {
        // Report an error diagnostic
        ctx.error(
            "This is an error message",
            "This is a help message for the error",
            Some(10..20),
        );
        
        // Report a warning diagnostic
        ctx.warning(
            "This is a warning message",
            "This is a help message for the warning",
            Some(30..40),
        );
        
        Ok(())
    }
}
```

## Visitor Pattern

The crate also provides a visitor pattern implementation for traversing HIR structures:

```rust
use hir_analysis::visitors::{Visitor, VisitorResult, walk_body};
use hir::body::{Body, Instruction};
use std::ops::ControlFlow;

// A visitor that counts the number of instructions
struct InstructionCounter {
    count: usize,
}

impl Visitor for InstructionCounter {
    type Result = usize;

    fn visit_instruction(&mut self, _instruction: &Instruction) -> VisitorResult<Self::Result> {
        self.count += 1;
        ControlFlow::Continue(())
    }

    fn finish(self) -> Self::Result {
        self.count
    }
}

// Usage
fn count_instructions(body: &Body) -> usize {
    let visitor = InstructionCounter { count: 0 };
    walk_body(visitor, body)
}
```
