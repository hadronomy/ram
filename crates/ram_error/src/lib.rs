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
    ParserError(#[from] ParserError),
}

#[derive(Error, Diagnostic, Debug)]
#[error("another error")]
#[diagnostic(code(ram::unknown_error))]
pub struct UnknownError {
    #[label("here")]
    pub at: SourceSpan,
}

#[derive(Error, Diagnostic, Debug, Clone, Eq, PartialEq)]
#[error("Parse error: {message}")]
pub struct SingleParserError {
    pub message: String,

    #[label(collection)]
    pub labels: Vec<LabeledSpan>,
}

#[derive(Error, Diagnostic, Debug, Clone, Eq, PartialEq)]
#[error("Multiple parser errors")]
#[diagnostic(code(ram::parse_error))]
pub struct ParserError {
    // Source code for all errors
    #[source_code]
    pub src: NamedSource<String>,

    // All related parser errors
    #[related]
    pub errors: Vec<SingleParserError>,
}
