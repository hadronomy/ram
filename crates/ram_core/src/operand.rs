//! Operand types for RAM instructions

use std::fmt;

/// Value of an operand, which can be either a number, a string, or an indexed reference
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperandValue {
    /// A numeric value
    Number(i64),
    /// A string value (typically a label name)
    String(String),
    /// An indexed value (base, index_register)
    Indexed(i64, i64),
}

impl OperandValue {
    /// Get the numeric value if this is a number
    pub fn as_number(&self) -> Option<i64> {
        match self {
            Self::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Get the string value if this is a string
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }
}

impl fmt::Display for OperandValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{}", n),
            Self::String(s) => write!(f, "{}", s),
            Self::Indexed(base, index) => write!(f, "{}[{}]", base, index),
        }
    }
}

/// Operand for a RAM instruction
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Operand {
    /// The kind of operand
    pub kind: OperandKind,
    /// The value of the operand
    pub value: OperandValue,
}

impl Operand {
    /// Create a new direct operand with a numeric value
    pub fn direct(value: i64) -> Self {
        Self { kind: OperandKind::Direct, value: OperandValue::Number(value) }
    }

    /// Create a new direct operand with a string value
    pub fn direct_str(value: impl Into<String>) -> Self {
        Self { kind: OperandKind::Direct, value: OperandValue::String(value.into()) }
    }

    /// Create a new indirect operand with a numeric value
    pub fn indirect(value: i64) -> Self {
        Self { kind: OperandKind::Indirect, value: OperandValue::Number(value) }
    }

    /// Create a new indirect operand with a string value
    pub fn indirect_str(value: impl Into<String>) -> Self {
        Self { kind: OperandKind::Indirect, value: OperandValue::String(value.into()) }
    }

    /// Create a new immediate operand with a numeric value
    pub fn immediate(value: i64) -> Self {
        Self { kind: OperandKind::Immediate, value: OperandValue::Number(value) }
    }

    /// Create a new immediate operand with a string value
    pub fn immediate_str(value: impl Into<String>) -> Self {
        Self { kind: OperandKind::Immediate, value: OperandValue::String(value.into()) }
    }

    /// Create a new indexed operand
    pub fn indexed(base: i64, index: i64) -> Self {
        Self { kind: OperandKind::Indexed, value: OperandValue::Indexed(base, index) }
    }
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            OperandKind::Direct => write!(f, "{}", self.value),
            OperandKind::Indirect => write!(f, "*{}", self.value),
            OperandKind::Immediate => write!(f, "={}", self.value),
            OperandKind::Indexed => write!(f, "{}", self.value),
        }
    }
}

/// The kind of operand
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperandKind {
    /// Direct addressing (e.g., 5)
    Direct,
    /// Indirect addressing (e.g., *5)
    Indirect,
    /// Immediate addressing (e.g., =5)
    Immediate,
    /// Indexed addressing (e.g., 5[2])
    Indexed,
}
