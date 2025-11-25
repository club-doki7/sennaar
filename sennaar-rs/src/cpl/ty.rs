use std::fmt::Display;

use crate::{Identifier, Internalize};

#[derive(Debug)]
pub enum CSign {
    Signed,
    ExplicitSigned,
    Unsigned,
}

#[derive(Debug)]
pub struct CType {
    pub is_const: bool,
    pub ty: CBaseType,
}

#[derive(Debug)]
pub enum CBaseType {
    Primitive { signed: CSign, ident: Identifier },
    Array(Box<CType>, u64),
    Pointer(Box<CType>),
    FunProto(Box<CType>, Vec<CType>),
    Struct(Identifier),
    Enum(Identifier),
    Typedef(Identifier),
}

// macro_rules! define_ty {
//     ($name:ident($($field:ident: $type:ty),*)) => {
//         #[derive(Debug)]
//         pub struct $name {
//             pub is_const: bool,
//             $(pub $field: $type),*
//         }
//     };
// }

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
    pub fn konst(ty: CBaseType) -> CType {
        CType {
            is_const: true,
            ty
        }
    }

    pub fn unkonst(ty: CBaseType) -> CType {
        CType {
            is_const: false,
            ty
        }
    }

    pub fn signed(ident: Identifier) -> CType {
        CType::unkonst(CBaseType::Primitive {
            signed: CSign::Signed,
            ident,
        })
    }

    pub fn unsigned(ident: Identifier) -> CType {
        CType::unkonst(CBaseType::Primitive {
            signed: CSign::Unsigned,
            ident,
        })
    }

    /// @param ptr only used when `name` is not [None]
    /// @param is_const only used when `is_ptr` is true
    pub fn fmt_fun<P: CParamLike>(
        f: &mut std::fmt::Formatter<'_>,
        ret: &CType,
        params: &[P],
        name: Option<&Identifier>,
        is_ptr: bool,
        is_const: bool,
    ) -> std::fmt::Result {
        write!(f, "{} ", ret)?;
        if let Some(name) = name {
            if is_ptr {
                write!(f, "(*")?;
                if is_const {
                    write!(f, "const ")?;
                }
                write!(f, "{})", name)?;
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

    fn normal_const(&self) -> bool {
        match &self.ty {
            CBaseType::Array(_, _) => false,
            CBaseType::Pointer(_) => false,
            CBaseType::FunProto(_, _) => false,
            _ => true
        }
    }
}

impl Display for CType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_const && self.normal_const() {
            write!(f, "const ")?;
        }

        match &self.ty {
            CBaseType::Primitive { signed, ident } => match *signed {
                CSign::Signed => write!(f, "{}", ident),
                CSign::ExplicitSigned => write!(f, "signed {}", ident),
                CSign::Unsigned => write!(f, "unsigned {}", ident),
            },
            CBaseType::Array(ctype, size) => write!(f, "{}[{}]", ctype, size),
            CBaseType::Pointer(inner) => match &inner.ty {
                CBaseType::FunProto(ret, params) => {
                    CType::fmt_fun(f, ret, params, Some(&"".interned()), true, self.is_const)
                }
                _ => {
                    write!(f, "{} *", inner)?;

                    if self.is_const {
                        write!(f, "const")?;
                    }

                    Ok(())
                },
            },
            // i guess function proto is never const
            CBaseType::FunProto(ret, params) => CType::fmt_fun(f, ret, params, None, false, false),
            CBaseType::Typedef(ident) => write!(f, "{}", ident),
            CBaseType::Struct(ident) => write!(f, "struct {}", ident),
            CBaseType::Enum(ident) => write!(f, "enum {}", ident),
        }
    }
}
