use std::path::PathBuf;

use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

#[derive(Error, Diagnostic, Debug)]
pub enum Error {
    #[error(transparent)]
    #[diagnostic(code(ram::io_error))]
    IoError(#[from] std::io::Error),

    #[error("Setup error: {0}")]
    #[diagnostic(code(ram::setup_error))]
    SetupError(miette::Report),

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
}

#[derive(Error, Diagnostic, Debug)]
#[error("another error")]
pub struct UnknownError {
    #[label("here")]
    pub at: SourceSpan,
}
