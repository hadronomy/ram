//! Instruction types for the RAM virtual machine

use std::fmt;
use std::sync::Arc;

use crate::db::VmState;
use crate::error::VmError;
use crate::operand::{Operand, OperandKind};

/// Information about an instruction
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionInfo {
    /// The name of the instruction
    pub name: String,
    /// Whether the instruction requires an operand
    pub requires_operand: bool,
    /// The allowed operand kinds for this instruction
    pub allowed_operand_kinds: Vec<OperandKind>,
    /// A description of the instruction
    pub description: String,
}

/// An instruction in the RAM virtual machine
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instruction {
    /// The kind of instruction
    pub kind: InstructionKind,
    /// The operand for the instruction (if any)
    pub operand: Option<Operand>,
}

impl Instruction {
    /// Create a new instruction
    pub fn new(kind: InstructionKind, operand: Option<Operand>) -> Self {
        Self { kind, operand }
    }

    /// Create a new instruction with no operand
    pub fn without_operand(kind: InstructionKind) -> Self {
        Self { kind, operand: None }
    }

    /// Create a new instruction with an operand
    pub fn with_operand(kind: InstructionKind, operand: Operand) -> Self {
        Self { kind, operand: Some(operand) }
    }

    /// Validate that the instruction has the correct operands
    pub fn validate(&self) -> Result<(), VmError> {
        self.kind.validate_operand(self.operand.as_ref())
    }

    /// Execute the instruction on the given VM state
    pub fn execute(&self, vm_state: &mut dyn VmState) -> Result<(), VmError> {
        self.kind.execute(self.operand.as_ref(), vm_state)
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.operand {
            Some(operand) => write!(f, "{} {}", self.kind, operand),
            None => write!(f, "{}", self.kind),
        }
    }
}

/// Trait for instruction definitions
pub trait InstructionDefinition: Send + Sync + 'static {
    /// Get the name of the instruction
    fn name(&self) -> &str;

    /// Check if the instruction requires an operand
    fn requires_operand(&self) -> bool;

    /// Get the allowed operand kinds for this instruction
    fn allowed_operand_kinds(&self) -> &[OperandKind];

    /// Validate that the operand is valid for this instruction
    fn validate_operand(&self, operand: Option<&Operand>) -> Result<(), VmError> {
        if self.requires_operand() && operand.is_none() {
            return Err(VmError::InvalidOperand(format!("{} requires an operand", self.name())));
        }

        if !self.requires_operand() && operand.is_some() {
            return Err(VmError::InvalidOperand(format!(
                "{} does not accept an operand",
                self.name()
            )));
        }

        if let Some(operand) = operand
            && !self.allowed_operand_kinds().contains(&operand.kind)
        {
            return Err(VmError::InvalidOperand(format!(
                "{} does not accept {} operands",
                self.name(),
                match operand.kind {
                    OperandKind::Direct => "direct",
                    OperandKind::Indirect => "indirect",
                    OperandKind::Immediate => "immediate",
                }
            )));
        }

        Ok(())
    }

    /// Execute the instruction with the given operand and VM state
    fn execute(&self, operand: Option<&Operand>, vm_state: &mut dyn VmState)
    -> Result<(), VmError>;
}

/// The kind of instruction
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InstructionKind {
    /// Load a value into the accumulator
    Load,
    /// Store the accumulator value in memory
    Store,
    /// Add a value to the accumulator
    Add,
    /// Subtract a value from the accumulator
    Sub,
    /// Multiply the accumulator by a value
    Mul,
    /// Divide the accumulator by a value
    Div,
    /// Jump to a label
    Jump,
    /// Jump to a label if the accumulator is greater than zero
    JumpGtz,
    /// Jump to a label if the accumulator is zero
    JumpZero,
    /// Read a value from input
    Read,
    /// Write a value to output
    Write,
    /// Halt the program
    Halt,
    /// Custom instruction
    Custom(Arc<str>),
}

impl InstructionKind {
    /// Get the name of the instruction
    pub fn name(&self) -> &str {
        match self {
            Self::Load => "LOAD",
            Self::Store => "STORE",
            Self::Add => "ADD",
            Self::Sub => "SUB",
            Self::Mul => "MUL",
            Self::Div => "DIV",
            Self::Jump => "JUMP",
            Self::JumpGtz => "JGTZ",
            Self::JumpZero => "JZERO",
            Self::Read => "READ",
            Self::Write => "WRITE",
            Self::Halt => "HALT",
            Self::Custom(name) => name,
        }
    }

    /// Check if the instruction requires an operand
    pub fn requires_operand(&self) -> bool {
        !matches!(self, Self::Halt)
    }

    /// Get the allowed operand kinds for this instruction
    pub fn allowed_operand_kinds(&self) -> &[OperandKind] {
        static ALL_KINDS: [OperandKind; 3] =
            [OperandKind::Direct, OperandKind::Indirect, OperandKind::Immediate];

        match self {
            Self::Halt => &[],
            _ => &ALL_KINDS,
        }
    }

    /// Get a description of the instruction
    pub fn description(&self) -> &str {
        match self {
            Self::Load => "Load a value into the accumulator",
            Self::Store => "Store the accumulator value in memory",
            Self::Add => "Add a value to the accumulator",
            Self::Sub => "Subtract a value from the accumulator",
            Self::Mul => "Multiply the accumulator by a value",
            Self::Div => "Divide the accumulator by a value",
            Self::Jump => "Jump to a label",
            Self::JumpGtz => "Jump to a label if the accumulator is greater than zero",
            Self::JumpZero => "Jump to a label if the accumulator is zero",
            Self::Read => "Read a value from input",
            Self::Write => "Write a value to output",
            Self::Halt => "Halt the program",
            Self::Custom(_) => "Custom instruction",
        }
    }

    /// Get information about the instruction
    pub fn info(&self) -> InstructionInfo {
        InstructionInfo {
            name: self.name().to_string(),
            requires_operand: self.requires_operand(),
            allowed_operand_kinds: self.allowed_operand_kinds().to_vec(),
            description: self.description().to_string(),
        }
    }

    /// Get information about all standard instructions
    pub fn standard_instructions_info() -> Vec<InstructionInfo> {
        vec![
            Self::Load.info(),
            Self::Store.info(),
            Self::Add.info(),
            Self::Sub.info(),
            Self::Mul.info(),
            Self::Div.info(),
            Self::Jump.info(),
            Self::JumpGtz.info(),
            Self::JumpZero.info(),
            Self::Read.info(),
            Self::Write.info(),
            Self::Halt.info(),
        ]
    }

    /// Get all standard instruction kinds
    pub fn standard_kinds() -> Vec<InstructionKind> {
        vec![
            Self::Load,
            Self::Store,
            Self::Add,
            Self::Sub,
            Self::Mul,
            Self::Div,
            Self::Jump,
            Self::JumpGtz,
            Self::JumpZero,
            Self::Read,
            Self::Write,
            Self::Halt,
        ]
    }

    /// Parse an instruction name into an InstructionKind
    pub fn from_name(name: &str) -> Self {
        match name.to_uppercase().as_str() {
            "LOAD" => Self::Load,
            "STORE" => Self::Store,
            "ADD" => Self::Add,
            "SUB" => Self::Sub,
            "MUL" => Self::Mul,
            "DIV" => Self::Div,
            "JUMP" | "JMP" => Self::Jump,
            "JGTZ" => Self::JumpGtz,
            "JZERO" => Self::JumpZero,
            "READ" => Self::Read,
            "WRITE" => Self::Write,
            "HALT" => Self::Halt,
            _ => Self::Custom(Arc::from(name)),
        }
    }

    /// Validate that the operand is valid for this instruction
    pub fn validate_operand(&self, _operand: Option<&Operand>) -> Result<(), VmError> {
        // This is just a placeholder - the actual validation is done by the instruction registry
        // which has access to the instruction definitions
        Ok(())
    }

    /// Execute the instruction with the given operand and VM state
    pub fn execute(
        &self,
        _operand: Option<&Operand>,
        _vm_state: &mut dyn VmState,
    ) -> Result<(), VmError> {
        // This is just a placeholder - the actual execution is done by the instruction registry
        // which has access to the instruction definitions
        Err(VmError::InvalidInstruction(format!(
            "No implementation for instruction: {}",
            self.name()
        )))
    }
}

impl InstructionDefinition for InstructionKind {
    /// Get the name of the instruction
    fn name(&self) -> &str {
        self.name()
    }

    /// Check if the instruction requires an operand
    fn requires_operand(&self) -> bool {
        self.requires_operand()
    }

    /// Get the allowed operand kinds for this instruction
    fn allowed_operand_kinds(&self) -> &[OperandKind] {
        self.allowed_operand_kinds()
    }

    /// Validate that the operand is valid for this instruction
    fn validate_operand(&self, operand: Option<&Operand>) -> Result<(), VmError> {
        if self.requires_operand() && operand.is_none() {
            return Err(VmError::InvalidOperand(format!("{} requires an operand", self.name())));
        }
        if !self.requires_operand() && operand.is_some() {
            return Err(VmError::InvalidOperand(format!(
                "{} does not accept an operand",
                self.name()
            )));
        }
        if let Some(operand) = operand
            && !self.allowed_operand_kinds().contains(&operand.kind)
        {
            return Err(VmError::InvalidOperand(format!(
                "{} does not accept {} operands",
                self.name(),
                match operand.kind {
                    OperandKind::Direct => "direct",
                    OperandKind::Indirect => "indirect",
                    OperandKind::Immediate => "immediate",
                }
            )));
        }
        Ok(())
    }

    /// Execute the instruction with the given operand and VM state
    fn execute(
        &self,
        operand: Option<&Operand>,
        vm_state: &mut dyn VmState,
    ) -> Result<(), VmError> {
        self.execute(operand, vm_state)
    }
}

impl fmt::Display for InstructionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}
