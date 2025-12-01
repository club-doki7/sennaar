use clang_sys::*;
use either::Either;

use crate::{
    Internalize, cpl::*, rossetta::{
        clang_ty::*, clang_utils::*
    }
};

/// Extract C declaration/definition of given [CXCursor] to [CDecl]
#[allow(non_upper_case_globals)]
pub fn map_decl(cursor: CXCursor, extra_decls: &mut Vec<CDecl>) -> Result<CDecl, ClangError> {
    unsafe {
        let kind = get_kind(cursor);
        // don't use get_cursor_display / clang_getCursorDisplayName, it includes some extra information.
        let name = get_cursor_spelling(cursor)?;

        let decl = match kind {
            CXCursor_VarDecl => {
                let ty = map_ty(clang_getCursorType(cursor))?;
                CDecl::Var(Box::new(CVarDecl { name: name.interned(), ty }))
            }

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
                // TODO: name of function pointer parameter is missing, consider enrich_param
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
                let mut subrecords = Vec::<RecordName>::new();

                if is_definition {
                    let children = get_children(cursor);
                    // all subrecords in struct/union definition will be delayed, they will be mapped after all fields are mapped.
                    let mut delayed_records = Vec::<CXCursor>::new();
                    children.into_iter()
                        .try_for_each(|e| {
                            // not all children are FieldDecl, unnamed StructDecl/UnionDecl can appear when it is the type of the field.
                            if e.kind() == CXCursor_FieldDecl {
                                map_field(e)
                                    .map(|it| fields.push(it))
                            } else {
                                delayed_records.push(e);
                                Ok(())    
                            }
                        })?;

                    delayed_records.into_iter()
                        .try_for_each(|e| {
                            // i guess we only accept struct/union decl in another struct/union decl
                            if e.kind() != CXCursor_StructDecl && e.kind() != CXCursor_UnionDecl {
                                return Err(format!(
                                    "Only StructDecl and UnionDecl can appear in another StructDecl or UnionDecl, but got: {}",
                                    e.get_kind_spelling()?));
                            }

                            let subdecl = map_decl(e, extra_decls)?;
                            if let Some(decl) = subdecl.get_record_decl()
                            // only unnamed nested record(struct/union) will introduce IndirectFieldDecl
                            && let Either::Right(usr) = &decl.name {
                                let used = fields.iter().any(|field| {
                                    any_usage(&field.ty, usr)
                                });

                                // don't add all nested unnamed record to subrecords, as it might be used by fields,
                                // thus doesn't introduce any IndirectFieldDecl
                                if ! used {
                                    subrecords.push(Either::Right(usr.clone()));
                                }
                            }

                            extra_decls.push(subdecl);

                            Ok(())
                        })?;
                }

                let decl = Box::new(CRecordDecl {
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
                let s = from_CXString(cs)
                    .map_err(|e| ClangError::from(format!("Failed to convert cursor kind spelling for kind {:?}: {}", kind, e)))?;
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
/// 
/// This can be also done by `map_ty` with a `Iterator<Item = CXCursor>`, but it is too fucking stupid.
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

            // TODO: if cursor is empty, then enrich_params is also empty,
            // we may assume cursor is either empty or the same size as params
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
        | CBaseType::Record(_, _)
        | CBaseType::Enum(_) 
        | CBaseType::Typedef(_) => ty,
    };

    Ok(result)
}

fn any_usage(ty: &CType, name: &String) -> bool {
    match &ty.ty {
        CBaseType::Record(_, Either::Right(usr)) => usr == name,
        CBaseType::Array(ty, _) => any_usage(ty, name),
        CBaseType::Pointer(ty) => any_usage(ty, name),
        CBaseType::FunProto(ret, params) => {
            any_usage(ret, name) 
            || params.iter().any(|p| any_usage(&p.ty, name))
        },
        
        CBaseType::Record(_, _)
        | CBaseType::Enum(_) 
        | CBaseType::Primitive(_) 
        | CBaseType::Typedef(_) => false,
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
        let ty = enrich_param(ty, &mut cursor.get_children().into_iter())?;

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
    let kind = get_kind(cursor);
    if kind != CXCursor_ParmDecl {
        let s = get_kind_spelling(kind)?;
        return Err(format!("Expected ParmDecl, but got: {}", s));
    }

    let name = get_cursor_spelling(cursor)?;
    let ty = map_cursor_ty(cursor)?;
    // TODO: maybe enrich_param

    Ok(CParam {
        name: Some(name.interned()),
        ty,
    })
}

#[cfg(test)]
mod tests {
    use either::Either;

    use crate::{Internalize, cpl::{CBaseType, CType}, rossetta::clang_decl::any_usage};

    #[test]
    fn test_any_usage() {
        let usr = "some usr".to_string();
        let in_ty = CType::konst(CBaseType::Pointer(Box::new(CType::konst(CBaseType::Record(true, Either::Right(usr.clone()))))));
        // note that the usage of [usr] is considered not in [not_in] even "some_typedef" is an alias of [usr]
        let not_in = CType::konst(CBaseType::Typedef("some_typedef".interned()));
        assert_eq!(any_usage(&in_ty, &usr), true);
        assert_eq!(any_usage(&not_in, &usr), false);
    }
}
