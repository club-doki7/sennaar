use std::{ffi::{c_void, CStr}, ptr::{null, null_mut}};

use sennaar::cpl::{clang_expr, clang_ty::map_ty};
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
      CXTranslationUnit_None
    );
    let cursor = clang_getTranslationUnitCursor(unit);
    clang_visitChildren(cursor, visitor, ((&mut ClientData { level: 0 }) as *mut ClientData).cast());
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
    let cxcs = clang_getCursorKindSpelling(cursor_kind);
    let raw_cs = clang_getCString(cxcs);
    let kind_display = CStr::from_ptr(raw_cs).to_owned().into_string().unwrap();
    clang_disposeString(cxcs);
    
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