//! Diagnostic types for the RAM parser.
//!
//! This module provides diagnostic types that are used to report errors,
//! warnings, and other information during parsing.
//!
//! # Examples
//!
//! Creating a simple error diagnostic:
//!
//! ```
//! use ram_parser::Diagnostic;
//! use std::ops::Range;
//!
//! let error = Diagnostic::error("Invalid syntax".to_string(), "Try fixing this".to_string(), 0..5);
//! ```
//!
//! Using the builder API for more complex diagnostics:
//!
//! ```
//! use ram_parser::{Diagnostic, DiagnosticKind};
//! use std::ops::Range;
//!
//! let error = Diagnostic::builder()
//!     .with_message("Invalid syntax")
//!     .with_help("Try fixing this")
//!     .with_kind(DiagnosticKind::Error)
//!     .with_primary_span(0..5, "here")
//!     .with_secondary_span(10..15, "related to this")
//!     .with_code("E001")
//!     .build();
//! ```

use std::ops::Range;

use ariadne::{Color, Label, Report, ReportKind};

/// A diagnostic type used during parsing.
/// This is compatible with ariadne's Report type and can be converted to ram_error::SingleParserError.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// The error message.
    pub message: String,
    /// Additional help text.
    pub help: String,
    /// The labeled spans for this error.
    pub labeled_spans: Vec<(Range<usize>, String)>,
    /// The kind of diagnostic (error, warning, advice, etc.)
    pub kind: DiagnosticKind,
    /// Optional code for the diagnostic
    pub code: Option<String>,
    /// Optional notes to provide additional context
    pub notes: Vec<String>,
}

/// The kind of diagnostic being reported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticKind {
    /// A critical error that prevents the program from continuing.
    Error,
    /// A warning about a potential issue, but not critical.
    Warning,
    /// Advice about code improvements or best practices.
    Advice,
    /// A custom diagnostic kind with a specific name and color.
    Custom(&'static str, Color),
}

impl Diagnostic {
    /// Create a new error diagnostic.
    pub fn error(message: String, help: String, span: Range<usize>) -> Self {
        Self {
            message,
            help,
            labeled_spans: vec![(span, "here".to_string())],
            kind: DiagnosticKind::Error,
            code: None,
            notes: Vec::new(),
        }
    }

    /// Create a new warning diagnostic.
    pub fn warning(message: String, help: String, span: Range<usize>) -> Self {
        Self {
            message,
            help,
            labeled_spans: vec![(span, "here".to_string())],
            kind: DiagnosticKind::Warning,
            code: None,
            notes: Vec::new(),
        }
    }

    /// Create a new advice diagnostic.
    pub fn advice(message: String, help: String, span: Range<usize>) -> Self {
        Self {
            message,
            help,
            labeled_spans: vec![(span, "here".to_string())],
            kind: DiagnosticKind::Advice,
            code: None,
            notes: Vec::new(),
        }
    }

    /// Create a new diagnostic with multiple labeled spans.
    #[must_use]
    pub fn with_labeled_spans(mut self, spans: Vec<(Range<usize>, String)>) -> Self {
        self.labeled_spans = spans;
        self
    }

    /// Add a code to this diagnostic.
    #[must_use]
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Add a note to this diagnostic.
    #[must_use]
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    /// Add multiple notes to this diagnostic.
    #[must_use]
    pub fn with_notes(mut self, notes: Vec<String>) -> Self {
        self.notes.extend(notes);
        self
    }

    /// Create a new diagnostic builder.
    pub fn builder() -> DiagnosticBuilder {
        DiagnosticBuilder::new()
    }

    /// Convert this diagnostic to an ariadne Report.
    pub fn to_report(&self) -> Report<Range<usize>> {
        let kind = match self.kind {
            DiagnosticKind::Error => ReportKind::Error,
            DiagnosticKind::Warning => ReportKind::Warning,
            DiagnosticKind::Advice => ReportKind::Advice,
            DiagnosticKind::Custom(name, color) => ReportKind::Custom(name, color),
        };

        // Create the report with the primary span
        let primary_span = self.labeled_spans[0].0.clone();
        let mut report = Report::build(kind, primary_span.clone()).with_message(&self.message);

        if !self.help.is_empty() {
            report = report.with_help(&self.help);
        }

        if let Some(code) = &self.code {
            report = report.with_code(code);
        }

        // Add all labeled spans
        for (span, label) in &self.labeled_spans {
            report = report.with_label(Label::new(span.clone()).with_message(label));
        }

        // Add all notes
        for note in &self.notes {
            report = report.with_note(note);
        }

        report.finish()
    }
}

/// A builder for creating diagnostics with a fluent API.
#[derive(Debug, Default)]
pub struct DiagnosticBuilder {
    /// The error message.
    message: Option<String>,
    /// Additional help text.
    help: Option<String>,
    /// The labeled spans for this error.
    labeled_spans: Vec<(Range<usize>, String)>,
    /// The kind of diagnostic (error, warning, advice, etc.)
    kind: Option<DiagnosticKind>,
    /// Optional code for the diagnostic
    code: Option<String>,
    /// Optional notes to provide additional context
    notes: Vec<String>,
}

impl DiagnosticBuilder {
    /// Create a new diagnostic builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the message for the diagnostic.
    #[must_use]
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Set the help text for the diagnostic.
    #[must_use]
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Set the kind of diagnostic.
    #[must_use]
    pub fn with_kind(mut self, kind: DiagnosticKind) -> Self {
        self.kind = Some(kind);
        self
    }

    /// Add a primary span to the diagnostic.
    /// This will be the first span in the labeled_spans list.
    #[must_use]
    pub fn with_primary_span(mut self, span: Range<usize>, label: impl Into<String>) -> Self {
        // If this is the first span, add it normally
        if self.labeled_spans.is_empty() {
            self.labeled_spans.push((span, label.into()));
        } else {
            // Otherwise, insert it at the beginning
            self.labeled_spans.insert(0, (span, label.into()));
        }
        self
    }

    /// Add a secondary span to the diagnostic.
    #[must_use]
    pub fn with_secondary_span(mut self, span: Range<usize>, label: impl Into<String>) -> Self {
        self.labeled_spans.push((span, label.into()));
        self
    }

    /// Add multiple spans to the diagnostic.
    #[must_use]
    pub fn with_spans(mut self, spans: Vec<(Range<usize>, String)>) -> Self {
        self.labeled_spans.extend(spans);
        self
    }

    /// Set the code for the diagnostic.
    #[must_use]
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Add a note to the diagnostic.
    #[must_use]
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    /// Add multiple notes to the diagnostic.
    #[must_use]
    pub fn with_notes(mut self, notes: Vec<String>) -> Self {
        self.notes.extend(notes);
        self
    }

    /// Build the diagnostic.
    ///
    /// # Panics
    ///
    /// Panics if the message, kind, or primary span is not set.
    pub fn build(self) -> Diagnostic {
        let message = self.message.expect("Diagnostic message must be set");
        let kind = self.kind.expect("Diagnostic kind must be set");

        assert!(!self.labeled_spans.is_empty(), "Diagnostic must have at least one span");

        Diagnostic {
            message,
            help: self.help.unwrap_or_default(),
            labeled_spans: self.labeled_spans,
            kind,
            code: self.code,
            notes: self.notes,
        }
    }

    /// Build the diagnostic as an error.
    ///
    /// This is a convenience method that sets the kind to Error and builds the diagnostic.
    ///
    /// # Panics
    ///
    /// Panics if the message or primary span is not set.
    pub fn build_error(self) -> Diagnostic {
        self.with_kind(DiagnosticKind::Error).build()
    }

    /// Build the diagnostic as a warning.
    ///
    /// This is a convenience method that sets the kind to Warning and builds the diagnostic.
    ///
    /// # Panics
    ///
    /// Panics if the message or primary span is not set.
    pub fn build_warning(self) -> Diagnostic {
        self.with_kind(DiagnosticKind::Warning).build()
    }

    /// Build the diagnostic as advice.
    ///
    /// This is a convenience method that sets the kind to Advice and builds the diagnostic.
    ///
    /// # Panics
    ///
    /// Panics if the message or primary span is not set.
    pub fn build_advice(self) -> Diagnostic {
        self.with_kind(DiagnosticKind::Advice).build()
    }
}

/// Convert internal Diagnostic to ram_error types.
///
/// This function converts our internal Diagnostic to the ram_error types
/// that can be used with miette for nice error reporting.
pub fn convert_errors(source: &str, errors: Vec<Diagnostic>) -> ram_error::ParserError {
    use miette::LabeledSpan;
    use ram_error::{ParserError, SingleParserError};

    // Convert each Diagnostic to a SingleParserError
    let single_errors = errors
        .into_iter()
        .map(|e| {
            // Convert labeled spans to miette LabeledSpans
            let labels = e
                .labeled_spans
                .iter()
                .map(|(span, label)| {
                    LabeledSpan::new(Some(label.clone()), span.start, span.end - span.start)
                })
                .collect::<Vec<_>>();

            // Create a SingleParserError with appropriate message based on diagnostic kind
            let message = match e.kind {
                DiagnosticKind::Error => format!("Error: {}", e.message),
                DiagnosticKind::Warning => format!("Warning: {}", e.message),
                DiagnosticKind::Advice => format!("Advice: {}", e.message),
                DiagnosticKind::Custom(name, _) => format!("{}: {}", name, e.message),
            };

            SingleParserError { message, labels }
        })
        .collect();

    // Create a ParserError with all the SingleParserErrors
    ParserError {
        src: miette::NamedSource::new("input.ram", source.to_string()),
        errors: single_errors,
    }
}
