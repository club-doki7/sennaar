use std::{ffi::{c_void}, ptr::{null, null_mut}};

use sennaar::cpl::{clang_expr::{self, map_nodes}, clang_ty::map_ty, clang_utils::{from_CXString, get_children, is_expression}};
use clang_sys::*;

#[test]
fn adapt_expr() {
  unsafe {
    let index = clang_sys::clang_createIndex(0, 0);
    let unit = clang_sys::clang_parseTranslationUnit(
      index, 
      c"./tests/resources/sample.c".as_ptr(),
      null(), 0, 
      null_mut(), 0, 
      CXTranslationUnit_DetailedPreprocessingRecord
    );
    let cursor = clang_getTranslationUnitCursor(unit);
    clang_visitChildren(cursor, visitor, ((&mut ClientData { level: 0 }) as *mut ClientData).cast());
  }
}

#[test]
fn real_test_with_assertion() {
  unsafe {
    let index = clang_sys::clang_createIndex(0, 0);
    let unit = clang_sys::clang_parseTranslationUnit(
      index, 
      c"./tests/resources/sample.c".as_ptr(),
      null(), 0, 
      null_mut(), 0, 
      CXTranslationUnit_DetailedPreprocessingRecord
    );

    let cursor = clang_getTranslationUnitCursor(unit);
    
    let fn_foo = find_fn(cursor, "foo");
    let actual = get_children(fn_foo)
      .into_iter()
      .filter(|c| is_expression(*c))
      .map(|e| format!("{}", map_nodes(e).unwrap_or_else(|err| error(err, e))))
      .collect::<Vec<String>>();

    let expected = vec![
      "arr[0x0] = a++",
      "arr[0x1] = ++b",
      "arr[0x2] = 0x1 ? !(0x1BF52) : 0x0",
      "foo(&arr, *arr)",
      "(int) 0x114514",
      "a += b"
    ];

    for i in 0..expected.len() {
      if let Some(actual) = actual.get(i) {
        let expected = expected[i];
        assert_eq!(expected, actual);
      } else {
        panic!("Expected {}-th expression, but got None", i)
      }
    }

    if expected.len() < actual.len() {
      panic!("Expected {} expressions, but got {} expressions, remaining: {:?}", expected.len(), actual.len(), &actual[expected.len()..])
    }
  }
}

struct ClientData {
  level: u32
}

extern "C" fn visitor(e: CXCursor, _p: CXCursor, data: *mut c_void) -> CXChildVisitResult {
  unsafe {
    let client_data = &*(data as *mut ClientData);
    let level = client_data.level;

    let cursor_kind = clang_getCursorKind(e);

    
    let kind_spelling = clang_getCursorKindSpelling(cursor_kind);
    let s = from_CXString(kind_spelling).unwrap_or_else(|err| error(err, e));
    print_padding(level);
    println!("Visiting cursor: {}", s);

    if clang_isExpression(cursor_kind) != 0 {
      let mapped = clang_expr::map_nodes(e)
        .unwrap_or_else(|err| error(err, e));
      
      print_padding(level);
      println!("Expr: {}", mapped);
    } else if cursor_kind == CXCursor_ParmDecl {
      let ty = clang_getCursorType(e);
      let cty = map_ty(ty).unwrap_or_else(|err| error(err, e));

      print_padding(level);
      println!("Type: {}", cty);
    } else {
      match cursor_kind {
        #[allow(non_upper_case_globals)]
        CXCursor_FunctionDecl => {
          let ty = clang_getCursorType(e);
          let cty = map_ty(ty).unwrap_or_else(|err| error(err, e));

          print_padding(level);
          println!("Function Type: {}", cty);
        }

        _ => { }
      }

      clang_visitChildren(e, visitor, ((&mut ClientData { level: level + 1 }) as *mut ClientData).cast());
    }

    CXChildVisit_Continue
  }
}

fn print_padding(level: u32) {
  print!("{}", " ".repeat(level as usize));
}

fn error(err: String, e: CXCursor) -> ! { 
  unsafe {
    let cursor_kind = clang_getCursorKind(e);
    let kind_spelling = clang_getCursorKindSpelling(cursor_kind);
    let kind_display = from_CXString(kind_spelling)
      .unwrap_or_else(|err| error(err, e));
    
    let range = clang_getCursorExtent(e);
    let loc_start = clang_getRangeStart(range);
    let loc_end = clang_getRangeEnd(range);
    let mut file: CXFile = null_mut();
    let mut start_line: u32 = 0;
    let mut start_column: u32 = 0;
    let mut start_offset: u32 = 0;
    let mut end_line: u32 = 0;
    let mut end_column: u32 = 0;
    let mut end_offset: u32 = 0;
    clang_getExpansionLocation(loc_start, &mut file, &mut start_line, &mut start_column, &mut start_offset);
    clang_getExpansionLocation(loc_end, &mut file, &mut end_line, &mut end_column, &mut end_offset);
    
    panic!("Failed to map nodes of cursor[{}({})]: {}({}, {})-{}({}, {}) with {}", 
      kind_display, cursor_kind, 
      start_offset, start_line, start_column,
      end_offset, end_line, end_column,
      err
    );
  }
}

fn find_fn<S>(root: CXCursor, name: S) -> CXCursor
  where 
    S : ToString {
  let top_children = get_children(root);

  let found = top_children.iter().find(|&&c| { 
    unsafe {
      let kind = clang_getCursorKind(c);
      if kind == CXCursor_FunctionDecl {
        let cs = clang_getCursorDisplayName(c);
        let fn_display_name = from_CXString(cs).unwrap();
        // fn_display_name contains type information
        let idx = fn_display_name.find('(').unwrap();
        name.to_string() == fn_display_name[0..idx]
      } else {
        false
      }
    }
  });

  if let Some(&found) = found {
    // find the statement of function decl
    let children = get_children(found);
    let found = children.iter().find(|&&c| {
      unsafe {
        let kind = clang_getCursorKind(c);
        kind == CXCursor_CompoundStmt
      }
    });

    if let Some(&found) = found {
      return found;
    } else {
      panic!("Function body of '{}' is not found", name.to_string());
    }
  } else {
    panic!("Function declaration with name '{}' is not found", name.to_string());
  }
}