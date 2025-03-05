use std::ffi::OsString;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::{Arc, RwLock};

use anstream::println;
use chumsky::{Parser as ChumskyParser, stream};
use clap::{CommandFactory, Parser};
use miette::*;
use shadow_rs::shadow;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry, reload};

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
        Command::Server => {
            tracing_controls.set_stdout_enabled(false);
            lsp::run()
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

pub type LogFilterReloadHandle = reload::Handle<EnvFilter, Registry>;

/// Struct to hold handles for controlling layers at runtime
#[derive(Clone)]
pub struct TracingControls {
    pub filter_reload: LogFilterReloadHandle,
    pub stdout_enabled: Arc<RwLock<bool>>,
    pub file_enabled: Arc<RwLock<bool>>,
    pub log_path: Arc<RwLock<Option<PathBuf>>>,
    pub use_ansi: Arc<RwLock<bool>>,
}

impl TracingControls {
    /// Initialize tracing with default settings and return the controls
    pub fn new() -> Self {
        // Create default settings
        let stdout_enabled = Arc::new(RwLock::new(true));
        let file_enabled = Arc::new(RwLock::new(false));
        let log_path = Arc::new(RwLock::new(None));
        let use_ansi = Arc::new(RwLock::new(true));

        // Create a registry for our subscribers
        let registry = Registry::default();

        // Create the initial filter with default level
        let initial_filter = EnvFilter::from_default_env().add_directive(LevelFilter::INFO.into());

        // Create a reloadable filter layer
        let (filter_layer, filter_reload) = reload::Layer::new(initial_filter);

        // Start with base registry + filter
        let subscriber = registry.with(filter_layer);

        // Create conditional stdout layer
        let stdout_enabled_clone = Arc::clone(&stdout_enabled);
        let use_ansi_clone = Arc::clone(&use_ansi);
        let stdout_layer = tracing_subscriber::fmt::layer()
            .with_writer(move || {
                let enabled = Arc::clone(&stdout_enabled_clone);
                Box::new(ConditionalWriter::new(io::stdout(), enabled)) as Box<dyn Write + Send>
            })
            .with_ansi(*use_ansi_clone.read().unwrap())
            .with_timer(UtcTime::rfc_3339());

        // Create conditional file layer
        let file_enabled_clone = Arc::clone(&file_enabled);
        let log_path_clone = Arc::clone(&log_path);
        let file_layer = tracing_subscriber::fmt::layer()
            .with_writer(move || {
                let enabled = Arc::clone(&file_enabled_clone);
                let path = log_path_clone.read().unwrap().clone();

                if let Some(file_path) = path {
                    Box::new(ConditionalWriter::new(create_file_writer(&file_path), enabled))
                        as Box<dyn Write + Send>
                } else {
                    // If no path is set, create a no-op writer
                    Box::new(ConditionalWriter::new(
                        NullWriter,
                        Arc::new(RwLock::new(false)), // Always disabled
                    )) as Box<dyn Write + Send>
                }
            })
            .with_ansi(false)
            .with_timer(UtcTime::rfc_3339());

        // Initialize the subscriber with all configured layers
        subscriber.with(stdout_layer).with(file_layer).init();

        // Return handles to control logging at runtime
        Self { filter_reload, stdout_enabled, file_enabled, log_path, use_ansi }
    }

    /// Create tracing controls from CLI arguments
    pub fn from_cli(cli: &Cli) -> Self {
        let controls = Self::new();
        controls.update_from_cli(cli);
        controls
    }

    /// Enable or disable stdout logging
    pub fn set_stdout_enabled(&self, enabled: bool) {
        let mut state = self.stdout_enabled.write().unwrap();
        *state = enabled;
    }

    /// Enable or disable file logging
    pub fn set_file_enabled(&self, enabled: bool) {
        let mut state = self.file_enabled.write().unwrap();
        *state = enabled;
    }

    /// Change the log file path
    pub fn set_log_path(&self, path: Option<PathBuf>) {
        let mut state = self.log_path.write().unwrap();
        *state = path;
    }

    /// Set whether to use ANSI colors
    pub fn set_use_ansi(&self, use_ansi: bool) {
        let mut state = self.use_ansi.write().unwrap();
        *state = use_ansi;
    }

    /// Get current stdout logging state
    pub fn is_stdout_enabled(&self) -> bool {
        *self.stdout_enabled.read().unwrap()
    }

    /// Get current file logging state
    pub fn is_file_enabled(&self) -> bool {
        *self.file_enabled.read().unwrap()
    }

    /// Update the log filter level
    pub fn set_log_level(&self, level: LevelFilter) -> Result<(), reload::Error> {
        let mut filter = EnvFilter::from_default_env();
        filter = filter.add_directive(level.into());
        self.filter_reload.reload(filter)
    }

    /// Get the current log path
    pub fn log_path(&self) -> Option<PathBuf> {
        self.log_path.read().unwrap().clone()
    }

    /// Update controls from CLI arguments
    pub fn update_from_cli(&self, cli: &Cli) {
        // Update log level
        let verbosity = cli.top_level.global_args.verbose;
        let level = match verbosity {
            0 => LevelFilter::WARN,
            1 => LevelFilter::INFO,
            2 => LevelFilter::DEBUG,
            _ => LevelFilter::TRACE,
        };
        let _ = self.set_log_level(level); // Ignore errors

        // Update ANSI colors
        let color_choice = cli.top_level.global_args.color.unwrap_or(ColorChoice::Auto);
        let color_config = color::ColorConfig::new(color_choice);
        self.set_use_ansi(color_config.should_colorize());

        // Update log path
        let log_path = cli.top_level.global_args.mirror.clone();
        self.set_log_path(log_path.clone());
        self.set_file_enabled(log_path.is_some());
    }
}

// Implement Default trait for TracingControls
impl Default for TracingControls {
    fn default() -> Self {
        Self::new()
    }
}

// For backward compatibility
pub fn init_tracing() -> TracingControls {
    TracingControls::new()
}

// For backward compatibility
pub fn init_tracing_from_cli(cli: &Cli) -> TracingControls {
    TracingControls::from_cli(cli)
}

/// A no-op writer that discards all output
struct NullWriter;

impl Write for NullWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// A writer that can be enabled/disabled at runtime
struct ConditionalWriter<W: Write> {
    inner: W,
    enabled: Arc<RwLock<bool>>,
}

impl<W: Write> ConditionalWriter<W> {
    fn new(inner: W, enabled: Arc<RwLock<bool>>) -> Self {
        Self { inner, enabled }
    }
}

impl<W: Write> Write for ConditionalWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // Only write if enabled
        if *self.enabled.read().unwrap() {
            self.inner.write(buf)
        } else {
            // Pretend we wrote successfully
            Ok(buf.len())
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        if *self.enabled.read().unwrap() { self.inner.flush() } else { Ok(()) }
    }
}

/// Creates a file writer with fallback to stdout on error
fn create_file_writer(log_path: &PathBuf) -> Box<dyn Write + Send> {
    // Ensure the parent directory exists
    if let Some(parent) = log_path.parent() {
        if !parent.exists() {
            if let Err(err) = std::fs::create_dir_all(parent) {
                eprintln!("Failed to create log directory '{}': {}", parent.display(), err);
                // Fallback to stdout if we can't create the directory
                return Box::new(io::stdout());
            }
        }
    }

    // Try to open the log file
    match std::fs::OpenOptions::new().create(true).append(true).open(log_path) {
        Ok(file) => Box::new(file),
        Err(err) => {
            eprintln!("Failed to open log file '{}': {}", log_path.display(), err);
            // Fallback to stdout if we can't open the file
            Box::new(io::stdout())
        }
    }
}
