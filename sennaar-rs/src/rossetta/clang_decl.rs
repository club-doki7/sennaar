use clang_sys::*;
use either::Either;

use crate::{
    Internalize, cpl::*, registry, rossetta::{
        clang_ctx::ClangCtx, clang_ty::*, clang_utils::*
    }
};

#[allow(non_upper_case_globals)]
pub unsafe fn map_decl(cursor: CXCursor, ctx: &mut ClangCtx) -> Result<CDecl, ClangError> {
    unsafe {
        let kind = get_kind(cursor);
        // don't use get_cursor_display / clang_getCursorDisplayName, it includes some extra information.
        let name = get_cursor_spelling(cursor)?;

        let decl = match kind {
            CXCursor_TypedefDecl => {
                let underlying  = map_ty(clang_getTypedefDeclUnderlyingType(cursor), ctx)?;

                CDecl::Typedef(Box::new(CTypedefDecl {
                    name: name.interned(),
                    underlying,
                }))
            }

            CXCursor_FunctionDecl => {
                // not sure if this differ to ParmDecl nodes
                let cty = map_cursor_ty(cursor, ctx)?;
                let parameters = get_children(cursor)
                    .into_iter()
                    .filter(|e| get_kind(*e) == CXCursor_ParmDecl)
                    .map(|e| map_param(e, ctx))
                    .collect::<Result<Vec<CParamDecl>, ClangError>>()?;
                if let CBaseType::FunProto(ret, _) = cty.ty {
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
                let has_name = ! cursor.is_anonymous();
                let name = if has_name {
                    Either::Left(name.interned())
                } else {
                    let usr = cursor.get_usr()?;
                    println!("FUCK ME");

                    Either::Right(usr)
                };

                let children = get_children(cursor);
                let fields = children
                    .into_iter()
                    .map(|e| map_field(e, ctx))
                    .collect::<Result<Vec<CFieldDecl>, ClangError>>()?;

                CDecl::Struct(Box::new(CStructDecl {
                    name, fields,
                }))
            }

            CXCursor_EnumDecl => {
                let children = get_children(cursor);
                let ty = map_cursor_ty(cursor, ctx)?;
                let members = children
                    .into_iter()
                    .map(|e| map_enum_const(e, ctx))
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

pub(crate) fn map_field(cursor: CXCursor, ctx: &mut ClangCtx) -> Result<CFieldDecl, ClangError> {
    unsafe {
        let kind = get_kind(cursor);
        if kind != CXCursor_FieldDecl {
            let spelling = clang_getCursorKindSpelling(kind);
            let s = from_CXString(spelling)?;
            return Err(format!("Expected FieldDecl, but got: {}", s));
        }

        let name = get_cursor_display(cursor)?;
        let ty = map_cursor_ty(cursor, ctx)?;

        Ok(CFieldDecl {
            name: name.interned(),
            ty,
        })
    }
}

pub(crate) fn map_enum_const(cursor: CXCursor, ctx: &mut ClangCtx) -> Result<CEnumConstantDecl, ClangError> {
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

pub(crate) fn map_param(cursor: CXCursor, ctx: &mut ClangCtx) -> Result<CParamDecl, ClangError> {
    unsafe {
        let kind = get_kind(cursor);
        if kind != CXCursor_ParmDecl {
            let s = get_kind_spelling(kind)?;
            return Err(format!("Expected ParmDecl, but got: {}", s));
        }

        let name = get_cursor_spelling(cursor)?;
        let ty = map_cursor_ty(cursor, ctx)?;

        Ok(CParamDecl {
            name: name.interned(),
            ty,
        })
    }
}

pub fn entitilize_decl<'de>(registry: &mut registry::RegistryBase, decl: &CDecl) -> Option<()> {
    match &decl {
        CDecl::Typedef(typedef) => {
            // match &typedef.underlying.ty {
            //     CBaseType::Pointer(ptr) => {
            //         match &ptr.ty {
            //             CBaseType::FunProto(ret, params) => {
            //                 // typedef void (*foo)(...)
            //                 let def = registry::FunctionTypedef::new(
            //                     typedef.name.clone(), 
            //                     todo!(), 
            //                     to_cpl_type(&ret)?,
            //                     true, 
            //                     false       // TODO: i don't know
            //                 );

            //                 registry.function_typedefs.insert(def.name.clone(), def);
            //                 return Some(())
            //             }

            //             CBaseType::Struct(ident) => {
            //                 todo!();
            //                 // typedef struct _Foo * Foo
            //                 // this is a opaque handle typedef
            //                 return Some(())
            //             }

            //             _ => {}
            //         }
            //     }

            //     CBaseType::Struct(ident) if ident == &typedef.name => {
            //         // typedef struct Foo Foo;
            //     }

            //     _ => {
            //         let def = registry::Typedef::new(
            //             typedef.name.clone(), to_cpl_type(&typedef.underlying)?
            //         );
            //     }
            // }
            todo!()
        }
        CDecl::Fn(decl) => {
            registry::Command::new(
                decl.name.clone(), todo!(), todo!(), todo!(), todo!(), todo!()
            );
        },
        CDecl::Struct(cstruct_decl) => todo!(),
        CDecl::Enum(cenum_decl) => todo!(),
    }

    Some(())
}
