//! Memory implementation for the RAM virtual machine optimized with paging.

use ram_core::error::VmError;

// Page size of 4096 elements (matches typical OS page sizes for efficiency)
const PAGE_SIZE: usize = 4096;
const PAGE_MASK: usize = PAGE_SIZE - 1;
const PAGE_SHIFT: u32 = 12; // 2^12 = 4096

/// Memory for the RAM virtual machine.
///
/// Uses a paged architecture to allow for sparse memory usage while maintaining
/// O(1) access times and cache locality for sequential operations.
/// This structure is significantly faster than a HashMap for hot-path access.
#[derive(Debug, Default, Clone)]
pub struct Memory {
    /// Directory of memory pages.
    /// `Option<Box<...>>` allows us to lazily allocate pages only when written to.
    pages: Vec<Option<Box<[i64; PAGE_SIZE]>>>,
}

impl Memory {
    /// Create a new empty memory
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a value from memory.
    ///
    /// Returns 0 for uninitialized cells (standard RAM behavior).
    #[inline(always)]
    pub fn get(&self, address: i64) -> Result<i64, VmError> {
        if address < 0 {
            return Err(VmError::InvalidMemoryAccess(format!(
                "Cannot access negative address: {}",
                address
            )));
        }

        let addr_usize = address as usize;
        let page_idx = addr_usize >> PAGE_SHIFT;
        let offset = addr_usize & PAGE_MASK;

        // Fast path: direct indexing into the pages vector
        match self.pages.get(page_idx) {
            // If the page exists, return the value at the offset
            Some(Some(page)) => Ok(page[offset]),
            // If the page doesn't exist, return 0 (uninitialized memory)
            _ => Ok(0),
        }
    }

    /// Set a value in memory.
    ///
    /// Lazily allocates pages as needed.
    #[inline(always)]
    pub fn set(&mut self, address: i64, value: i64) -> Result<(), VmError> {
        if address < 0 {
            return Err(VmError::InvalidMemoryAccess(format!(
                "Cannot access negative address: {}",
                address
            )));
        }

        let addr_usize = address as usize;
        let page_idx = addr_usize >> PAGE_SHIFT;
        let offset = addr_usize & PAGE_MASK;

        // Ensure the page directory is large enough
        if page_idx >= self.pages.len() {
            // Safety Check: Prevent arbitrary OOM if a program tries to write to a massive address
            // Limit to approx 1 billion addresses (~8GB max theoretical usage)
            if page_idx > 262_144 {
                // 262144 * 4096 = ~1 Billion indices
                return Err(VmError::InvalidMemoryAccess(format!(
                    "Memory limit exceeded: address {} is too large",
                    address
                )));
            }

            // Expand the directory, filling new slots with None
            self.pages.resize(page_idx + 1, None);
        }

        // Allocate the specific page if it doesn't exist yet
        if self.pages[page_idx].is_none() {
            self.pages[page_idx] = Some(Box::new([0; PAGE_SIZE]));
        }

        // Write the value to the page
        // Safety: We verified bounds and allocation above
        if let Some(page) = &mut self.pages[page_idx] {
            page[offset] = value;
        }

        Ok(())
    }

    /// Clear all memory cells
    pub fn clear(&mut self) {
        self.pages.clear();
    }
}
