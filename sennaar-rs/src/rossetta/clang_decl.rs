use clang_sys::*;
use either::Either;

use crate::{
    Internalize, cpl::*, rossetta::{
        clang_ty::*, clang_utils::*
    }
};

#[allow(non_upper_case_globals)]
pub fn map_decl(cursor: CXCursor, extra_decls: &mut Vec<CDecl>) -> Result<CDecl, ClangError> {
    unsafe {
        let kind = get_kind(cursor);
        // don't use get_cursor_display / clang_getCursorDisplayName, it includes some extra information.
        let name = get_cursor_spelling(cursor)?;

        let decl = match kind {
            CXCursor_TypedefDecl => {
                let underlying  = map_ty(clang_getTypedefDeclUnderlyingType(cursor))?;
                // a more efficent way is [map_ty] with a cursor, but that makes thing more complicate.
                let enrich_underlying = enrich_param(underlying, &mut cursor.get_children().into_iter())?;

                CDecl::Typedef(Box::new(CTypedefDecl {
                    name: name.interned(),
                    underlying: enrich_underlying,
                }))
            }

            CXCursor_FunctionDecl => {
                // not sure if this differ to ParmDecl nodes
                let cty = map_cursor_ty(cursor)?;
                let parameters = get_children(cursor)
                    .into_iter()
                    .filter(|e| get_kind(*e) == CXCursor_ParmDecl)
                    .map(|e| map_param(e))
                    .collect::<Result<Vec<CParam>, ClangError>>()?;
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

            CXCursor_StructDecl | CXCursor_UnionDecl => {
                let has_name = ! cursor.is_anonymous();
                let name = if has_name {
                    Either::Left(name.interned())
                } else {
                    Either::Right(cursor.get_usr()?)
                };

                let is_definition = cursor.is_definition();

                let mut fields = Vec::<CFieldDecl>::new();
                let mut subrecords = Vec::<String>::new();

                if is_definition {
                    let children = get_children(cursor);
                    // TODO: not all children are FieldDecl, unnamed StructDecl/UnionDecl can appear when it is the type of the field.
                    children.into_iter()
                        .try_for_each(|e| {
                            if e.kind() == CXCursor_FieldDecl {
                                map_field(e)
                                    .map(|it| fields.push(it))
                            } else {
                                map_decl(e, extra_decls)
                                    .map(|it| {
                                        if let Some(decl) = it.get_record_decl()
                                        && let Either::Right(usr) = &decl.name {
                                            // only unnamed nested record(struct/union) will introduce IndirectFieldDecl
                                            subrecords.push(usr.clone());
                                        }
                                        
                                        extra_decls.push(it)
                                    })
                            }
                        })?;
                }

                let decl = Box::new(CStructDecl {
                    name, fields, subrecords, is_definition
                });

                match kind {
                    CXCursor_StructDecl => CDecl::Struct(decl),
                    CXCursor_UnionDecl => CDecl::Union(decl),
                    _ => unreachable!()
                }
            }

            CXCursor_EnumDecl => {
                let children = get_children(cursor);
                let ty = map_cursor_ty(cursor)?;
                let members = children
                    .into_iter()
                    .map(|e| map_enum_const(e))
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

/// Enrich [CBaseType::FunProto] with children of [cursor].
/// Consider `typedef void (*(*fp_that_accept_fp_and_return_fp)(void (*f)(int f_input)))(int ret_input)`
/// or in Rust `fn(f: fn(f_input: u32) -> ()) -> (fn(ret_input: u32) -> ())`   // ignore some size problem
/// The ParmDecl in its children looks like:
/// ```plain
/// ParmDecl(name: ret_input, type: int)
/// ParmDecl(name: f, type: void (*)(int f_input))
///   ParmDecl(name: f_input, type: int)        // child of `f`, not the typedef
/// ```
pub(crate) fn enrich_param(ty: CType, cursor: &mut impl Iterator<Item = CXCursor>) -> Result<CType, ClangError> {
    let result = match ty.ty {
        CBaseType::Array(elem_ty, len) => {
            CType {
                is_const: ty.is_const,
                ty: CBaseType::Array(Box::new(enrich_param(*elem_ty, cursor)?), len)
            }
        },
        CBaseType::Pointer(inner_ty) => CType {
            is_const: ty.is_const,
            ty: CBaseType::Pointer(Box::new(enrich_param(*inner_ty, cursor)?))
        },
        CBaseType::FunProto(ret_ty, params) => {
            let enrich_ret = enrich_param(*ret_ty, cursor)?;
            let enrich_params = params.into_iter()
                .zip(cursor)
                .map::<Result<CParam, ClangError>, _>(|(param, cursor)| {
                    if cursor.kind() == CXCursor_ParmDecl {
                        let name = cursor.get_spelling()?;
                        if ! name.is_empty() {
                            let enrich_ty = enrich_param(param.ty, &mut cursor.get_children().into_iter())?;
                            return Ok(CParam {
                                name: Some(name.interned()),
                                ty: enrich_ty
                            });
                        }
                        
                        // fall though
                    }

                    Ok(param)
                })
                .collect::<Result<Vec<CParam>, ClangError>>()?;

            CType {
                is_const: ty.is_const,
                ty: CBaseType::FunProto(Box::new(enrich_ret), enrich_params)
            }
        },

        // leaf types
        CBaseType::Primitive(_) 
        | CBaseType::UnnamedStruct(_) 
        | CBaseType::Struct(_) 
        | CBaseType::Enum(_) 
        | CBaseType::Typedef(_) => ty,
    };

    Ok(result)
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

pub(crate) fn map_param(cursor: CXCursor) -> Result<CParam, ClangError> {
    unsafe {
        let kind = get_kind(cursor);
        if kind != CXCursor_ParmDecl {
            let s = get_kind_spelling(kind)?;
            return Err(format!("Expected ParmDecl, but got: {}", s));
        }

        let name = get_cursor_spelling(cursor)?;
        let ty = map_cursor_ty(cursor)?;

        Ok(CParam {
            name: Some(name.interned()),
            ty,
        })
    }
}
