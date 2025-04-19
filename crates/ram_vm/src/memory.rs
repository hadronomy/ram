//! Memory implementation for the RAM virtual machine

use std::collections::HashMap;

use ram_core::error::VmError;

/// Memory for the RAM virtual machine
#[derive(Debug, Default, Clone)]
pub struct Memory {
    /// The memory cells
    cells: HashMap<i64, i64>,
}

impl Memory {
    /// Create a new empty memory
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a value from memory
    pub fn get(&self, address: i64) -> Result<i64, VmError> {
        if address < 0 {
            return Err(VmError::InvalidMemoryAccess(format!(
                "Cannot access negative address: {}",
                address
            )));
        }
        Ok(*self.cells.get(&address).unwrap_or(&0))
    }

    /// Set a value in memory
    pub fn set(&mut self, address: i64, value: i64) -> Result<(), VmError> {
        if address < 0 {
            return Err(VmError::InvalidMemoryAccess(format!(
                "Cannot access negative address: {}",
                address
            )));
        }
        self.cells.insert(address, value);
        Ok(())
    }

    /// Clear all memory cells
    pub fn clear(&mut self) {
        self.cells.clear();
    }
}
