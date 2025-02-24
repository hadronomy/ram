use std::ffi::OsString;
use std::process::ExitCode;

use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};
use chumsky::{Parser as ChumskyParser, stream};
use clap::{CommandFactory, Parser};
use cli::{Cli, Command};
pub use error::Error;
use miette::*;

pub mod cli;
pub mod error;
pub mod lang;

fn handle_parser_errors(src: &str, errors: Vec<chumsky::error::Simple<char>>) {
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

pub fn main<Args, T>(args: Args) -> Result<ExitCode>
where
    Args: Iterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let cli = match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(err) => {
            err.exit();
        }
    };

    miette::set_hook(Box::new(|_| {
        Box::new(
            miette::MietteHandlerOpts::new()
                .break_words(false)
                .word_separator(textwrap::WordSeparator::AsciiSpace)
                .word_splitter(textwrap::WordSplitter::NoHyphenation)
                .build(),
        )
    }))
    .map_err(|err| Error::SetupError(err.into()))?;

    match *cli.command {
        // execute help
        Command::Help(_) => {
            Cli::command().print_help().into_diagnostic()?;
            Ok::<_, Error>(ExitCode::SUCCESS)
        }
        Command::Validate { program, ast } => {
            let src = std::fs::read_to_string(program.clone())
                .into_diagnostic()
                .wrap_err(format!("Failed to read file: {}", program))?;
            let (program, errors) =
                lang::parser().parse_recovery(stream::Stream::from(src.clone()));
            handle_parser_errors(&src, errors);

            if ast {
                #[cfg(feature = "serde")]
                {
                    let json = serde_json::to_string_pretty(&program).unwrap();
                    println!("{json}");
                }
                #[cfg(not(feature = "serde"))]
                {
                    println!("{program:#?}");
                }
            }
            Ok::<_, Error>(ExitCode::SUCCESS)
        }
        _ => Err(Error::Unimplemented),
    }
    .wrap_err("Failed to execute command")
}
