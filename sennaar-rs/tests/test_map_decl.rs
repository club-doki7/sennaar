pub mod prelude;
use clang_sys::{CXChildVisit_Continue, clang_isDeclaration};
use prelude::*;
use sennaar::rossetta::{
    clang_decl::map_decl, clang_utils::{get_kind, visit_children}, name_unnamed::name_unnamed_structs
};

#[test]
fn traverse_map_decls() {
    test_map_decls(false);
}

#[test]
fn assert_map_decls() {
    test_map_decls(true);
}

fn test_map_decls(assert: bool) {
    let e = test_resource_of(c"test_map_decl.c");
    let mut extra_decls = Vec::new();
    let mut all_decls = Vec::new();

    let mut expected = vec![
        "struct /* USR: c:test_map_decl.c@S@test_map_decl.c@45 */ { int value; };",
        "struct <USR: c:test_map_decl.c@S@test_map_decl.c@45> topVarDecl;",
        "struct Named { int value; };",
        "struct TypedefUnnamed { int value; };",
        "typedef struct TypedefUnnamed TypedefUnnamed;",
        "struct TypedefNamed { int value; };",
        "typedef struct TypedefNamed TypedefNamed;",
        "struct Nest { struct <USR: c:@S@Nest@S@test_map_decl.c@238> walue; struct <USR: c:@S@Nest@S@test_map_decl.c@238> * pvalue; void (*)(struct <USR: c:@S@Nest@S@test_map_decl.c@289> palue) f; int ualue; <subdecl USR: c:@S@Nest@Ua>; };",
        "struct /* USR: c:@S@Nest@S@test_map_decl.c@238 */ { int value; };",
        "struct /* USR: c:@S@Nest@S@test_map_decl.c@289 */ { int value; };",
        "union /* USR: c:@S@Nest@Ua */ { int indirect0; int indirect1; };",
    ].into_iter();

    visit_children(e, |cursor, _| unsafe {
        let kind = get_kind(cursor);

        if clang_isDeclaration(kind) != 0 {
            let decl = map_decl(cursor, &mut extra_decls).unwrap_or_error(e);
            let display = format!("{}", decl);
            if assert {
                assert_eq!(expected.next(), Some(display.as_str()));
            } else {
                println!("{}", display);
            }

            all_decls.push(decl);
        }

        if ! extra_decls.is_empty() {
            if ! assert {
                println!("All subdecls that is introduced by the previous decl:");
            }

            extra_decls.iter().for_each(|it| {
                let display = format!("{}", it);
                if assert {
                    assert_eq!(expected.next(), Some(display.as_str()));
                } else {
                    println!("{}", display);
                }
            });

            all_decls.append(&mut extra_decls);
        }

        CXChildVisit_Continue
    });

    let new_decls = name_unnamed_structs(all_decls);

    if ! assert {
        println!("After naming:");

        new_decls.iter().for_each(|decl| {
            println!("{}", decl);
        });
    }
}
