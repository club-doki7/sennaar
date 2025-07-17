use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::cpl::CExpr;
use crate::Identifier;


#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(JsonSchema)]
#[serde(tag = "$kind")]
pub enum Type<'a> {
    IdentifierType(Box<IdentifierType>),
    ArrayType(Box<ArrayType<'a>>),
    PointerType(Box<PointerType<'a>>)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(JsonSchema)]
pub struct IdentifierType {
    pub ident: Identifier
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(JsonSchema)]
pub struct ArrayType<'a> {
    pub element: Type<'a>,
    pub length: CExpr<'a>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PointerType<'a> {
    pub pointee: Type<'a>,
    pub is_const: bool,
    pub pointer_to_one: bool
}
