//! Type representation in HIR
//!
//! This module defines the type system for the HIR.
//! It represents the types that expressions and values can have.

use std::fmt;

/// A type in the HIR
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Ty {
    /// The unknown type, used during type inference
    Unknown,

    /// An integer type
    Int,

    /// A string type
    String,

    /// A label type (represents a position in the code)
    Label,

    /// A memory address type
    Address,

    /// An error type, used when type checking fails
    Error,
}

impl Ty {
    /// Returns true if this is the error type
    pub fn is_error(&self) -> bool {
        matches!(self, Ty::Error)
    }

    /// Returns true if this is the unknown type
    pub fn is_unknown(&self) -> bool {
        matches!(self, Ty::Unknown)
    }
}

impl fmt::Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ty::Unknown => write!(f, "?"),
            Ty::Int => write!(f, "int"),
            Ty::String => write!(f, "string"),
            Ty::Label => write!(f, "label"),
            Ty::Address => write!(f, "address"),
            Ty::Error => write!(f, "{{error}}"),
        }
    }
}

/// A trait for types that have a HIR type
pub trait HasType {
    /// Get the type of this item
    fn ty(&self) -> Ty;
}
