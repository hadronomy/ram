use clap::builder::Styles;
use clap::builder::styling::{AnsiColor, Effects, Style};
use clap::{Args, Parser, Subcommand};

// Configures Clap v3-style help menu colors
const STYLES: Styles = Styles::styled()
    .header(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .usage(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .literal(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
    .placeholder(AnsiColor::Cyan.on_default());

#[derive(Parser)]
#[command(name = "ram", author, version = "0.1.0")]
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
    version: Option<bool>,
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
}

#[derive(Debug, Copy, Clone, clap::ValueEnum)]
pub enum ColorChoice {
    /// Enables colored output only when the output is going to a terminal or TTY with support.
    Auto,

    /// Enables colored output regardless of the detected environment.
    Always,

    /// Disables colored output.
    Never,
}

impl ColorChoice {
    /// Combine self (higher priority) with an [`anstream::ColorChoice`] (lower priority).
    ///
    /// This method allows prioritizing the user choice, while using the inferred choice for a
    /// stream as default.
    #[must_use]
    pub fn and_colorchoice(self, next: anstream::ColorChoice) -> Self {
        match self {
            Self::Auto => match next {
                anstream::ColorChoice::Auto => Self::Auto,
                anstream::ColorChoice::Always | anstream::ColorChoice::AlwaysAnsi => Self::Always,
                anstream::ColorChoice::Never => Self::Never,
            },
            Self::Always | Self::Never => self,
        }
    }
}

impl From<ColorChoice> for anstream::ColorChoice {
    fn from(value: ColorChoice) -> Self {
        match value {
            ColorChoice::Auto => Self::Auto,
            ColorChoice::Always => Self::Always,
            ColorChoice::Never => Self::Never,
        }
    }
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
        #[arg(long, value_enum, default_value = "text")]
        output_format: VersionFormat,
    },

    /// Run the Language Server Protocol (LSP) server.
    Lsp
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
