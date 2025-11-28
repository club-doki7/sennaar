use std::borrow::Cow;

use crate::panspace::tok::Token;

#[derive(Debug, Clone)]
pub enum MacroToken<'a> {
    Parameter(usize),
    Token(Token<'a>)
}

pub struct MacroDefinition<'a> {
    pub name: Cow<'a, str>,
    pub params: Option<Vec<Cow<'a, str>>>,
    pub replacement: Vec<MacroToken<'a>>,
}

impl<'a> MacroDefinition<'a> {
}
