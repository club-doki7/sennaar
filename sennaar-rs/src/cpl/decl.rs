use std::fmt::Display;

use crate::{Identifier, cpl::{CParamLike, CType}};


pub enum CDecl {
    Typedef(Box<CTypedefDecl>),
    Fn(Box<CFnDecl>),
    Struct(Box<CStructDecl>),
    Enum(Box<CEnumDecl>),
}

pub struct CTypedefDecl {
    pub name: Identifier,
    pub underlying: CType,
}

pub struct CFnDecl {
    pub name: Identifier,
    pub ret: Box<CType>,
    pub parameters: Vec<CParamDecl>,
}

pub struct CParamDecl {
    pub name: Identifier,
    pub ty: CType,
}

pub struct CStructDecl {
    pub name: Identifier,
    pub fields: Vec<CFieldDecl>,
}

pub struct CFieldDecl {
    pub name: Identifier,
    pub ty: CType,
}

pub struct CEnumDecl {
    pub name: Identifier,
    pub ty: CType,
    pub members: Vec<CEnumConstantDecl>,
}

/// @param explicit whether the value of this decl is explicit
pub struct CEnumConstantDecl {
    pub name: Identifier,
    pub explicit: bool,
    pub value: u64,
}

impl Display for CParamDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.ty, self.name)
    }
}

impl CParamLike for CParamDecl {
    fn name(&self) -> Option<&Identifier> {
        Some(&self.name)
    }

    fn ty(&self) -> &CType {
        &self.ty
    }
}
