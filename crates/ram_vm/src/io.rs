//! Input/output implementations for the RAM virtual machine

use std::io::{self, BufRead, Write};

use ram_core::error::VmError;

/// Input source for the RAM virtual machine
pub trait Input {
    /// Read a value from the input
    fn read(&mut self) -> Result<i64, VmError>;
}

/// Output sink for the RAM virtual machine
pub trait Output {
    /// Write a value to the output
    fn write(&mut self, value: i64) -> Result<(), VmError>;
}

/// Standard input implementation
pub struct StdinInput {
    /// The input buffer
    buffer: Vec<i64>,
}

impl StdinInput {
    /// Create a new stdin input
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }
}

impl Default for StdinInput {
    fn default() -> Self {
        Self::new()
    }
}

impl Input for StdinInput {
    fn read(&mut self) -> Result<i64, VmError> {
        if self.buffer.is_empty() {
            // Read a new line from stdin
            let stdin = io::stdin();
            let mut line = String::new();
            print!("Input: ");
            io::stdout().flush().map_err(|e| VmError::IoError(e.to_string()))?;
            stdin.lock().read_line(&mut line).map_err(|e| VmError::IoError(e.to_string()))?;

            // Parse the line as a list of integers
            for token in line.split_whitespace() {
                match token.parse::<i64>() {
                    Ok(value) => self.buffer.push(value),
                    Err(e) => return Err(VmError::IoError(format!("Invalid input: {}", e))),
                }
            }

            if self.buffer.is_empty() {
                return Err(VmError::IoError("No input provided".to_string()));
            }
        }

        Ok(self.buffer.remove(0))
    }
}

/// Standard output implementation
pub struct StdoutOutput;

impl StdoutOutput {
    /// Create a new stdout output
    pub fn new() -> Self {
        Self
    }
}

impl Default for StdoutOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl Output for StdoutOutput {
    fn write(&mut self, value: i64) -> Result<(), VmError> {
        println!("Output: {}", value);
        Ok(())
    }
}

/// Vector-based input implementation for testing
pub struct VecInput {
    /// The input values
    values: Vec<i64>,
    /// The current position
    pos: usize,
}

impl VecInput {
    /// Create a new vector input with the given values
    pub fn new(values: Vec<i64>) -> Self {
        Self { values, pos: 0 }
    }
}

impl Input for VecInput {
    fn read(&mut self) -> Result<i64, VmError> {
        if self.pos >= self.values.len() {
            return Err(VmError::IoError("End of input".to_string()));
        }
        let value = self.values[self.pos];
        self.pos += 1;
        Ok(value)
    }
}

/// Vector-based output implementation for testing
#[derive(Debug, Clone)]
pub struct VecOutput {
    /// The output values
    pub values: Vec<i64>,
}

impl VecOutput {
    /// Create a new empty vector output
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }
}

impl Default for VecOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl Output for VecOutput {
    fn write(&mut self, value: i64) -> Result<(), VmError> {
        self.values.push(value);
        Ok(())
    }
}
