use std::collections::{BTreeSet, HashMap};

use clang_sys::CXChildVisit_Continue;
use either::Either;
use sennaar::{Identifier, cpl::{CDecl, RecordName}, registry, rossetta::{clang_decl::map_decl, clang_utils::{CXCursorExtension, visit_children}, name_unnamed::name_unnamed_structs, to_registry::to_registry_decl}};

use crate::prelude::{ResultExtension, test_resource_of};

mod prelude;

struct DeclMap {
    typedefs: HashMap<Identifier, CDecl>,
    decls: HashMap<RecordName, CDecl>,
}

fn add_decl(dest: &mut DeclMap, decl: CDecl) {
    // Be careful that different CDecl can have same name, like `Struct` and `Typedef`
    // We can put `Typedef` to another map to avoid this problem.
    if let CDecl::Struct(decl) | CDecl::Union(decl) = &decl {
        if ! decl.is_definition {
            let exist = dest.decls.get(&decl.name);
            if exist.is_some() {
                return;
            }
        }
    }

    // update
    if let CDecl::Typedef(typedef) = &decl {
        dest.typedefs.insert(typedef.name.clone(), decl);
    } else {
        dest.decls.insert(decl.name(), decl);
    }
}

#[test]
fn test_registry() {
    let cursor = test_resource_of(c"test_registry.c");

    let mut all_decls = Vec::<CDecl>::new();
    
    visit_children(cursor, |e, _| {
        if e.is_declaration() {
            let decl = map_decl(e, &mut all_decls)
                .unwrap_or_error(e);

            all_decls.push(decl);
        }
        
        CXChildVisit_Continue
    });

    let named_decl = name_unnamed_structs(all_decls);
    let mut decl_map = DeclMap { typedefs: HashMap::new(), decls: HashMap::new() };

    named_decl.into_iter().for_each(|decl| {
        add_decl(&mut decl_map, decl);
    });

    let mut registry = registry::RegistryBase {
        name: "what".to_string(),
        metadefs: HashMap::new(),
        imports: BTreeSet::new(),
        aliases: HashMap::new(),
        bitmasks: HashMap::new(),
        constants: HashMap::new(),
        commands: HashMap::new(),
        enumerations: HashMap::new(),
        function_typedefs: HashMap::new(),
        opaque_typedefs: HashMap::new(),
        opaque_handle_typedefs: HashMap::new(),
        structs: HashMap::new(),
        unions: HashMap::new(),
    };

    let resolver = |ident: &Identifier| {
        decl_map.decls.get(&Either::Left(ident.clone()))
            .or_else(|| decl_map.typedefs.get(ident))
    };

    decl_map.typedefs.values()
        .for_each(|decl| {
            to_registry_decl(&mut registry, decl, &resolver).unwrap();
        });

    decl_map.decls.values()
        .for_each(|decl| {
            // don't add struct declaration to registry
            if decl.get_record_decl().map(|r| r.is_definition).unwrap_or(true) {
                to_registry_decl(&mut registry, decl, &resolver).unwrap();
            }
        });

    println!("{:#?}", registry);
}
