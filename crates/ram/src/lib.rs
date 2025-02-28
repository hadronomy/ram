use std::ffi::OsString;
use std::process::ExitCode;

use chumsky::{Parser as ChumskyParser, stream};
use clap::{CommandFactory, Parser};
use cli::{Cli, Command};
use error::handle_parser_errors;
use miette::*;

pub use crate::error::Error;

pub mod cli;
pub mod error;
pub mod language;

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
                .with_syntax_highlighting(language::highlighter())
                .tab_width(2)
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
                language::parser().parse_recovery(stream::Stream::from(src.clone()));
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
