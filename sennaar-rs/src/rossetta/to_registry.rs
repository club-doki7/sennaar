use either::Either;

use crate::{Identifier, Internalize, cpl::*, registry::{self, Param}, rossetta::clang_expr::map_int_literal};

/// @param idx the index of the param, used when the param has no name.
pub fn to_registry_param(param: &CParam, idx: usize) -> Result<registry::Param<'static>, String> {
    // TODO: unnamed case
    let param = registry::Param::new(
        param.name.as_ref()
            .map(|id| id.clone())
            .unwrap_or_else(|| format!("param{}", idx).interned()),
        to_registry_type(&param.ty)?, 
        false, 
        None);
    Ok(param)
}

/// @return None when [cty] contains [FunProto].
pub fn to_registry_type(cty: &CType) -> Result<registry::Type<'static>, String> {
    let ty = match &cty.ty {
        CBaseType::Primitive(primitive) => registry::Type::identifier(format!("{}", primitive).interned()),
        CBaseType::Array(element_type, len) => {
            let len_expr = len.map(|i| CExpr::IntLiteral(Box::new(map_int_literal(i))));
            registry::Type::ArrayType(Box::new(registry::ArrayType {
                element: to_registry_type(&element_type)?,
                length: len_expr,
                is_const: cty.is_const,
            }))
        },
        CBaseType::Pointer(inner) => {
            registry::Type::PointerType(Box::new(registry::PointerType {
                pointee: to_registry_type(&inner)?,
                is_const: inner.is_const,
                pointer_to_one: false,      // TODO: ??
                nullable: false,    // TODO: ??
            }))
        },
        CBaseType::FunProto(_, _) => return Err(format!("Cannot convert a FunProto to Type: {}", cty)),
        CBaseType::Struct(identifier) => registry::Type::identifier(identifier.clone()),
        CBaseType::UnnamedStruct(_) => return Err(format!("Cannot convert a UnnamedStruct to Type, please wrap with a Typedef: {}", cty)),
        CBaseType::Enum(identifier) => registry::Type::identifier(identifier.clone()),
        CBaseType::Typedef(identifier) => registry::Type::identifier(identifier.clone()),
    };

    Ok(ty)
}

pub(crate) fn to_registry_command<'decl, 'de>(
    registry: &mut registry::RegistryBase, name: &Identifier, ret: &CType, params: &Vec<CParam>, is_pointer: bool
) -> Result<(), String> {
    let reg_params = params.iter()
        .enumerate()
        .map(|(idx, p)| to_registry_param(p, idx))
        .collect::<Result<Vec<Param<'static>>, String>>()?;
    
    // typedef void (*foo)(...)
    let def = registry::FunctionTypedef::new(
        name.clone(),
        reg_params,
        to_registry_type(&ret)?,
        is_pointer, 
        false       // TODO: i don't know
    );

    registry.function_typedefs.insert(def.name.clone(), def);
    return Ok(());
}

/// If [decl] is struct declaration (not definition), then no entity will be created, a opaque struct must live in a typedef.
/// 
/// @param resolver resolve the given [RecordName] to [CDecl], return the definition if possible if it is [CDecl::Struct]
pub fn to_registry_decl<'decl, 'de, Resolver: Fn(&Identifier) -> Option<&'decl CDecl>>(
    registry: &mut registry::RegistryBase, decl: &CDecl, resolver: &Resolver
) -> Result<(), String> {
    match &decl {
        CDecl::Typedef(typedef) => {
            match &typedef.underlying.ty {
                CBaseType::Pointer(ptr) => {
                    match &ptr.ty {
                        CBaseType::FunProto(ret, params) => {
                            return to_registry_command(registry, &typedef.name, ret, params, true);
                        }

                        // only named case
                        CBaseType::Struct(name) => {
                            let decl = resolver(name)
                                .ok_or(format!("Cannot resolve {}", name))?
                                .get_record_decl()
                                .ok_or("Expected CStructDecl")?;


                            if ! decl.is_definition {
                                let handle_typedef = registry::OpaqueHandleTypedef::new(typedef.name.clone());

                                // typedef struct _Foo * Bar
                                registry.opaque_handle_typedefs.insert(handle_typedef.name.clone(), handle_typedef);
                                return Ok(());
                            }

                            // otherwise, fall though to alias case
                        }

                        _ => {}
                    }
                }

                CBaseType::Struct(name) => {
                    // identical to opaque handle typedef case, maybe extract
                    let decl = resolver(name)
                        .ok_or(format!("Cannot resolve {}", name))?
                        .get_record_decl()
                        .ok_or("Expected CStructDecl")?;

                    if ! decl.is_definition {
                        let opaque_typedef = registry::OpaqueTypedef::new(typedef.name.clone());
                        // typedef struct Foo Foo;

                        registry.opaque_typedefs.insert(opaque_typedef.name.clone(), opaque_typedef);
                        return Ok(());
                    }

                    // fall though to alias case
                }

                // handle `typedef void F(int i);`
                CBaseType::FunProto(ret, params) => {
                    return to_registry_command(registry, &typedef.name, ret, params, false);
                },

                _ => {}
            }

            // alias case

            let entity = registry::Typedef::new(
                typedef.name.clone(), to_registry_type(&typedef.underlying)?
            );

            registry.aliases.insert(entity.name.clone(), entity);
            return Ok(())
        }
        CDecl::Fn(decl) => {
            let ret = to_registry_type(&decl.ret)?;
            let reg_params = decl.parameters.iter()
                .enumerate()
                .map(|(idx, p)| to_registry_param(p, idx))
                .collect::<Result<Vec<Param<'static>>, String>>()?;

            let command = registry::Command::new(
                decl.name.clone(),
                reg_params,
                ret,
                Vec::new(), 
                Vec::new(),
                None,
            );

            registry.commands.insert(command.name.clone(), command);
            return Ok(())
        },
        CDecl::Struct(_) | CDecl::Union(_) => {
            let record = decl.get_record_decl()
                .expect("unreachable, decl is Struct or Union but get_record_decl == None??");

            if record.is_definition {
                if let Either::Left(ident) = &record.name {
                    let members = record.fields
                        .iter()
                        .map(|field| {
                            Ok(registry::Member::new(
                                field.name.clone(), 
                                to_registry_type(&field.ty)?,
                                None, 
                                None,
                                false, 
                                None,
                            ))
                        })
                        .collect::<Result<Vec<registry::Member<'static>>, String>>()?;
                    let strukt = registry::Structure::new(
                        ident.clone(), members);
                    
                    match &decl {
                        CDecl::Struct(_) => {
                            registry.structs.insert(strukt.name.clone(), strukt);
                        },
                        CDecl::Union(_) => {
                            registry.unions.insert(strukt.name.clone(), strukt);
                        },
                        _ => unreachable!("decl is Struct or Union but not Struct or Union??")
                    }
                    
                    return Ok(())
                } else {
                    return Err(format!("Trying to entitilize a anonymous struct: {}", decl));
                }
            } else {
                return Err(format!("Trying to entitilize a definition of a struct: {}", decl));
            }
        },
        CDecl::Enum(enum_decl) => {
            let variants = enum_decl.members.iter().map(|variant| {
                registry::EnumVariant::new(
                    variant.name.clone(),
                    CExpr::IntLiteral(Box::new(map_int_literal(variant.value))))
            })
            .collect();

            let enume = registry::Enumeration::new(enum_decl.name.clone(), variants);
            registry.enumerations.insert(enume.name.clone(), enume);

            return Ok(())
        },
    }
}
