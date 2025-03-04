use std::io::{self};

use anstream::{self, AutoStream};

/// Global color configuration to manage colorized output
pub struct ColorConfig {
    choice: ColorChoice,
}

impl ColorConfig {
    /// Create a new color configuration based on user preference
    pub fn new(choice: ColorChoice) -> Self {
        Self { choice }
    }

    /// Get the effective color choice, considering both user preference and terminal capabilities
    pub fn effective_choice(&self) -> anstream::ColorChoice {
        self.choice.into()
    }

    /// Return a stdout writer that respects color configuration
    ///
    /// Note: Callers must make sure to have `std::io::Write` in scope when using this.
    pub fn stdout(&self) -> impl io::Write {
        AutoStream::new(std::io::stdout(), self.effective_choice())
    }

    /// Return a stderr writer that respects color configuration
    ///
    /// Note: Callers must make sure to have `std::io::Write` in scope when using this.
    pub fn stderr(&self) -> impl io::Write {
        AutoStream::new(std::io::stderr(), self.effective_choice())
    }

    /// Determine if colors should be used in the current context
    pub fn should_colorize(&self) -> bool {
        match self.choice {
            ColorChoice::Always => true,
            ColorChoice::Never => false,
            ColorChoice::Auto => {
                anstream::AutoStream::new(io::stdout(), anstream::ColorChoice::Auto).is_terminal()
            }
        }
    }
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
