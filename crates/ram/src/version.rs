use core::fmt;
use std::sync::LazyLock;

use owo_colors::{OwoColorize, Style};
use serde::Serialize;
use shadow_rs::formatcp;
use taplo::formatter::Options;
use taplo::rowan::{NodeOrToken, WalkEvent};

use crate::build;
use crate::cli::ColorChoice;

/// Global VERSION instance that can be accessed from anywhere
pub static VERSION: LazyLock<Version<'static>> = LazyLock::new(Version::default);

#[derive(Debug, Clone, Serialize)]
pub struct Version<'a> {
    #[serde(rename = "version")]
    pkg_version: &'a str,
    branch: &'a str,
    commit_hash: &'a str,
    build_time: &'a str,
    build_env: &'a str,
    build_channel: &'a str,
    #[serde(skip)]
    styles: Styles,
    #[serde(skip)]
    color_choice: ColorChoice,
}

impl Version<'_> {
    /// Creates a new Version instance with default (non-colored) styling
    pub fn new() -> Self {
        #[allow(clippy::const_is_empty)]
        let pkg_version = if build::TAG.is_empty() {
            formatcp!("{}-dev", build::PKG_VERSION)
        } else {
            build::PKG_VERSION
        };

        #[allow(clippy::const_is_empty)]
        let commit_hash = if !build::GIT_CLEAN {
            formatcp!("{}+", build::SHORT_COMMIT)
        } else {
            build::SHORT_COMMIT
        };

        Self {
            pkg_version,
            branch: build::BRANCH,
            commit_hash,
            build_time: build::BUILD_TIME,
            build_env: build::RUST_VERSION,
            build_channel: build::RUST_CHANNEL,
            styles: Styles::default(),
            color_choice: ColorChoice::Auto,
        }
    }

    /// Returns the package version string
    pub fn pkg_version(&self) -> &str {
        self.pkg_version
    }

    /// Returns the git branch
    pub fn branch(&self) -> &str {
        self.branch
    }

    /// Returns the commit hash
    pub fn commit_hash(&self) -> &str {
        self.commit_hash
    }

    /// Returns the build time
    pub fn build_time(&self) -> &str {
        self.build_time
    }

    /// Returns the build environment (Rust version)
    pub fn build_env(&self) -> &str {
        self.build_env
    }

    /// Returns the Rust channel
    pub fn build_channel(&self) -> &str {
        self.build_channel
    }

    /// Set the color choice for this version
    pub fn with_color_choice(mut self, choice: ColorChoice) -> Self {
        self.color_choice = choice;
        self
    }

    /// Private helper to determine if we should colorize based on the context
    fn should_colorize(&self) -> bool {
        match self.color_choice {
            ColorChoice::Always => true,
            ColorChoice::Never => false,
            ColorChoice::Auto => true, // FIXME: Implement auto-detection
        }
    }
}

impl Default for Version<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Version<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let toml = toml::to_string(&self).map_err(|_| fmt::Error)?;

        if !self.should_colorize() {
            return write!(f, "{}", toml);
        }

        let formatted =
            taplo::formatter::format(&toml, Options { align_entries: true, ..Default::default() });
        let syntax = taplo::parser::parse(&formatted);
        if !syntax.errors.is_empty() {
            return Err(fmt::Error);
        }

        let mut output = String::with_capacity(formatted.len());

        let tokens = syntax.into_syntax().preorder_with_tokens().filter_map(|event| match event {
            WalkEvent::Enter(NodeOrToken::Token(token)) => Some(token),
            _ => None,
        });

        for token in tokens {
            let text = token.text();
            match token.kind() {
                taplo::syntax::SyntaxKind::STRING
                | taplo::syntax::SyntaxKind::INTEGER
                | taplo::syntax::SyntaxKind::FLOAT => {
                    output.push_str(&text.style(self.styles.value_style).to_string());
                }
                taplo::syntax::SyntaxKind::IDENT | taplo::syntax::SyntaxKind::PERIOD => {
                    output.push_str(&text.style(self.styles.name_style).to_string());
                }
                _ => output.push_str(text),
            }
        }

        write!(f, "{}", output)
    }
}

#[derive(Debug, Clone, Copy)]
struct Styles {
    name_style: Style,
    value_style: Style,
}

impl Default for Styles {
    fn default() -> Self {
        Self { name_style: Style::new().blue(), value_style: Style::new().yellow() }
    }
}
