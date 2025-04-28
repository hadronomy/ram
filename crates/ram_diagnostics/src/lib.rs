//! Diagnostic types for the RAM compiler.
//!
//! This module provides diagnostic types that are used to report errors,
//! warnings, and other information during compilation.
//!
//! # Examples
//!
//! Creating a simple error diagnostic:
//!
//! ```
//! use ram_diagnostics::Diagnostic;
//! use std::ops::Range;
//!
//! let error = Diagnostic::error("Invalid syntax".to_string(), "Try fixing this".to_string(), 0..5);
//! ```
//!
//! Using the builder API for more complex diagnostics:
//!
//! ```
//! use ram_diagnostics::{Diagnostic, DiagnosticKind};
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

/// A diagnostic type used during compilation.
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
    /// A custom diagnostic kind with a specific name.
    Custom(&'static str),
}

impl From<DiagnosticKind> for miette::Severity {
    fn from(kind: DiagnosticKind) -> Self {
        match kind {
            DiagnosticKind::Custom(_) | DiagnosticKind::Error => miette::Severity::Error,
            DiagnosticKind::Warning => miette::Severity::Warning,
            DiagnosticKind::Advice => miette::Severity::Advice,
        }
    }
}

impl Diagnostic {
    /// Create a new error diagnostic.
    pub fn error(message: impl Into<String>, help: impl Into<String>, span: Range<usize>) -> Self {
        Self {
            message: message.into(),
            help: help.into(),
            labeled_spans: vec![(span, "here".to_string())],
            kind: DiagnosticKind::Error,
            code: None,
            notes: Vec::new(),
        }
    }

    /// Create a new warning diagnostic.
    pub fn warning(
        message: impl Into<String>,
        help: impl Into<String>,
        span: Range<usize>,
    ) -> Self {
        Self {
            message: message.into(),
            help: help.into(),
            labeled_spans: vec![(span, "here".to_string())],
            kind: DiagnosticKind::Warning,
            code: None,
            notes: Vec::new(),
        }
    }

    /// Create a new advice diagnostic.
    pub fn advice(message: impl Into<String>, help: impl Into<String>, span: Range<usize>) -> Self {
        Self {
            message: message.into(),
            help: help.into(),
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

    /// Set the kind for the diagnostic.
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
pub fn convert_errors(source: &str, errors: Vec<Diagnostic>) -> ram_error::Report {
    use miette::LabeledSpan;
    use ram_error::{Report, SingleReport};

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
                DiagnosticKind::Custom(name) => format!("{}: {}", name, e.message),
            };

            SingleReport { 
                message, 
                labels, 
                severity: Some(e.kind.into()), 
                code: e.code 
            }
        })
        .collect();

    // Create a ParserError with all the SingleParserErrors
    Report { 
        src: miette::NamedSource::new("input.ram", source.to_string()), 
        errors: single_errors 
    }
}

/// A collection of diagnostics
#[derive(Debug, Clone, Default)]
pub struct DiagnosticCollection {
    /// List of diagnostics
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticCollection {
    /// Create a new empty diagnostic collection
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }
    
    /// Add a diagnostic to the collection
    pub fn add(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }
    
    /// Add an error diagnostic
    pub fn error(&mut self, message: impl Into<String>, help: impl Into<String>, span: Option<Range<usize>>) {
        let span = span.unwrap_or(0..0);
        self.add(Diagnostic::error(message, help, span));
    }
    
    /// Add a warning diagnostic
    pub fn warning(&mut self, message: impl Into<String>, help: impl Into<String>, span: Option<Range<usize>>) {
        let span = span.unwrap_or(0..0);
        self.add(Diagnostic::warning(message, help, span));
    }
    
    /// Add an info diagnostic
    pub fn info(&mut self, message: impl Into<String>, help: impl Into<String>, span: Option<Range<usize>>) {
        let span = span.unwrap_or(0..0);
        self.add(Diagnostic::advice(message, help, span));
    }
    
    /// Add a hint diagnostic
    pub fn hint(&mut self, message: impl Into<String>, help: impl Into<String>, span: Option<Range<usize>>) {
        let span = span.unwrap_or(0..0);
        self.add(Diagnostic::advice(message, help, span));
    }
    
    /// Check if the collection has any diagnostics
    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }
    
    /// Check if the collection has any errors
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.kind == DiagnosticKind::Error)
    }
    
    /// Get all diagnostics
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
    
    /// Get the number of diagnostics
    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }
    
    /// Extend this collection with diagnostics from another collection
    pub fn extend(&mut self, other: DiagnosticCollection) {
        self.diagnostics.extend(other.diagnostics);
    }
    
    /// Convert to a ram_error::Report
    pub fn to_report(&self, source: &str) -> ram_error::Report {
        convert_errors(source, self.diagnostics.clone())
    }
}
