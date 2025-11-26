pub mod prelude;
use clang_sys::{CXChildVisit_Continue, clang_isDeclaration};
use prelude::*;
use sennaar::rossetta::{
    clang_decl::map_decl, clang_utils::{get_kind, visit_children}
};

#[test]
fn test_map_decls() {
    let e = test_resource();
    let mut extra_decls = Vec::new();
    
    visit_children(e, |cursor, _| unsafe {
        let kind = get_kind(cursor);

        if clang_isDeclaration(kind) != 0 {
            let decl = map_decl(cursor, &mut extra_decls).unwrap_or_error(e);
            println!("{}", decl);
        }

        if ! extra_decls.is_empty() {
            println!("All subdecls that is introduced by the previous decl:");

            extra_decls.iter().for_each(|it| {
                println!("{}", it);
            });

            extra_decls.clear();
        }

        CXChildVisit_Continue
    });
}
