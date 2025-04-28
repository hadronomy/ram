//! Semantic validation rules for RAM HIR
//!
//! This module provides validation rules for checking semantic correctness
//! of RAM programs beyond simple type checking.

use hir::ids::LocalDefId;
use hir::ExprId;
use hir::body::{Body, Instruction};

use ram_diagnostics::DiagnosticCollection;

/// Context for validation rules
pub struct ValidationContext<'a, 'db, 'body> {
    /// Analysis context
    pub analysis_ctx: &'a mut crate::AnalysisContext<'db, 'body>,
}

/// A semantic validation rule
pub trait ValidationRule {
    /// Name of the rule
    fn name(&self) -> &'static str;

    /// Description of the rule
    fn description(&self) -> &'static str;

    /// Check if the rule is applicable to the given instruction
    fn is_applicable(&self, instruction: &Instruction) -> bool;

    /// Validate an instruction
    fn validate_instruction(
        &self,
        ctx: &mut ValidationContext,
        instruction: &Instruction,
    );

    /// Validate an expression
    fn validate_expression(
        &self,
        ctx: &mut ValidationContext,
        expr_id: ExprId,
    );

    /// Validate the entire body
    fn validate_body(
        &self,
        ctx: &mut ValidationContext,
        body: &Body,
    );
}

/// A collection of validation rules
pub struct ValidationRules {
    rules: Vec<Box<dyn ValidationRule>>,
}

impl ValidationRules {
    /// Create a new empty collection of validation rules
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
        }
    }

    /// Add a validation rule
    pub fn add_rule<R: ValidationRule + 'static>(&mut self, rule: R) {
        self.rules.push(Box::new(rule));
    }

    /// Validate a body using all rules
    pub fn validate_body(
        &self,
        analysis_ctx: &mut crate::AnalysisContext,
        body: &Body,
    ) -> DiagnosticCollection {
        let mut ctx = ValidationContext {
            analysis_ctx,
        };

        // Run all rules on the body
        for rule in &self.rules {
            rule.validate_body(&mut ctx, body);
        }

        // Run all applicable rules on each instruction
        for instruction in &body.instructions {
            for rule in &self.rules {
                if rule.is_applicable(instruction) {
                    rule.validate_instruction(&mut ctx, instruction);
                }
            }
        }

        // Run all rules on each expression
        for (expr_id, _) in body.exprs.iter() {
            for rule in &self.rules {
                rule.validate_expression(&mut ctx, expr_id);
            }
        }

        ctx.analysis_ctx.diagnostics().clone()
    }
}

/// Rule for checking that all labels are defined
pub struct UndefinedLabelRule;

impl ValidationRule for UndefinedLabelRule {
    fn name(&self) -> &'static str {
        "undefined-label"
    }

    fn description(&self) -> &'static str {
        "Check that all labels used in jump instructions are defined"
    }

    fn is_applicable(&self, instruction: &Instruction) -> bool {
        // Only applicable to jump instructions
        instruction.opcode == "JUMP" || instruction.opcode == "JZERO" || instruction.opcode == "JGTZ"
    }

    fn validate_instruction(
        &self,
        ctx: &mut ValidationContext,
        instruction: &Instruction,
    ) {
        // Implementation would check if the jump target is a valid label
        // This is just a placeholder
        if let Some(operand) = instruction.operand {
            let body = ctx.analysis_ctx.body();
            if let Some(expr) = body.expr(operand) {
                // Check if the expression refers to a valid label
                // This would depend on the actual expression representation
            }
        }
    }

    fn validate_expression(
        &self,
        _ctx: &mut ValidationContext,
        _expr_id: ExprId,
    ) {
        // Not applicable to expressions
    }

    fn validate_body(
        &self,
        _ctx: &mut ValidationContext,
        _body: &Body,
    ) {
        // Not applicable to the entire body
    }
}

/// Rule for checking that all variables are initialized before use
pub struct UninitializedVariableRule;

impl ValidationRule for UninitializedVariableRule {
    fn name(&self) -> &'static str {
        "uninitialized-variable"
    }

    fn description(&self) -> &'static str {
        "Check that all variables are initialized before use"
    }

    fn is_applicable(&self, instruction: &Instruction) -> bool {
        // Applicable to LOAD instructions
        instruction.opcode == "LOAD"
    }

    fn validate_instruction(
        &self,
        ctx: &mut ValidationContext,
        instruction: &Instruction,
    ) {
        // Implementation would check if the variable being loaded has been initialized
        // This would require data flow analysis
        if let Some(operand) = instruction.operand {
            if let Some(data_flow) = &ctx.analysis_ctx.data_flow {
                // Check if the variable is initialized at this point
                // This would depend on the actual data flow analysis
            }
        }
    }

    fn validate_expression(
        &self,
        _ctx: &mut ValidationContext,
        _expr_id: ExprId,
    ) {
        // Not applicable to expressions
    }

    fn validate_body(
        &self,
        _ctx: &mut ValidationContext,
        _body: &Body,
    ) {
        // Not applicable to the entire body
    }
}
