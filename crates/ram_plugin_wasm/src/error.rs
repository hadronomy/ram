//! Error types for the WASM plugin system

use thiserror::Error;

/// Errors that can occur when working with WASM plugins
#[derive(Debug, Error)]
pub enum PluginError {
    /// Failed to load a WASM plugin
    #[error("Failed to load WASM plugin: {0}")]
    LoadError(String),

    /// Failed to instantiate a WASM plugin
    #[error("Failed to instantiate WASM plugin: {0}")]
    InstantiationError(String),

    /// An error occurred during plugin execution
    #[error("Plugin execution error: {0}")]
    ExecutionError(String),

    /// Conversion between host and plugin types failed
    #[error("Type conversion error: {0}")]
    ConversionError(String),

    /// The instruction is not provided by this plugin
    #[error("Unknown instruction: {0}")]
    UnknownInstruction(String),
}

/// Convert from our plugin error type to the WIT-defined error type
impl From<PluginError> for crate::bindings::ram::plugin::types::Error {
    fn from(error: PluginError) -> Self {
        use crate::bindings::ram::plugin::types::Error;
        match error {
            PluginError::LoadError(msg) | PluginError::InstantiationError(msg) => {
                Error::InvalidInstruction(msg)
            }
            PluginError::ExecutionError(msg) => Error::ExecutionError(msg),
            PluginError::ConversionError(msg) => Error::VmError(msg),
            PluginError::UnknownInstruction(msg) => Error::InvalidInstruction(msg),
        }
    }
}

/// Convert from ram_core error types to our plugin error type
impl From<ram_core::error::VmError> for PluginError {
    fn from(error: ram_core::error::VmError) -> Self {
        match error {
            ram_core::error::VmError::InvalidInstruction(msg) => {
                PluginError::UnknownInstruction(msg)
            }
            ram_core::error::VmError::InvalidOperand(msg) => {
                PluginError::ExecutionError(format!("Invalid operand: {msg}"))
            }
            _ => PluginError::ExecutionError(format!("VM error: {error}")),
        }
    }
}
