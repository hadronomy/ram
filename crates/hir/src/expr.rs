//! Expression representation in HIR
//!
//! This module defines the expression identifiers and utilities
//! used in the HIR body representation.

use std::fmt;

use crate::ty::Ty;

/// A unique identifier for an expression in a body
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExprId(pub u32);

impl fmt::Display for ExprId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "expr{}", self.0)
    }
}

/// A database of expressions with their types
#[derive(Debug, Default)]
pub struct ExprDatabase {
    /// The types of expressions
    types: Vec<Ty>,
}

impl ExprDatabase {
    /// Create a new expression database
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the type of an expression
    pub fn set_type(&mut self, expr_id: ExprId, ty: Ty) {
        let idx = expr_id.0 as usize;
        if idx >= self.types.len() {
            self.types.resize(idx + 1, Ty::Unknown);
        }
        self.types[idx] = ty;
    }

    /// Get the type of an expression
    pub fn get_type(&self, expr_id: ExprId) -> Ty {
        let idx = expr_id.0 as usize;
        if idx < self.types.len() { self.types[idx].clone() } else { Ty::Unknown }
    }
}
