//! This module provides a way to construct a `File`
//! It is intended to be completely decoupled from the
//! parser, so as to allow to evolve the tree representation
//! without affecting the parser.

use std::mem;

use crate::SyntaxKind::{self, *};

/// [`crate::Parser`] produces a flat list of [`Event`]s.
/// They are converted to a tree-structure in
/// a separate pass, via `TreeBuilder`
#[derive(Debug, PartialEq)]
pub enum Event {}
