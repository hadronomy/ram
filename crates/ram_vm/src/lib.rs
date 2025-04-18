//! RAM virtual machine implementation
//!
//! This crate implements the RAM virtual machine, which can execute RAM programs.

pub mod db;
pub mod io;
pub mod memory;
pub mod program;
#[cfg(test)]
mod tests;
pub mod vm;

pub use crate::db::{VmDatabase, VmDatabaseImpl};
pub use crate::io::{Input, Output};
pub use crate::memory::Memory;
pub use crate::program::Program;
pub use crate::vm::VirtualMachine;
