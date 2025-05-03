//! Tracing and logging configuration for the RAM application.
//!
//! This module provides utilities for configuring and controlling tracing
//! at runtime, including conditional writers that can be enabled/disabled
//! and file output support.

use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry, reload};

use crate::cli::Cli;
use crate::color;
use crate::color::ColorChoice;

/// Type alias for the log filter reload handle
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

/// Initialize tracing with default settings and return the controls
pub fn init_tracing() -> TracingControls {
    TracingControls::new()
}

/// Initialize tracing from CLI arguments and return the controls
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
