use std::borrow::Cow;
use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// Represents a location in a source file.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize)]
pub struct SourceLoc<'a> {
    /// The source file name.
    pub file: Cow<'a, str>,
    /// The line number (1-based).
    pub line: usize,
    /// The column number (1-based).
    pub column: usize,
}

impl<'a> SourceLoc<'a> {
    /// Creates a new source location.
    pub fn new(file: Cow<'a, str>, line: usize, column: usize) -> Self {
        debug_assert_ne!(line, 0);
        debug_assert_ne!(column, 0);

        Self { file, line, column }
    }

    /// Creates a dummy source location.
    pub fn dummy() -> Self {
        Self {
            file: Cow::Borrowed("<unknown>"),
            line: 0,
            column: 0,
        }
    }

    /// Checks whether the source location is a dummy location.
    pub fn is_dummy(&self) -> bool {
        self.line == 0 || self.column == 0
    }
}

impl Display for SourceLoc<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_dummy() {
            write!(f, "<unknown>")
        } else {
            write!(f, "{}:{}:{}", self.file, self.line, self.column)
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Info
}

#[derive(Debug, Clone)]
pub struct Diagnostic<'a> {
    pub level: DiagnosticLevel,
    pub loc: SourceLoc<'a>,
    pub message: String
}

#[derive(Debug, Clone)]
pub struct DiagnosticContext<'a> {
    pub diagnostics: Vec<Diagnostic<'a>>
}
