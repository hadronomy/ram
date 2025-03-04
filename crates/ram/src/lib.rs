use std::ffi::OsString;
use std::process::ExitCode;

use anstream::println;
use chumsky::{Parser as ChumskyParser, stream};
use clap::{CommandFactory, Parser};
use miette::*;
use shadow_rs::shadow;
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{Layer, Registry};

use crate::cli::{Cli, Command, VersionFormat};
use crate::color::ColorChoice;
pub use crate::error::Error;
use crate::error::handle_parser_errors;
pub use crate::version::*;

pub mod cli;
pub mod color;
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
            // If error is --version or -V, we'll handle that specially
            if err.kind() == clap::error::ErrorKind::DisplayVersion {
                // Create a default CLI and run the version command
                let default_cli = Cli::parse_from(["ram", "version"]);
                handle_command(default_cli).await?;
                return Ok(ExitCode::SUCCESS);
            } else {
                // For all other errors, just exit with the error
                err.exit();
            }
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

    handle_command(cli).await
}

async fn handle_command(cli: Cli) -> Result<ExitCode> {
    if cli.top_level.version.is_some() {
        handle_command_iner(
            cli.top_level.global_args,
            Box::new(Command::Version { output_format: VersionFormat::Text }),
        )
        .await?;
        return Ok(ExitCode::SUCCESS);
    }

    handle_command_iner(cli.top_level.global_args, cli.command).await
}

async fn handle_command_iner(
    global_args: Box<cli::GlobalArgs>,
    command: Box<Command>,
) -> std::result::Result<ExitCode, ErrReport> {
    use std::io::Write; // Add this import to bring Write trait into scope

    // Create a color config from user preference
    let color_config = color::ColorConfig::new(global_args.color.unwrap_or(ColorChoice::Auto));

    match *command {
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
        Command::Server => lsp::run()
            .await
            .wrap_err("Failed to run LSP server")
            .map(|_| ExitCode::SUCCESS)
            .map_err(Error::LspError),
        Command::Version { output_format } => {
            // Use the color config directly instead of setting it on VERSION
            let version = VERSION.clone();
            let should_colorize = color_config.should_colorize();

            // Get a properly colorized stdout writer
            let mut out = color_config.stdout();

            match output_format {
                VersionFormat::Text => {
                    // Create a version instance that respects color settings
                    let version_display = Version::new().with_colorization(should_colorize);
                    writeln!(out, "{}", version_display).into_diagnostic()?;
                }
                VersionFormat::Json => {
                    #[cfg(feature = "serde")]
                    {
                        let json = serde_json::to_string_pretty(&Version::new()).unwrap();
                        writeln!(out, "{json}").into_diagnostic()?;
                    }
                    #[cfg(not(feature = "serde"))]
                    {
                        writeln!(out, "JSON output is not supported in this build")
                            .into_diagnostic()?;
                    }
                }
                VersionFormat::Toml => writeln!(out, "{}", version).into_diagnostic()?,
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

    // Get color choice from CLI arguments
    let color_choice = cli.top_level.global_args.color.unwrap_or(ColorChoice::Auto);
    let color_config = color::ColorConfig::new(color_choice);
    let use_ansi = color_config.should_colorize();

    // Create a registry for our subscribers
    let registry = Registry::default();

    let mut stdout_subscriber = None;

    // Always add the stdout subscriber
    if !no_stdout_log {
        stdout_subscriber = Some(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stdout)
                .with_ansi(use_ansi) // Use the color config here
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

                // Ensure the parent directory exists
                if let Some(parent) = log_path.parent() {
                    if !parent.exists() {
                        if let Err(err) = std::fs::create_dir_all(parent) {
                            eprintln!(
                                "Failed to create log directory '{}': {}",
                                parent.display(),
                                err
                            );
                            // Fallback to stdout if we can't create the directory
                            return Box::new(std::io::stdout()) as Box<dyn std::io::Write + Send>;
                        }
                    }
                }

                // Try to open the log file
                match std::fs::OpenOptions::new().create(true).append(true).open(&log_path) {
                    Ok(file) => Box::new(file) as Box<dyn std::io::Write + Send>,
                    Err(err) => {
                        eprintln!("Failed to open log file '{}': {}", log_path.display(), err);
                        // Fallback to stdout if we can't open the file
                        Box::new(std::io::stdout()) as Box<dyn std::io::Write + Send>
                    }
                }
            })
            .with_ansi(false)
            .with_timer(UtcTime::rfc_3339());

        registry.with(file_subscriber).init();
        return;
    }

    registry.init();
}
