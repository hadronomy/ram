//! RAM virtual machine implementation
//!
//! This crate implements the RAM virtual machine, which can execute RAM programs.
//! It provides a convenient API for creating and running RAM programs.

pub mod db;
pub mod io;
pub mod memory;
pub mod program;
pub mod runner;
#[cfg(test)]
mod tests;
pub mod vm;

pub use crate::db::{VmDatabase, VmDatabaseImpl};
pub use crate::io::{Input, Output, VecInput, VecOutput};
pub use crate::memory::Memory;
pub use crate::program::Program;
pub use crate::runner::{
    RunResult, run_program, run_program_with_max_iterations, run_program_with_memory,
};
pub use crate::vm::{VirtualMachine, VirtualMachineBuilder};
