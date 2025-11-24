use std::fmt::Display;

use crate::{Identifier, Internalize};


#[derive(Debug)]
pub enum CSign {
    Signed,
    ExplicitSigned,
    Unsigned,
}

#[derive(Debug)]
pub enum CType {
    Primitive { signed: CSign, ident: Identifier },
    Array(Box<CType>, u64),
    Pointer(Box<CType>),
    FunProto(Box<CType>, Vec<CType>),
    Struct(Identifier),
    Enum(Identifier),
    Typedef(Identifier),
}

// something that can be used as a parameter
pub trait CParamLike: Display {
    fn name(&self) -> Option<&Identifier>;
    fn ty(&self) -> &CType;
}

impl CParamLike for CType {
    fn name(&self) -> Option<&Identifier> {
        None
    }

    fn ty(&self) -> &CType {
        self
    }
}

impl CType {
    pub fn signed(ident: Identifier) -> CType {
        CType::Primitive {
            signed: CSign::Signed,
            ident,
        }
    }

    pub fn unsigned(ident: Identifier) -> CType {
        CType::Primitive {
            signed: CSign::Unsigned,
            ident,
        }
    }

    /// @param ptr only used when `name` is not [None]
    pub fn fmt_fun<P: CParamLike>(
        f: &mut std::fmt::Formatter<'_>,
        ret: &CType,
        params: &[P],
        name: Option<&Identifier>,
        is_ptr: bool,
    ) -> std::fmt::Result {
        write!(f, "{} ", ret)?;
        if let Some(name) = name {
            if is_ptr {
                write!(f, "(*{})", name)?;
            } else {
                write!(f, "{}", name)?;
            }
        }

        write!(f, "(")?;

        for (idx, p) in params.iter().enumerate() {
            if idx != 0 {
                write!(f, ", ")?;
            }

            p.fmt(f)?;
        }

        write!(f, ")")?;

        Ok(())
    }
}

impl Display for CType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            CType::Primitive { signed, ident } => match *signed {
                CSign::Signed => write!(f, "{}", ident),
                CSign::ExplicitSigned => write!(f, "signed {}", ident),
                CSign::Unsigned => write!(f, "unsigned {}", ident),
            },
            CType::Array(ctype, size) => write!(f, "{}[{}]", ctype, size),
            CType::Pointer(ctype) => match &*(*ctype) {
                CType::FunProto(ret, params) => {
                    CType::fmt_fun(f, ret, params, Some(&"".interned()), true)
                }
                _ => write!(f, "{}*", ctype),
            },
            CType::FunProto(ret, params) => CType::fmt_fun(f, ret, params, None, false),
            CType::Typedef(ident) => write!(f, "{}", ident),
            CType::Struct(ident) => write!(f, "struct {}", ident),
            CType::Enum(ident) => write!(f, "enum {}", ident),
        }
    }
}
