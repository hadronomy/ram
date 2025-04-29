//! Type system for RAM HIR
//!
//! This module provides a simple type system for RAM programs.
//! It includes type checking and inference for expressions and instructions.

use std::collections::HashMap;
use std::sync::Arc;

use hir::body::Body;
use hir::expr::ExprId;
use hir::ids::{DefId, LocalDefId};

/// A type in the RAM type system
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    /// Integer type
    Int,

    /// Address type (for labels)
    Address,

    /// Unknown type (for inference)
    Unknown,

    /// Error type (for recovery)
    Error,
}

/// A unique identifier for a type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(pub u32);

/// Type system for RAM programs
#[derive(Debug, Clone)]
pub struct TypeSystem {
    /// Map from type ID to type
    types: HashMap<TypeId, Type>,

    /// Next type ID to assign
    next_id: u32,

    /// Predefined type IDs
    pub int_type: TypeId,
    pub address_type: TypeId,
    pub unknown_type: TypeId,
    pub error_type: TypeId,
}

/// Type information for a body
#[derive(Debug, Clone, Default)]
pub struct TypeInfo {
    /// Map from expression ID to type ID
    expr_types: HashMap<ExprId, TypeId>,

    /// Map from instruction ID to type ID
    instr_types: HashMap<LocalDefId, TypeId>,
}

impl TypeSystem {
    /// Create a new type system
    pub fn new() -> Self {
        let mut system = Self {
            types: HashMap::new(),
            next_id: 0,
            int_type: TypeId(0),
            address_type: TypeId(0),
            unknown_type: TypeId(0),
            error_type: TypeId(0),
        };

        // Register predefined types
        system.int_type = system.register_type(Type::Int);
        system.address_type = system.register_type(Type::Address);
        system.unknown_type = system.register_type(Type::Unknown);
        system.error_type = system.register_type(Type::Error);

        system
    }

    /// Register a new type and get its ID
    pub fn register_type(&mut self, ty: Type) -> TypeId {
        let id = TypeId(self.next_id);
        self.next_id += 1;
        self.types.insert(id, ty);
        id
    }

    /// Get the type for a type ID
    pub fn get_type(&self, id: TypeId) -> Option<&Type> {
        self.types.get(&id)
    }
}

impl TypeInfo {
    /// Create a new type info
    pub fn new() -> Self {
        Self { expr_types: HashMap::new(), instr_types: HashMap::new() }
    }

    /// Set the type of an expression
    pub fn set_expr_type(&mut self, expr_id: ExprId, type_id: TypeId) {
        self.expr_types.insert(expr_id, type_id);
    }

    /// Get the type of an expression
    pub fn get_expr_type(&self, expr_id: ExprId) -> Option<TypeId> {
        self.expr_types.get(&expr_id).copied()
    }

    /// Set the type of an instruction
    pub fn set_instr_type(&mut self, instr_id: LocalDefId, type_id: TypeId) {
        self.instr_types.insert(instr_id, type_id);
    }

    /// Get the type of an instruction
    pub fn get_instr_type(&self, instr_id: LocalDefId) -> Option<TypeId> {
        self.instr_types.get(&instr_id).copied()
    }

    /// Get a value by type
    pub fn get<T: 'static>(&self) -> Option<&T> {
        None // This is a stub implementation
    }

    /// Set a value by type
    pub fn set_type<T: 'static>(&mut self, value: T) {
        // This is a stub implementation
    }
}

/// Query function for type checking a body
pub(crate) fn type_check_query(db: &dyn crate::AnalysisDatabase, def_id: DefId) -> Arc<TypeInfo> {
    let body = db.body(def_id);
    let mut context = crate::AnalysisContext::new(db, &body);

    // Create a new type info
    let type_info = TypeInfo::new();

    // Set the type info in the context
    context.type_info = type_info;

    // Return the type info
    Arc::new(context.type_info().clone())
}

/// Query function for getting the type of an expression
pub(crate) fn expr_type_query(
    db: &dyn crate::AnalysisDatabase,
    def_id: DefId,
    expr_id: ExprId,
) -> TypeId {
    let type_info = db.type_info(def_id);
    type_info.get_expr_type(expr_id).unwrap_or_else(|| {
        // If the type is not found, return the error type
        // In a real implementation, we would have a way to get the error type ID
        TypeId(3) // Assuming error_type is TypeId(3)
    })
}
