use std::ffi::OsString;
use std::io::Write;
use std::process::ExitCode;

use anstream::println;
use clap::{CommandFactory, Parser};
use human_panic::{Metadata, setup_panic};
use miette::*;
use ram_error::Error;
use serde::Serialize;
use shadow_rs::shadow;
use tracing::{debug, error};

use crate::cli::{Cli, Command, VersionFormat};
use crate::color::ColorChoice;
use crate::tracing_setup::TracingControls;
pub use crate::tracing_setup::{init_tracing, init_tracing_from_cli};
pub use crate::version::*;

pub mod cli;
pub mod color;
pub mod error;
pub mod language;
pub mod run;
pub mod tracing_setup;
pub mod version;

shadow!(build);

/// Main entry point for the application.
///
/// Initializes tracing, parses command-line arguments, configures error reporting,
/// and delegates to the appropriate command handler.
///
/// # Arguments
///
/// * `args` - An iterator of command-line arguments that can be converted to `OsString`
///
/// # Returns
///
/// * `Result<ExitCode>` - Success or failure exit code wrapped in a Result
///
/// # Errors
///
/// Returns an error if initialization fails or if command execution fails.
/// Special handling is provided for --version flags.
pub async fn main<Args, T>(args: Args) -> Result<ExitCode>
where
    Args: Iterator<Item = T>,
    T: Into<OsString> + Clone,
{
    setup_panic!(
        Metadata::new(build::PROJECT_NAME, VERSION.pkg_version())
            .authors("Pablo Hernandez <hadronomy@gmail.com>")
            .homepage("hadronomy.com")
            .support("- Open an issue on GitHub: https://github.com/hadronomy/ram/issues/new")
    );

    let tracing_controls = init_tracing();

    let cli = match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(err) => {
            // If error is --version or -V, we'll handle that specially
            if err.kind() == clap::error::ErrorKind::DisplayVersion {
                // Create a default CLI and run the version command
                let default_cli = Cli::parse_from(["ram", "version"]);
                handle_command(default_cli, &tracing_controls).await?;
                return Ok(ExitCode::SUCCESS);
            } else {
                // For all other errors, just exit with the error
                err.exit();
            }
        }
    };

    miette::set_hook(Box::new(|_| {
        Box::new(
            miette::MietteHandlerOpts::new()
                .break_words(false)
                .show_related_errors_as_nested()
                .word_separator(textwrap::WordSeparator::AsciiSpace)
                .word_splitter(textwrap::WordSplitter::NoHyphenation)
                .with_syntax_highlighting(language::highlighter())
                .tab_width(2)
                .build(),
        )
    }))
    .map_err(|err| Error::SetupError(err.into()))?;

    handle_command(cli, &tracing_controls).await
}

async fn handle_command(cli: Cli, tracing_controls: &TracingControls) -> Result<ExitCode> {
    tracing_controls.update_from_cli(&cli);

    if cli.top_level.version.is_some() {
        handle_command_iner(
            cli.top_level.global_args,
            Box::new(Command::Version { output_format: VersionFormat::Text }),
            tracing_controls,
        )
        .await?;
        return Ok(ExitCode::SUCCESS);
    }

    handle_command_iner(cli.top_level.global_args, cli.command, tracing_controls).await
}

async fn handle_command_iner(
    global_args: Box<cli::GlobalArgs>,
    command: Box<Command>,
    tracing_controls: &TracingControls,
) -> std::result::Result<ExitCode, ErrReport> {
    use std::io::Write;

    // Create a color config from user preference
    let color_config = color::ColorConfig::new(global_args.color.unwrap_or(ColorChoice::Auto));

    match *command {
        // execute help
        Command::Help(_) => {
            Cli::command().print_help().into_diagnostic()?;
            Ok::<_, Error>(ExitCode::SUCCESS)
        }
        Command::Validate { program, ast, reprint, show_pipeline, show_cfg, show_hir } => {
            let src = std::fs::read_to_string(program.clone())
                .into_diagnostic()
                .wrap_err(format!("Failed to read file: {}", program))?;
            let (program, body, pipeline, context, errors) = language::parser()(&src);

            // Report any errors
            for error in errors {
                eprintln!("{:?}", error);
            }

            if ast {
                // Just print the debug representation of the program
                println!("{program:#?}");
            }

            if reprint {
                // Print the program back out
                println!("{program}");
            }

            if show_hir {
                println!("{body:#?}");
            }

            if show_cfg {
                // Get the control flow graph from the context
                if let Ok(cfg) =
                    context.get_result::<hir_analysis::analyzers::ControlFlowAnalysis>()
                {
                    // Convert the CFG to a mermaid diagram with detailed instruction information
                    let mermaid = cfg.to_mermaid_with_context(&context);
                    open_mermaid(mermaid)?;
                } else {
                    error!("Failed to get control flow graph from context");
                }
            }

            if show_pipeline {
                open_mermaid(pipeline.export_dependency_graph(
                    hir_analysis::ExportFormat::Mermaid,
                    &Default::default(),
                ))?;
            }

            Ok::<_, Error>(ExitCode::SUCCESS)
        }
        Command::Run { program, input: _, memory: _ } => {
            let program_path = std::path::Path::new(&program);
            run::run_program(program_path, None, None)
                .map(|_| ExitCode::SUCCESS)
                .map_err(Error::RunError)
        }
        Command::Server => {
            tracing_controls.set_stdout_enabled(false);
            ram_lsp::run()
                .await
                .wrap_err("Failed to run LSP server")
                .map(|_| ExitCode::SUCCESS)
                .map_err(Error::LspError)
        }
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

fn open_mermaid(mermaid_str: impl Into<String>) -> Result<(), ErrReport> {
    // TODO: Use [https://mermaid.live] for now
    use base64::Engine;
    use base64::engine::general_purpose;
    use flate2::Compression;
    use flate2::write::ZlibEncoder;
    #[derive(Serialize)]
    struct Pan {
        x: f64,
        y: f64,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Payload {
        code: String,
        grid: bool,
        mermaid: String,
        pan_zoom: bool,
        rough: bool,
        update_diagram: bool,
        render_count: u32,
        pan: Pan,
        zoom: f64,
        editor_mode: String,
    }
    let payload = Payload {
        code: mermaid_str.into(),
        grid: true,
        mermaid: "{\n  \"theme\": \"dark\"\n}".to_string(),
        pan_zoom: true,
        rough: false,
        update_diagram: true,
        render_count: 85,
        pan: Pan { x: 181.0, y: 181.0 },
        zoom: 0.7,
        editor_mode: "code".to_string(),
    };
    let json_bytes = serde_json::to_vec(&payload).into_diagnostic()?;
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&json_bytes).into_diagnostic()?;
    let compressed = encoder.finish().into_diagnostic()?;
    let encoded = general_purpose::STANDARD.encode(compressed);
    let url = format!("https://mermaid.live/edit#pako:{}", encoded);
    debug!("Opening URL: {}", url);
    open::that(url).into_diagnostic()?;
    Ok(())
}
