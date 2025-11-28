use std::borrow::Cow;

use crate::panspace::tok::Token;

/// A stateless parse context (lexer) scanning C source code.
#[derive(Debug, Clone)]
pub struct ParseContext<'a> {
    /// The source code being parsed.
    pub source: &'a [u8],
    /// The buffer for macro expansion tokens.
    pub macro_expand_buf: Cow<'a, Vec<Token<'a>>>,

    /// The current position in the source code.
    pub pos: usize,
    /// The current position in the macro expansion buffer.
    pub macro_expand_pos: usize,

    /// The current line number (1-based).
    pub line: u32,
    /// The current column number (1-based).
    pub col: u32
}

#[derive(Debug, Clone)]
pub struct ParseConfig {}

impl<'a> ParseContext<'a> {
    /// Creates a new parse context from the given source code.
    pub fn new(source: &'a [u8]) -> Self {
        Self {
            source,
            macro_expand_buf: Cow::Owned(Vec::new()),
            pos: 0,
            macro_expand_pos: 0,
            line: 1,
            col: 1
        }
    }
}

impl<'a> From<&'a [u8]> for ParseContext<'a> {
    /// Creates a new parse context from the given source code.
    fn from(source: &'a [u8]) -> Self {
        Self::new(source)
    }
}

impl<'a> From<&'a str> for ParseContext<'a> {
    /// Creates a new parse context from the given source code.
    fn from(source: &'a str) -> Self {
        Self::new(source.as_bytes())
    }
}
