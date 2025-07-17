use std::borrow::Cow;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::Identifier;


include!("../macross.rs");

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(JsonSchema)]
#[serde(tag = "$kind")]
pub enum CExpr<'a> {
    IntLiteral(Box<CIntLiteralExpr<'a>>),
    FloatLiteral(Box<CFloatLiteralExpr<'a>>),
    CharLiteral(Box<CCharLiteralExpr<'a>>),
    StringLiteral(Box<CStringLiteralExpr<'a>>),
    Identifier(Box<CIdentifierExpr>),
    Index(Box<CIndexExpr<'a>>),
    Call(Box<CCallExpr<'a>>),
    Member(Box<CMemberExpr<'a>>),
    PtrMember(Box<CPtrMemberExpr<'a>>),
    PostfixIncDec(Box<CPostfixIncDecExpr<'a>>),
    Unary(Box<CUnaryExpr<'a>>),
    Cast(Box<CCastExpr<'a>>),
    Binary(Box<CBinaryExpr<'a>>),
    Conditional(Box<CConditionalExpr<'a>>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(JsonSchema)]
pub struct CIntLiteralExpr<'a> {
    pub value: Cow<'a, str>,
    pub suffix: Cow<'a, str>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(JsonSchema)]
pub struct CFloatLiteralExpr<'a> {
    pub value: Cow<'a, str>,
    pub suffix: Cow<'a, str>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(JsonSchema)]
pub struct CCharLiteralExpr<'a> {
    pub value: Cow<'a, str>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(JsonSchema)]
pub struct CStringLiteralExpr<'a> {
    pub value: Cow<'a, str>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(JsonSchema)]
pub struct CIdentifierExpr {
    pub ident: Identifier,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(JsonSchema)]
pub struct CIndexExpr<'a> {
    pub base: CExpr<'a>,
    pub index: CExpr<'a>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(JsonSchema)]
pub struct CCallExpr<'a> {
    pub callee: CExpr<'a>,
    pub args: Vec<CExpr<'a>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(JsonSchema)]
pub struct CMemberExpr<'a> {
    pub obj: CExpr<'a>,
    pub member: Identifier,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(JsonSchema)]
pub struct CPtrMemberExpr<'a> {
    pub obj: CExpr<'a>,
    pub member: Identifier,
}

ss_enum!{CPostfixIncDecOp, Inc, Dec}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(JsonSchema)]
pub struct CPostfixIncDecExpr<'a> {
    pub expr: CExpr<'a>,
    pub op: CPostfixIncDecOp,
}

ss_enum!{
    CUnaryOp,
    Plus, Minus, Not, BitNot, Deref, AddrOf, SizeOf, AlignOf, Inc, Dec
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(JsonSchema)]
pub struct CUnaryExpr<'a> {
    pub expr: CExpr<'a>,
    pub op: CUnaryOp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(JsonSchema)]
pub struct CCastExpr<'a> {
    pub expr: CExpr<'a>,
    pub ty: CExpr<'a>,
}

ss_enum!{
    CBinaryOp,
    Mul, Div, Mod,
    Add, Sub,
    Shl, Shr,
    Less, Greater, LessEq, GreaterEq,
    Eq, NotEq,
    BitAnd, BitXor, BitOr,
    And, Or, Xor,
    Assign, MulAssign, DivAssign, ModAssign,
    AddAssign, SubAssign,
    ShlAssign, ShrAssign,
    BitAndAssign, BitXorAssign, BitOrAssign,
    AndAssign, OrAssign, XorAssign,
    Comma
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(JsonSchema)]
pub struct CBinaryExpr<'a> {
    pub op: CBinaryOp,
    pub lhs: CExpr<'a>,
    pub rhs: CExpr<'a>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(JsonSchema)]
pub struct CConditionalExpr<'a> {
    pub cond: CExpr<'a>,
    pub then: CExpr<'a>,
    pub otherwise: CExpr<'a>
}
