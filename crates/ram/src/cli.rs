use std::path::PathBuf;

use clap::builder::Styles;
use clap::builder::styling::{AnsiColor, Effects, Style};
use clap::{Args, Parser, Subcommand};

use crate::VERSION;
use crate::color::ColorChoice;

// Configures Clap v3-style help menu colors
const STYLES: Styles = Styles::styled()
    .header(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .usage(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .literal(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
    .placeholder(AnsiColor::Cyan.on_default());

#[derive(Parser)]
#[command(name = "ram", author, version = VERSION.pkg_version())]
#[command(about = "The ram language toolkit")]
#[command(propagate_version = true)]
#[command(
    after_help = "Use `ram help` for more details.",
    after_long_help = "",
    disable_help_subcommand = true,
    disable_version_flag = true
)]
#[command(styles=STYLES)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Box<Command>,

    #[command(flatten)]
    pub top_level: TopLevelArgs,
}

#[derive(Subcommand, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum Command {
    /// Display documentation for a command.
    #[command(help_template = "\
{about-with-newline}
{usage-heading} {usage}{after-help}
",
        after_help = format!("\
{heading}Options:{heading:#}
  {option}--no-pager{option:#} Disable pager when printing help
",
            heading = Style::new().bold().underline(),
            option = Style::new().bold(),
        ),
    )]
    Help(HelpArgs),

    /// Display the version.
    Version {
        #[arg(long, short = 'f', value_enum, default_value = "text")]
        output_format: VersionFormat,
    },

    /// Run the Language Server Protocol (LSP) server.
    #[command(alias = "lsp")]
    Server,

    /// Validate a RAM file.
    Validate {
        /// The file to validate.
        program: String,

        /// Output the ast as JSON.
        #[arg(long, short, action)]
        ast: bool,

        #[arg(long, short, action)]
        reprint: bool,
    },

    /// Run a RAM program in the virtual machine.
    Run {
        /// The RAM program file to execute.
        program: String,

        /// Input values to provide to the program (space-separated).
        #[arg(long, short, value_delimiter = ' ')]
        input: Option<Vec<i64>>,

        /// Show memory contents after execution.
        #[arg(long, short, action)]
        memory: bool,
    },
}

#[derive(Parser)]
#[command(disable_help_flag = true, disable_version_flag = true)]
pub struct TopLevelArgs {
    #[command(flatten)]
    pub global_args: Box<GlobalArgs>,

    /// Display the concise help for this command.
    #[arg(global = true, short, long, action = clap::ArgAction::HelpShort, help_heading = "Global options")]
    help: Option<bool>,

    /// Display the version.
    #[arg(global = true, short = 'V', long, action = clap::ArgAction::Version, help_heading = "Global options")]
    pub version: Option<bool>,
}

#[derive(Parser, Debug, Clone)]
#[command(next_help_heading = "Global options", next_display_order = 1000)]
pub struct GlobalArgs {
    /// Do not print any output.
    #[arg(global = true, long, short, conflicts_with = "verbose")]
    pub quiet: bool,

    /// Use verbose output.
    ///
    /// You can configure fine-grained logging using the `RUST_LOG` environment variable.
    /// (<https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#directives>)
    #[arg(global = true, action = clap::ArgAction::Count, long, short, conflicts_with = "quiet")]
    pub verbose: u8,

    /// Mirror logs to a specified file.
    #[arg(global = true, long, value_name = "FILE")]
    pub mirror: Option<PathBuf>,

    /// Control the use of color in output.
    ///
    /// By default, uv will automatically detect support for colors when writing to a terminal.
    #[arg(global = true, long, value_enum, value_name = "COLOR_CHOICE")]
    pub color: Option<ColorChoice>,
}

#[derive(Args, Clone)]
pub struct HelpArgs {
    /// Disable pager when printing help
    #[arg(long)]
    pub no_pager: bool,

    pub command: Option<Vec<String>>,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum VersionFormat {
    /// Display the version as a plain text.
    Text,
    /// Display the version as a JSON.
    Json,
    /// Display the version as a TOML.
    Toml,
}
