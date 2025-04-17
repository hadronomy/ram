//! Database interface for HIR queries
//!
//! This module defines the salsa database interface for the HIR crate.

use std::sync::Arc;

use base_db::input::FileId;
use hir_def::db::HirDefDatabase;

/// The database trait for HIR queries
#[salsa::db]
pub trait HirDatabase: HirDefDatabase {
    /// Resolve all definitions for a given file
    fn resolve_file(&self, file_id: FileId) -> Arc<crate::name_resolution::ResolvedFile>;

    /// Get the body for a specific definition
    fn body(&self, def_id: crate::ids::DefId) -> Arc<crate::body::Body>;
}
