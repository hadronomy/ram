use std::ffi::OsString;
use std::process::ExitCode;

use anstream::println;
use chumsky::{Parser as ChumskyParser, stream};
use clap::{CommandFactory, Parser};
use cli::{Cli, ColorChoice, Command, VersionFormat};
use error::handle_parser_errors;
use miette::*;
use shadow_rs::shadow;
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{Layer, Registry};

pub use crate::error::Error;
pub use crate::version::*;

pub mod cli;
pub mod error;
pub mod language;
pub mod lsp;
pub mod version;

shadow!(build);

pub async fn main<Args, T>(args: Args) -> Result<ExitCode>
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

    init_tracing(&cli);

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
        Command::Lsp => lsp::run()
            .await
            .wrap_err("Failed to run LSP server")
            .map(|_| ExitCode::SUCCESS)
            .map_err(Error::LspError),
        Command::Version { output_format } => {
            let version = VERSION
                .clone()
                .with_color_choice(cli.top_level.global_args.color.unwrap_or(ColorChoice::Auto));

            match output_format {
                VersionFormat::Text => println!("{}", Version::new()),
                VersionFormat::Json => {
                    #[cfg(feature = "serde")]
                    {
                        let json = serde_json::to_string_pretty(&Version::new()).unwrap();
                        println!("{json}");
                    }
                    #[cfg(not(feature = "serde"))]
                    {
                        println!("JSON output is not supported in this build");
                    }
                }
                VersionFormat::Toml => println!("{}", version),
            }
            Ok::<_, Error>(ExitCode::SUCCESS)
        }
    }
    .wrap_err("Failed to execute command")
}

fn init_tracing(cli: &Cli) {
    let log_path = &cli.top_level.global_args.mirror;
    let level = cli.top_level.global_args.verbose;
    let no_stdout_log = cli.top_level.global_args.no_stdout_log;

    // Create a registry for our subscribers
    let registry = Registry::default();

    let mut stdout_subscriber = None;

    // Always add the stdout subscriber
    if !no_stdout_log {
        stdout_subscriber = Some(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stdout)
                .with_ansi(true)
                .with_timer(UtcTime::rfc_3339()),
        );
    }

    let level_filter =
        tracing_subscriber::EnvFilter::from_default_env().add_directive(match level {
            0 => "warn".parse().unwrap(),
            1 => "info".parse().unwrap(),
            2 => "debug".parse().unwrap(),
            _ => "trace".parse().unwrap(),
        });

    let registry = registry.with(stdout_subscriber.with_filter(level_filter));

    // Only add the file subscriber if log_path is specified
    if let Some(path) = log_path {
        let file_path = path.clone();
        let file_subscriber = tracing_subscriber::fmt::layer()
            .with_writer(move || {
                let log_path = file_path.clone();
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&log_path)
                    .unwrap_or_else(|_| panic!("Failed to open log file: {}", log_path.display()))
            })
            .with_ansi(false)
            .with_timer(UtcTime::rfc_3339());

        registry.with(file_subscriber).init();
        return;
    }

    registry.init();
}
