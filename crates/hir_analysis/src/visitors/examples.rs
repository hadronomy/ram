//! Examples of using the visitor pattern
//!
//! This module contains examples of how to use the visitor pattern to traverse
//! and analyze HIR structures.

use std::ops::ControlFlow;

use hir::body::{Body, Instruction, Literal};

use super::traits::{Visitor, VisitorResult};
use super::walkers::walk_body;

/// A visitor that counts the number of instructions
pub struct InstructionCounter {
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

/// Count the number of instructions in a body
pub fn count_instructions(body: &Body) -> usize {
    let visitor = InstructionCounter { count: 0 };
    walk_body(visitor, body)
}

/// A visitor that collects all integer literals
pub struct IntLiteralCollector {
    literals: Vec<i64>,
}

impl Visitor for IntLiteralCollector {
    type Result = Vec<i64>;

    fn visit_literal(&mut self, literal: &Literal) -> VisitorResult<Self::Result> {
        if let Literal::Int(value) = literal {
            self.literals.push(*value);
        }
        ControlFlow::Continue(())
    }

    fn finish(self) -> Self::Result {
        self.literals
    }
}

/// Collect all integer literals in a body
pub fn collect_int_literals(body: &Body) -> Vec<i64> {
    let visitor = IntLiteralCollector { literals: Vec::new() };
    walk_body(visitor, body)
}

/// A visitor that finds the first instruction with a specific opcode
pub struct InstructionFinder<'a> {
    target_opcode: &'a str,
    found_instruction: Option<Instruction>,
}

impl<'a> Visitor for InstructionFinder<'a> {
    type Result = Option<Instruction>;

    fn visit_instruction(&mut self, instruction: &Instruction) -> VisitorResult<Self::Result> {
        if instruction.opcode == self.target_opcode {
            self.found_instruction = Some(instruction.clone());
            return ControlFlow::Break(self.found_instruction.clone());
        }
        ControlFlow::Continue(())
    }

    fn finish(self) -> Self::Result {
        self.found_instruction
    }
}

/// Find the first instruction with a specific opcode
pub fn find_instruction(body: &Body, opcode: &str) -> Option<Instruction> {
    let visitor = InstructionFinder { target_opcode: opcode, found_instruction: None };
    walk_body(visitor, body)
}
