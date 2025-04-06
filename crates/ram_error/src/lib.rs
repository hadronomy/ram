use std::path::PathBuf;

use chumsky::Span;
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

pub fn handle_parser_errors(src: &str, errors: Vec<chumsky::error::Simple<char>>) {
    let parser_errors: Vec<SingleParserError> = errors
        .into_iter()
        .map(|e| {
            // Format the error message similar to the original
            let message = if let chumsky::error::SimpleReason::Custom(msg) = e.reason() {
                msg.clone()
            } else {
                format!(
                    "{}{}, expected {}",
                    if e.found().is_some() {
                        "Unexpected token"
                    } else {
                        "Unexpected end of input"
                    },
                    if let Some(label) = e.label() {
                        format!(" while parsing {label}")
                    } else {
                        String::new()
                    },
                    if e.expected().len() == 0 {
                        "something else".to_string()
                    } else {
                        e.expected()
                            .map(|expected| match expected {
                                Some(expected) => expected.to_string(),
                                None => "end of input".to_string(),
                            })
                            .collect::<Vec<_>>()
                            .join(", ")
                    },
                )
            };

            // Create a list of labeled spans
            let mut labels = Vec::new();

            // Convert main error span to a LabeledSpan
            let span_offset = e.span().start() + 1;
            let span_len = e.span().end() - e.span().start();

            // Create a dynamic label showing the unexpected token
            let label_text = match e.found() {
                Some(c) => format!("Unexpected token '{c}'"),
                None => "Unexpected end of input".to_string(),
            };

            labels.push(LabeledSpan::new(Some(label_text), span_offset, span_len));

            // Check for unclosed delimiter case and add as another LabeledSpan
            if let chumsky::error::SimpleReason::Unclosed { span: delim_span, delimiter: _ } =
                e.reason()
            {
                let delim_offset = delim_span.start();
                let delim_len = delim_span.end() - delim_span.start();

                labels.push(LabeledSpan::new(
                    Some("Unclosed delimiter".to_string()),
                    delim_offset,
                    delim_len,
                ));
            }

            // Create single parser error
            SingleParserError { message, labels }
        })
        .collect();

    // Create the collection error
    let error_collection =
        ParserError { src: NamedSource::new("input.ram", src.to_string()), errors: parser_errors };

    // Report the errors using miette
    eprintln!("{:?}", miette::Report::new(error_collection));
}
