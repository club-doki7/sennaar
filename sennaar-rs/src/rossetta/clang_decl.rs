use std::fmt::Display;

use clang_sys::*;

use crate::{
    Internalize, cpl::*, rossetta::{
        clang_ty::*,
        clang_utils::*,
    }
};

#[allow(non_upper_case_globals)]
pub unsafe fn map_decl(cursor: CXCursor) -> Result<CDecl, ClangError> {
    unsafe {
        let kind = get_kind(cursor);
        // don't use get_cursor_display / clang_getCursorDisplayName, it includes some extra information.
        let name = get_cursor_spelling(cursor)?;

        let decl = match kind {
            CXCursor_TypedefDecl => {
                let underlying = clang_getTypedefDeclUnderlyingType(cursor);
                CDecl::Typedef(Box::new(CTypedefDecl {
                    name: name.interned(),
                    underlying: map_ty(underlying)?,
                }))
            }

            CXCursor_FunctionDecl => {
                // not sure if this differ to ParmDecl nodes
                let cty = map_cursor_ty(cursor)?;
                let parameters = get_children(cursor)
                    .into_iter()
                    .filter(|e| get_kind(*e) == CXCursor_ParmDecl)
                    .map(map_param)
                    .collect::<Result<Vec<CParamDecl>, ClangError>>()?;
                if let CType::FunProto(ret, _) = cty {
                    CDecl::Fn(Box::new(CFnDecl {
                        name: name.interned(),
                        ret,
                        parameters,
                    }))
                } else {
                    return Err(format!("Expected FunProto, but got: {}", cty));
                }
            }

            CXCursor_StructDecl => {
                let children = get_children(cursor);
                let fields = children
                    .into_iter()
                    .map(map_field)
                    .collect::<Result<Vec<CFieldDecl>, ClangError>>()?;

                CDecl::Struct(Box::new(CStructDecl {
                    name: name.interned(),
                    fields,
                }))
            }

            CXCursor_EnumDecl => {
                let children = get_children(cursor);
                let ty = map_cursor_ty(cursor)?;
                let members = children
                    .into_iter()
                    .map(map_enum_const)
                    .collect::<Result<Vec<CEnumConstantDecl>, ClangError>>()?;

                CDecl::Enum(Box::new(CEnumDecl {
                    name: name.interned(),
                    ty,
                    members: members,
                }))
            }

            _ => {
                let cs = clang_getCursorKindSpelling(kind);
                let s = from_CXString(cs).unwrap();
                todo!("unknown cursor kind on declaration {}: {}", name, s);
            }
        };

        Ok(decl)
    }
}

pub(crate) fn map_field(cursor: CXCursor) -> Result<CFieldDecl, ClangError> {
    unsafe {
        let kind = get_kind(cursor);
        if kind != CXCursor_FieldDecl {
            let spelling = clang_getCursorKindSpelling(kind);
            let s = from_CXString(spelling)?;
            return Err(format!("Expected FieldDecl, but got: {}", s));
        }

        let name = get_cursor_display(cursor)?;
        let ty = map_cursor_ty(cursor)?;

        Ok(CFieldDecl {
            name: name.interned(),
            ty,
        })
    }
}

pub(crate) fn map_enum_const(cursor: CXCursor) -> Result<CEnumConstantDecl, ClangError> {
    unsafe {
        let kind = get_kind(cursor);
        if kind != CXCursor_EnumConstantDecl {
            let s = get_kind_spelling(kind)?;
            return Err(format!("Expected EnumConstantDecl, but got: {}", s));
        }

        let name = get_cursor_spelling(cursor)?;
        let explicit = first_children(cursor).is_some();
        // TODO: what about signed?
        let value = clang_getEnumConstantDeclUnsignedValue(cursor);

        Ok(CEnumConstantDecl {
            name: name.interned(),
            explicit,
            value,
        })
    }
}

pub(crate) fn map_param(cursor: CXCursor) -> Result<CParamDecl, ClangError> {
    unsafe {
        let kind = get_kind(cursor);
        if kind != CXCursor_ParmDecl {
            let s = get_kind_spelling(kind)?;
            return Err(format!("Expected ParmDecl, but got: {}", s));
        }

        let name = get_cursor_spelling(cursor)?;
        let ty = map_cursor_ty(cursor)?;

        Ok(CParamDecl {
            name: name.interned(),
            ty,
        })
    }
}

impl Display for CDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CDecl::Typedef(decl) => {
                write!(f, "typedef {} {};", decl.underlying, decl.name)
            }
            CDecl::Fn(decl) => {
                CType::fmt_fun(f, &decl.ret, &decl.parameters, Some(&decl.name), false)?;

                write!(f, ";")
            }
            CDecl::Struct(decl) => {
                write!(f, "struct {} {{ ", decl.name)?;

                decl.fields
                    .iter()
                    .try_for_each(|field| write!(f, "{} {};", field.ty, field.name))?;

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
