pub mod prelude;
use clang_sys::{CXChildVisit_Continue, clang_isDeclaration};
use prelude::*;
use sennaar::rossetta::{clang_decl::map_decl, clang_utils::{get_kind, visit_children}};

#[test]
fn test_map_decls() {
  let e = test_resource();
  visit_children(e, |cursor, _| {
    unsafe {
      let kind = get_kind(cursor);

      if clang_isDeclaration(kind) != 0 { 
        let decl = map_decl(cursor).unwrap_or_error(e);
        println!("{}", decl);
      }
      
      CXChildVisit_Continue
    }
  });
}