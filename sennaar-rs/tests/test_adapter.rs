use std::{ffi::{c_void, CStr}, ptr::{null, null_mut}};

use sennaar::cpl::adapter;
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
    clang_visitChildren(cursor, visitor, null_mut());
  }
}

extern "C" fn visitor(e: CXCursor, _p: CXCursor, _data: *mut c_void) -> CXChildVisitResult {
  unsafe {
    let cursor_kind = clang_getCursorKind(e);
    if clang_isExpression(cursor_kind) != 0 {
      let mapped = adapter::map_nodes(e);
      match mapped {
        Ok(ce) => {
          println!("{}", ce);
        }
        Err(err) => {
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
      };
    } else {
      clang_visitChildren(e, visitor, null_mut());
    }

    CXChildVisit_Continue
  }
}