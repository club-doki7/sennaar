use std::{ffi::{CStr, CString}, ptr::{null, null_mut}};

use clang_sys::*;
use sennaar::rossetta::clang_utils::*;

const TEST_RESOURCE_DIR: &'static CStr = c"./tests/resources/";

pub fn test_resource_of(path: &CStr) -> CXCursor {
    let mut base = CString::from(TEST_RESOURCE_DIR)
        .into_bytes();
    base.extend_from_slice(path.to_bytes_with_nul());
    let full_path = CString::from_vec_with_nul(base)
        .unwrap();


    unsafe {
        let index = clang_sys::clang_createIndex(0, 0);
        let unit = clang_sys::clang_parseTranslationUnit(
            index,
            full_path.as_ptr(),
            null(),
            0,
            null_mut(),
            0,
            // this option keeps `typedef`s and macros
            CXTranslationUnit_DetailedPreprocessingRecord,
        );

        clang_getTranslationUnitCursor(unit)
    }
}

pub fn test_resource() -> CXCursor {
    test_resource_of(c"sample.c")
}

#[macro_export]
macro_rules! println_with_padding {
    ($level:ident, $($args:tt)*) => {
        print_padding($level);
        println!($($args)*);
    };
}

pub fn print_padding(level: u32) {
    print!("{}", " ".repeat(level as usize));
}

pub trait ResultExtension<T> {
    fn unwrap_or_error(self, e: CXCursor) -> T;
}

impl<T> ResultExtension<T> for Result<T, ClangError> {
    fn unwrap_or_error(self, e: CXCursor) -> T {
        self.unwrap_or_else(|err| error(err, e))
    }
}

pub fn error(err: String, e: CXCursor) -> ! {
    unsafe {
        let cursor_kind = clang_getCursorKind(e);
        let kind_spelling = clang_getCursorKindSpelling(cursor_kind);
        let kind_display = from_CXString(kind_spelling).unwrap_or_else(|err| error(err, e));

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
        clang_getExpansionLocation(
            loc_start,
            &mut file,
            &mut start_line,
            &mut start_column,
            &mut start_offset,
        );
        clang_getExpansionLocation(
            loc_end,
            &mut file,
            &mut end_line,
            &mut end_column,
            &mut end_offset,
        );

        panic!(
            "Operation failed on cursor[{}({})]: {}({}, {})-{}({}, {}) with {}",
            kind_display,
            cursor_kind,
            start_offset,
            start_line,
            start_column,
            end_offset,
            end_line,
            end_column,
            err
        );
    }
}
