use std::{borrow::Cow, fmt::Display};

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
    Paren(Box<CParenExpr<'a>>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(JsonSchema)]
pub struct CIntLiteralExpr<'a> {
    pub value: Cow<'a, str>,
    pub suffix: Cow<'a, str>,
}

impl<'a> CIntLiteralExpr<'a> {
    pub fn new(value: Cow<'a, str>) -> Self {
        Self {
            value,
            suffix: Cow::Borrowed(""),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(JsonSchema)]
pub struct CFloatLiteralExpr<'a> {
    pub value: Cow<'a, str>,
    pub suffix: Cow<'a, str>,
}

impl<'a> CFloatLiteralExpr<'a> {
    pub fn new(value: Cow<'a, str>) -> Self {
        Self {
            value,
            suffix: Cow::Borrowed(""),
        }
    }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(JsonSchema)]
pub struct CParenExpr<'a> {
    pub expr: CExpr<'a>,
}

impl <'a> Display for CExpr<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            CExpr::IntLiteral(i) => write!(f, "{}{}", i.value, i.suffix),
            CExpr::FloatLiteral(d) => write!(f, "{}{}", d.value, d.suffix),
            CExpr::CharLiteral(c) => write!(f, "'{}'", c.value),
            CExpr::StringLiteral(s) => write!(f, "\"{}\"", s.value),
            CExpr::Identifier(i) => write!(f, "{}", i.ident),
            CExpr::Index(arr_sub) => write!(f, "{}[{}]", arr_sub.base, arr_sub.index),
            CExpr::Call(call) => {
                write!(f, "{}(", call.callee)?;

                for i in 0..call.args.len() {
                    write!(f, "{}", call.args[i])?;
                    if i != call.args.len() - 1 {
                        write!(f, ",")?;
                    }
                }

                write!(f, ")")?;

                Ok(())
            },
            CExpr::Member(mem) => write!(f, "{}.{}", mem.obj, mem.member),
            CExpr::PtrMember(mem) => write!(f, "(*{}).{}", mem.obj, mem.member),
            CExpr::PostfixIncDec(e) => write!(f, "{}{}", e.expr, post_op_describe(e.op)),
            CExpr::Unary(e) => match e.op {
                CUnaryOp::AddrOf => write!(f, "sizeof({})", e.expr),
                CUnaryOp::AlignOf => write!(f, "??({})", e.expr),
                _ => write!(f, "{}{}", pre_op_describe(e.op), e.expr),
            },
            CExpr::Cast(_c) => todo!(),
            CExpr::Binary(b) => write!(f, "{} {} {}", b.lhs, bin_op_describe(b.op), b.rhs),
            CExpr::Conditional(c) => write!(f, "{} ? {} : {}", c.cond, c.then, c.otherwise),
            CExpr::Paren(p) => write!(f, "({})", p.expr),
        }
    }
}

fn bin_op_describe(op: CBinaryOp) -> &'static str {
    match op {
        CBinaryOp::Mul => "*",
        CBinaryOp::Div => "/",
        CBinaryOp::Mod => "%",
        CBinaryOp::Add => "+",
        CBinaryOp::Sub => "-",
        CBinaryOp::Shl => "<<",
        CBinaryOp::Shr => ">>",
        CBinaryOp::Less => "<",
        CBinaryOp::Greater => ">",
        CBinaryOp::LessEq => "<=",
        CBinaryOp::GreaterEq => ">=",
        CBinaryOp::Eq => "==",
        CBinaryOp::NotEq => "!=",
        CBinaryOp::BitAnd => "&",
        CBinaryOp::BitXor => "^",
        CBinaryOp::BitOr => "|",
        CBinaryOp::And => "&&",
        CBinaryOp::Or => "||",
        CBinaryOp::Xor => "^",
        CBinaryOp::Assign => "=",
        CBinaryOp::MulAssign => "*=",
        CBinaryOp::DivAssign => "/=",
        CBinaryOp::ModAssign => "%=",
        CBinaryOp::AddAssign => "+=",
        CBinaryOp::SubAssign => "-=",
        CBinaryOp::ShlAssign => "<<=",
        CBinaryOp::ShrAssign => ">>=",
        CBinaryOp::BitAndAssign => "&=",
        CBinaryOp::BitXorAssign => "^=",
        CBinaryOp::BitOrAssign => "|=",
        CBinaryOp::AndAssign => "&&=",      // ???
        CBinaryOp::OrAssign => "||=",
        CBinaryOp::XorAssign => "^=",
        CBinaryOp::Comma => ",",
    }
}

fn pre_op_describe(op: CUnaryOp) -> &'static str {
    match op {
        CUnaryOp::Plus => "+",
        CUnaryOp::Minus => "-",
        CUnaryOp::Not => "!",
        CUnaryOp::BitNot => "~",
        CUnaryOp::Deref => "*",
        CUnaryOp::AddrOf => "&",
        CUnaryOp::Inc => "++",
        CUnaryOp::Dec => "--",
        CUnaryOp::SizeOf | CUnaryOp::AlignOf => unreachable!(),
    }
}

fn post_op_describe(op: CPostfixIncDecOp) -> &'static str {
    match op {
        CPostfixIncDecOp::Inc => "++",
        CPostfixIncDecOp::Dec => "--",
    }
}