use std::fmt::Display;

use either::Either;

use crate::{
    Identifier,
    cpl::{CParamLike, CType},
};

#[derive(Debug)]
pub enum CDecl {
    Typedef(Box<CTypedefDecl>),
    Fn(Box<CFnDecl>),
    Struct(Box<CStructDecl>),
    Enum(Box<CEnumDecl>),
}

#[derive(Debug)]
pub struct CTypedefDecl {
    pub name: Identifier,
    pub underlying: CType,
}

#[derive(Debug)]
pub struct CFnDecl {
    pub name: Identifier,
    pub ret: Box<CType>,
    pub parameters: Vec<CParamDecl>,
}

#[derive(Debug)]
pub struct CParamDecl {
    pub name: Identifier,
    pub ty: CType,
}

#[derive(Debug)]
pub struct CStructDecl {
    /// either a named struct or a unnamed struct with its USR in [ClangCtx]
    pub name: Either<Identifier, String>,
    pub fields: Vec<CFieldDecl>,
}

#[derive(Debug)]
pub struct CFieldDecl {
    pub name: Identifier,
    pub ty: CType,
}

#[derive(Debug)]
pub struct CEnumDecl {
    pub name: Identifier,
    pub ty: CType,
    pub members: Vec<CEnumConstantDecl>,
}

/// @param explicit whether the value of this decl is explicit
#[derive(Debug)]
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

impl Display for CDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CDecl::Typedef(decl) => {
                write!(f, "typedef {} {};", decl.underlying, decl.name)
            }
            CDecl::Fn(decl) => {
                CType::fmt_fun(f, &decl.ret, &decl.parameters, Some(&decl.name), false, false)?;

                write!(f, ";")
            }
            CDecl::Struct(decl) => {
                write!(f, "struct ")?;

                match &decl.name {
                    Either::Left(name) => write!(f, "{}", name)?,
                    Either::Right(usr) => write!(f, "/* USR: {} */", usr)?,
                }

                write!(f, " {{")?;

                decl.fields
                    .iter()
                    .try_for_each(|field| write!(f, " {} {};", field.ty, field.name))?;

                write!(f, " }};")
            }
            CDecl::Enum(decl) => {
                write!(f, "enum {} {{ ", decl.name)?;

                for (idx, member) in decl.members.iter().enumerate() {
                    if idx != 0 {
                        write!(f, ", ")?;
                    }

                    if member.explicit {
                        write!(f, "{} = {}", member.name, member.value)?;
                    } else {
                        write!(f, "{}", member.name)?;
                    }
                }

                write!(f, " }};")
            }
        }
    }
}
