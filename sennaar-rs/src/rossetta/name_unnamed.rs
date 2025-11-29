use std::collections::HashMap;

use either::Either;

use crate::{Internalize, cpl::{CBaseType, CDecl, CFieldDecl, CRecordDecl, CType, CVarDecl, RecordName}};


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
