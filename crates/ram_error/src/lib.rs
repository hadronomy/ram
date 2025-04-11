use std::path::PathBuf;

use miette::{Diagnostic, LabeledSpan, NamedSource, SourceSpan};
use thiserror::Error;

#[derive(Error, Diagnostic, Debug)]
pub enum Error {
    #[error(transparent)]
    #[diagnostic(code(ram::io_error))]
    IoError(#[from] std::io::Error),

    #[error("Setup error: {0}")]
    #[diagnostic(code(ram::setup_error))]
    SetupError(miette::Report),

    #[error("LSP error: {0}")]
    #[diagnostic(code(ram::lsp_error))]
    LspError(miette::Report),

    #[error("Path not found: {0}")]
    #[diagnostic(code(ram::path_not_found))]
    PathNotFound(PathBuf),

    #[error("Invalid path format")]
    #[diagnostic(code(ram::invalid_path))]
    InvalidPathFormat,

    #[error("Unimplemented feature")]
    #[diagnostic(code(ram::unimplemented))]
    #[diagnostic(help("Ooops! This feature is not implemented yet."))]
    Unimplemented,

    #[error(transparent)]
    #[diagnostic(transparent)]
    UnknownError(#[from] UnknownError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    ParserError(#[from] Report),
}

#[derive(Error, Diagnostic, Debug)]
#[error("another error")]
#[diagnostic(code(ram::unknown_error))]
pub struct UnknownError {
    #[label("here")]
    pub at: SourceSpan,
}

#[derive(Error, Debug, Clone, Eq, PartialEq)]
#[error("{message}")]
pub struct SingleReport {
    pub message: String,
    pub labels: Vec<LabeledSpan>,
    pub severity: Option<miette::Severity>,
    pub code: Option<String>,
}

impl SingleReport {
    pub fn new(message: String, labels: Vec<LabeledSpan>) -> Self {
        Self { message, labels, severity: None, code: None }
    }
}

impl miette::Diagnostic for SingleReport {
    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        Some(Box::new(self.labels.iter().cloned()))
    }

    fn severity(&self) -> Option<miette::Severity> {
        self.severity
    }

    fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        self.code.as_ref().map(|c| Box::new(c) as Box<dyn std::fmt::Display>)
    }
}

#[derive(Error, Diagnostic, Debug, Clone, Eq, PartialEq)]
#[error("Multiple parser errors")]
pub struct Report {
    // Source code for all errors
    #[source_code]
    pub src: NamedSource<String>,

    // All related parser errors
    #[related]
    pub errors: Vec<SingleReport>,
}
