use std::collections::HashMap;

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

/// region Naming Unnamed Struct

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

/// @param decls a sequence of decl that lives in the same level, it could be either a decl sequence at top-level or in some struct
pub fn name_unnamed_structs(decls: Vec<CDecl>) -> Vec<CDecl> {
    let mut usage_map: HashMap<String, Vec<Usage>> = HashMap::new();
    let mut new_decls = Vec::<CDecl>::new();

    decls.iter().for_each(|decl| {
        match decl {
            CDecl::Var(var) => {
                collect_usage_on_ty(&var.ty, &mut usage_map, vec![ ContextPathNode::Var(var.name.to_string()) ]);
            }
            CDecl::Struct(record) | CDecl::Union(record) => {
                if record.is_definition {
                    let name = decl.name();
                    let node = match decl {
                        CDecl::Struct(_) => ContextPathNode::Struct(name),
                        CDecl::Union(_) => ContextPathNode::Union(name),
                        _ => unreachable!(),
                    };

                    // collect all usage by field, such as `struct { struct { ... } f, *p; }`
                    record.fields.iter().for_each(|field| {
                        collect_usage_on_ty(
                            &field.ty, 
                            &mut usage_map, 
                            vec![ 
                                node.clone(), 
                                ContextPathNode::Field(field.name.to_string())
                            ]
                        );
                    });

                    // collect all usage by struct (i mean `struct { struct { ... }; }`)
                    // these struct will not be referenced by any field, thus we can put them to usage_map at any time
                    record.subrecords.iter().enumerate().for_each(|(idx, name)| {
                        if let Either::Right(usr) = name {
                            // println!("subdecl {}: {}", idx, usr);
                            usage_map.insert(usr.clone(), vec![ Usage(vec![ node.clone(), ContextPathNode::Nest(idx) ]) ]);
                        } else {
                            unreachable!("Subrecord in struct/union must be unnamed before name_unnamed_structs")
                        }
                    });
                }
            },
            
            CDecl::Typedef(_) => {},        // TODO: handle typedef struct { ... } *p;` case
            CDecl::Fn(_) 
            | CDecl::Enum(_) => {},
        }
    });

    let mut cache: HashMap<String, String> = HashMap::new();

    println!("{:#?}", usage_map);
    
    decls.into_iter().for_each(|decl| {
        match decl {
            CDecl::Struct(ref record) | CDecl::Union(ref record) => {
                if record.is_definition {

                    let name = match &record.name {
                        Either::Left(ident) => ident.clone(),
                        Either::Right(usr) => {
                            let is_struct = match decl {
                                CDecl::Struct(_) => true,
                                CDecl::Union(_) => false,
                                _ => unreachable!()
                            };

                            Usage::to_name(&usage_map, usr, is_struct, &mut cache).interned()
                        },
                    };

                    let decl_op = |is_struct: bool, decl: CRecordDecl| {
                        let named_subrecords = decl.subrecords.iter()
                            .map(|usr| {
                                let usr = usr.as_ref()
                                    .expect_right("Subrecord in struct/union must be unnamed before name_unnamed_structs");
                                Either::Left(Usage::to_name(&usage_map, usr, is_struct, &mut cache).interned())
                            })
                            .collect();

                        let inst_fields = decl.fields.into_iter().map(|decl| {
                            let new_ty = Usage::replace_usage_in_ty(&usage_map, decl.ty, &mut cache);
                            CFieldDecl {
                                name: decl.name,
                                ty: new_ty,
                            }
                        }).collect::<Vec<CFieldDecl>>();
                    
                        CRecordDecl {
                            name: Either::Left(name),
                            fields: inst_fields,
                            is_definition: decl.is_definition,
                            subrecords: named_subrecords,
                        }
                    };

                    let decl = match decl {
                        CDecl::Struct(record) => CDecl::Struct(Box::new(decl_op(true, *record))),
                        CDecl::Union(record) => CDecl::Union(Box::new(decl_op(false, *record))),
                        _ => unreachable!(),
                    };

                    new_decls.push(decl);
                    return;
                }
            },
            CDecl::Var(decl) => {
                let new_ty = Usage::replace_usage_in_ty(&usage_map, decl.ty, &mut cache);
                let new_decl = CVarDecl {
                    name: decl.name,
                    ty: new_ty,
                };

                new_decls.push(CDecl::Var(Box::new(new_decl)));
                return;
            },

            CDecl::Typedef(_) => {},        // TODO: `handle typedef struct { ... } *p;` case

            _ => {}
        }

        new_decls.push(decl);
    });

    new_decls
}

#[derive(Clone, Copy, Debug)]
pub enum ContextPathKind {
    // record path node kind
    Struct, Union, 
    // type path node kind
    FunRet, Ptr, Array,
    // name owner kind
    Field, FunParam, 
}

#[derive(Clone, Debug)]
pub enum ContextPathNode {
    // `struct A { struct { ... } here; };` give you `Struct(A)`
    Struct(RecordName),
    Union(RecordName),
    // `void (*f)(struct { ... } here)` give you `vec![Ptr, FunRet]`
    FunRet, Ptr, Array,
    // usage kind, for example,
    // the context of `struct { ... }` in `struct A { struct { ... } *foo; }` is `vec![ Struct(A), Field(foo), Ptr ]`
    // and the context of `struct A { void (*f)(struct { ... } bar); }` is `vec![ Struct(A), Field(f), Ptr, FunParam(bar) ]`
    Field(String), FunParam(String), Var(String),
    // unfortunately, we don't know the kind of the nest structure (struct or union),
    // cause `CStructDecl.subrecords` only stores USR.
    Nest(usize),
}

#[derive(Debug)]
struct Usage(Vec<ContextPathNode>);

impl Usage {
    fn to_name(usages: &HashMap<String, Vec<Usage>>, usr: &String, is_struct: bool, cache: &mut HashMap<String, String>) -> String {
        let name = Usage::_to_name(usages, usr, cache);
        let prefix = if is_struct {
            "struct"
        } else {
            "union"
        };

        format!("{}_de_{}", prefix, name)
    }

    /// Convert a USR to human readable name with [usages]
    /// @param usages all usage to unnamed structures' usr
    fn _to_name(usages: &HashMap<String, Vec<Usage>>, usr: &String, cache: &mut HashMap<String, String>) -> String {
        if let Some(hit) = cache.get(usr) {
            return hit.clone();
        }

        let usage = usages.get(usr)
            .expect(&format!("Usage of {} not found, this could be either a bug in `map_decl` or the map haven't been initialized.", usr));

        // TODO: generate name with ALL usage of usr
        let result = usage.first()
            .unwrap_or_else(|| panic!("Usage of {} contains empty context path, are you serious??", usr))
            .0.iter().map(|node| {
            match node {
                ContextPathNode::Struct(name) | ContextPathNode::Union(name) => {
                    let record_name = match name {
                        Either::Left(l) => l.to_string(),
                        // hope that there is no cyclic usage between unnamed structures
                        Either::Right(r) => Usage::_to_name(usages, r, cache),
                    };

                    let node_name = match node {
                        ContextPathNode::Struct(_) => "struct",
                        ContextPathNode::Union(_) => "union",
                        _ => unreachable!(),
                    };

                    format!("{}_{}", node_name, record_name)
                },
                ContextPathNode::Var(name) => format!("var_{}", name),
                ContextPathNode::FunRet => "f".to_string(),
                ContextPathNode::Ptr => "p".to_string(),
                ContextPathNode::Array => "arr".to_string(),
                ContextPathNode::Field(name) => format!("de_field_{}", name),
                ContextPathNode::FunParam(name) => format!("de_param_{}", name),
                ContextPathNode::Nest(idx) => format!("de_nest_{}", idx),
            }
        })
        .collect::<Vec<String>>()
        .join("_");

        cache.insert(usr.clone(), result.clone());

        result
    }

    /// Replace all USR usage in [ty] with human readable name with [usages]
    fn replace_usage_in_ty(usages: &HashMap<String, Vec<Usage>>, ty: CType, cache: &mut HashMap<String, String>) -> CType {
        if let CBaseType::Record(is_struct, Either::Right(name)) = &ty.ty {
            return CType { 
                is_const: ty.is_const, 
                ty: CBaseType::Record(*is_struct, Either::Left(Usage::to_name(usages, name, true, cache).interned()))
            }
        }

        ty.map(|ty| {
            Box::new(Usage::replace_usage_in_ty(usages, *ty, cache))
        })
    }
}

/// Collect all usages to unnamed structs.
///  
/// @param usage field/param that uses `ty` directly (`struct { ... } foo`) or indirectly (`struct { ... } ****foo`)
fn collect_usage_on_ty<'m, 't : 'm>(
    ty: &'t CType, 
    dest: &'m mut HashMap<String, Vec<Usage>>, 
    // TODO: get rid of clone
    mut context: Vec<ContextPathNode>
) {
    match &ty.ty {
        CBaseType::Array(ty, _) => {
            context.push(ContextPathNode::Array);
            collect_usage_on_ty(ty, dest, context);
        },
        CBaseType::Pointer(ty) => {
            context.push(ContextPathNode::Ptr);
            collect_usage_on_ty(ty, dest, context);
        },
        CBaseType::FunProto(ret, params) => {
            // TODO: fix clone
            let mut ret_ctx = context.clone();
            ret_ctx.push(ContextPathNode::FunRet);
            collect_usage_on_ty(ret, dest, ret_ctx);

            for (idx, p) in params.iter().enumerate() {
                let name = if let Some(name) = &p.name {
                    name.to_string()
                } else {
                    format!("param{}", idx)
                };

                let mut param_ctx = context.clone();
                param_ctx.push(ContextPathNode::FunParam(name));
                collect_usage_on_ty(&p.ty, dest, param_ctx);
            }
        },
        CBaseType::Record(is_struct, Either::Right(usr)) => {
            let exist = dest.get_mut(usr);
            let usages: &mut Vec<Usage>;
            if let Some(usages_) = exist {
                usages = usages_;
            } else {
                dest.insert(usr.clone(), Vec::new());
                usages = dest.get_mut(usr)
                    .expect("What do you mean I got None right after I insert something to it??");
            }

            usages.push(Usage(context));
        },
        
        CBaseType::Primitive(_) 
        | CBaseType::Record(_, _)
        | CBaseType::Enum(_) 
        | CBaseType::Typedef(_) => {},
    }
}

/// endregion Naming Unnamed Struct

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
