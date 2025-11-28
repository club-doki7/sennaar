use std::{borrow::Cow, collections::HashMap};

use crate::panspace::tok::Token;

#[derive(Debug, Clone)]
pub enum MacroToken<'a> {
    Parameter(usize),
    Token(Token<'a>),
}

pub struct MacroDefinition<'a> {
    pub name: Cow<'a, str>,
    pub params: Option<Vec<Cow<'a, str>>>,
    pub replacement: Vec<Token<'a>>,
}

impl<'a> MacroDefinition<'a> {
    pub fn expand_rec(
        &self,
        args: Option<&[Vec<Token<'a>>]>,
        registry: &HashMap<&str, MacroDefinition<'a>>
    ) {
    }
}
