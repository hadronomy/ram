use std::ffi::OsString;
use std::process::ExitCode;

use clap::{CommandFactory, Parser};
pub use error::Error;
use ram_cli::{Cli, Command};
use miette::*;

pub mod error;

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
        _ => Err(Error::Unimplemented),
    }
    .wrap_err("Failed to execute command")
}
