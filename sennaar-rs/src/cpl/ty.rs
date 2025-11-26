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
pub enum CPrimitive {
    Void, Bool, UChar, CharS, SChar, UShort, Short, UInt, Int, ULong, Long, ULongLong, LongLong, Float, Double, LongDouble
}

impl CPrimitive {
    fn sign(&self) -> Option<CSign> {
        match &self {
            CPrimitive::UChar | CPrimitive::UShort | CPrimitive::UInt | CPrimitive::ULong | CPrimitive::ULongLong => Some(CSign::Unsigned),
            CPrimitive::SChar => Some(CSign::ExplicitSigned),
            CPrimitive::Void | CPrimitive::Bool | CPrimitive::Float | CPrimitive::Double | CPrimitive::LongDouble => None,
            _ => Some(CSign::Signed),
        }
    }

    /// return the type name of this primitive type, without any signed/unsigned qualifier
    fn type_name(&self) -> &'static str {
        match &self {
            CPrimitive::Void => "void",
            CPrimitive::Bool => "bool",
            CPrimitive::UChar | CPrimitive::CharS | CPrimitive::SChar => "char",
            CPrimitive::UShort | CPrimitive::Short => "short",
            CPrimitive::UInt | CPrimitive::Int => "int",
            CPrimitive::ULong | CPrimitive::Long => "long",
            CPrimitive::ULongLong | CPrimitive::LongLong => "long long",
            CPrimitive::Float => "float",
            CPrimitive::Double => "double",
            CPrimitive::LongDouble => "long double",
        }
    }
}

impl Display for CPrimitive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sign = self.sign();
        if let Some(sign) = sign {
            match sign {
                CSign::Signed => {},
                CSign::ExplicitSigned => write!(f, "signed ")?,
                CSign::Unsigned => write!(f, "unsigned ")?,
            }
        }

        write!(f, "{}", self.type_name())
    }
}

#[derive(Debug)]
pub struct CParam {
    pub name: Option<Identifier>,
    pub ty: CType,
}

#[derive(Debug)]
pub enum CBaseType {
    Primitive(CPrimitive),
    Array(Box<CType>, Option<u64>),
    Pointer(Box<CType>),
    FunProto(Box<CType>, Vec<CParam>),
    Struct(Identifier),
    /// a USR in [ClangCtx]
    UnnamedStruct(String),
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

impl CBaseType {
    pub fn is_pointer(&self) -> bool {
        if let CBaseType::Pointer(_) = &self {
            true
        } else {
            false
        }
    }

    
    pub fn normal_const(&self) -> bool {
        match &self {
            CBaseType::Array(_, _) => false,
            CBaseType::Pointer(_) => false,
            CBaseType::FunProto(_, _) => false,
            _ => true
        }
    }

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, is_const: bool) -> std::fmt::Result {
        if is_const && self.normal_const() {
            write!(f, "const ")?;
        }

        match &self {
            CBaseType::Primitive(primitive) => primitive.fmt(f),
            CBaseType::Array(ctype, size) => {
                write!(f, "{}[", ctype)?;

                if let Some(size) = size {
                    write!(f, "{}", size)?;
                } else if is_const {
                    // i guess `else if` is okay..
                    write!(f, "const")?;
                }

                write!(f, "]")
            },
            CBaseType::Pointer(inner) => match &inner.ty {
                CBaseType::FunProto(ret, params) => {
                    CType::fmt_fun(f, ret, params, Some(&"".interned()), true, is_const)
                }
                _ => {
                    if inner.ty.is_pointer() {
                        write!(f, "{}*", inner)?;
                    } else {
                        write!(f, "{} *", inner)?;
                    }

                    if is_const {
                        write!(f, "const")?;
                    }

                    Ok(())
                },
            },
            // i guess function proto is never const
            CBaseType::FunProto(ret, params) => CType::fmt_fun(f, ret, params, None, false, false),
            CBaseType::Typedef(ident) => write!(f, "{}", ident),
            CBaseType::Struct(ident) => write!(f, "struct {}", ident),
            CBaseType::UnnamedStruct(usr) => write!(f, "struct <USR: {}>", usr),
            CBaseType::Enum(ident) => write!(f, "enum {}", ident),
        }
    }
}

impl Display for CBaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        CBaseType::fmt(&self, f, false)
    }
}

impl Display for CParam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ty)?;

        if let Some(name) = &self.name {
            if self.ty.ty.is_pointer() {
                write!(f, "{}", name)?;
            } else {
                write!(f, " {}", name)?;
            }
        }

        Ok(())
    }
}

impl CParamLike for CParam {
    fn name(&self) -> Option<&Identifier> {
        self.name.as_ref()
    }

    fn ty(&self) -> &CType {
        &self.ty
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
        CBaseType::fmt(&self.ty, f, self.is_const)
    }
}
