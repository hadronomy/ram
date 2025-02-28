use std::path::PathBuf;

use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};
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

pub fn handle_parser_errors(src: &str, errors: Vec<chumsky::error::Simple<char>>) {
    errors.into_iter().for_each(|e| {
        let msg = if let chumsky::error::SimpleReason::Custom(msg) = e.reason() {
            msg.clone()
        } else {
            format!(
                "{}{}, expected {}",
                if e.found().is_some() { "Unexpected token" } else { "Unexpected end of input" },
                if let Some(label) = e.label() {
                    format!(" while parsing {}", label)
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

        let report =
            Report::build(ReportKind::Error, e.span()).with_code(3).with_message(msg).with_label(
                Label::new(e.span())
                    .with_message(match e.reason() {
                        chumsky::error::SimpleReason::Custom(msg) => msg.clone(),
                        _ => format!(
                            "Unexpected {}",
                            e.found()
                                .map(|c| format!("token {}", c.fg(Color::Red)))
                                .unwrap_or_else(|| "end of input".to_string())
                        ),
                    })
                    .with_color(Color::Red),
            );

        let report = match e.reason() {
            chumsky::error::SimpleReason::Unclosed { span, delimiter } => report.with_label(
                Label::new(span.clone())
                    .with_message(format!("Unclosed delimiter {}", delimiter.fg(Color::Yellow)))
                    .with_color(Color::Yellow),
            ),
            chumsky::error::SimpleReason::Unexpected => report,
            chumsky::error::SimpleReason::Custom(_) => report,
        };

        report.finish().print(Source::from(&src)).unwrap();
    });
}
