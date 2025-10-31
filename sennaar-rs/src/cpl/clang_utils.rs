use std::ffi::{c_void, CStr};

use clang_sys::*;

pub type ClangError = String;

/// convert and consume
pub unsafe fn from_CXString(s: CXString) -> Result<String, ClangError> {
  unsafe {
    let raw_cs = clang_getCString(s);
    let owned = CStr::from_ptr(raw_cs).to_str().map_err(|e| e.to_string())?.to_owned();
    clang_disposeString(s);
    Ok(owned)
  }
}

pub fn get_children(cursor: CXCursor) -> Vec<CXCursor> {
  let mut buffer = Vec::<CXCursor>::new();

  extern "C" fn visit(cursor: CXCursor, _: CXCursor, data: CXClientData) -> CXChildVisitResult {
    unsafe {
      let buffer = &mut *(data as *mut Vec<CXCursor>);
      buffer.push(cursor);
      CXChildVisit_Continue
    }
  }

  unsafe {
    clang_visitChildren(cursor, visit, (&mut buffer as *mut Vec<CXCursor>) as *mut c_void);
  }

  buffer
}

pub fn get_kind(cursor: CXCursor) -> CXCursorKind {
  unsafe {
    return clang_getCursorKind(cursor);
  }
}

pub fn is_expression(cursor: CXCursor) -> bool {
  unsafe {
    clang_isExpression(get_kind(cursor)) != 0
  }
}

pub fn get_children_n<const N: usize>(cursor: CXCursor) -> Result<[CXCursor ; N], ClangError> {
  let children = get_children(cursor);
  children.try_into()
    .map_err(|v: Vec<CXCursor>| format!("Children size doesn't match, expected {}, but got {}", N, v.len()))
}

pub fn get_parameters(ty: CXType) -> Vec<CXType> {
  unsafe {
    let argc = clang_getNumArgTypes(ty);
    if argc == -1 {
      panic!("Not a function type")
    } else {
      (0..(argc as u32)).map(|i| clang_getArgType(ty, i)).collect::<Vec<CXType>>()
    }
  }
}