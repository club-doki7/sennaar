use std::ffi::CStr;

use clang_sys::*;

pub type ClangError = String;

/// convert and consume
#[allow(non_snake_case)]
pub unsafe fn from_CXString(s: CXString) -> Result<String, ClangError> {
    unsafe {
        let raw_cs = clang_getCString(s);
        let owned = CStr::from_ptr(raw_cs)
            .to_str()
            .map_err(|e| e.to_string())?
            .to_owned();
        clang_disposeString(s);
        Ok(owned)
    }
}
pub fn get_cursor_display(cursor: CXCursor) -> Result<String, ClangError> {
    unsafe {
        let raw = clang_getCursorDisplayName(cursor);
        from_CXString(raw)
    }
}

pub fn get_cursor_spelling(cursor: CXCursor) -> Result<String, ClangError> {
    unsafe {
        let raw = clang_getCursorSpelling(cursor);
        from_CXString(raw)
    }
}

// copy from clang-rs
pub fn visit_children<F>(cursor: CXCursor, mut visitor: F)
where
    F: FnMut(CXCursor, CXCursor) -> CXChildVisitResult,
{
    unsafe {
        trait ChildVisitor {
            fn visit(&mut self, cursor: CXCursor, parent: CXCursor) -> CXChildVisitResult;
        }

        extern "C" fn visit(
            cursor: CXCursor,
            parent: CXCursor,
            data_ptr: CXClientData,
        ) -> CXChildVisitResult {
            unsafe {
                let &mut ((), ref mut visitor) =
                    &mut (*(data_ptr as *mut ((), &mut dyn ChildVisitor)));
                visitor.visit(cursor, parent)
            }
        }

        impl<F: FnMut(CXCursor, CXCursor) -> CXChildVisitResult> ChildVisitor for F {
            fn visit(&mut self, cursor: CXCursor, parent: CXCursor) -> CXChildVisitResult {
                self(cursor, parent)
            }
        }

        let mut data = ((), (&mut visitor as &mut dyn ChildVisitor));

        clang_visitChildren(
            cursor,
            visit,
            (&mut data as *mut ((), &mut dyn ChildVisitor)).cast(),
        );
    }
}

pub fn first_children(cursor: CXCursor) -> Option<CXCursor> {
    let mut opt: Option<CXCursor> = None;

    visit_children(cursor, |cursor, _| {
        opt.replace(cursor);
        CXChildVisit_Break
    });

    opt
}

pub fn get_children(cursor: CXCursor) -> Vec<CXCursor> {
    let mut buffer = Vec::<CXCursor>::new();

    visit_children(cursor, |cursor, _| {
        buffer.push(cursor);
        CXVisit_Continue
    });

    buffer
}

pub fn get_kind(cursor: CXCursor) -> CXCursorKind {
    unsafe { clang_getCursorKind(cursor) }
}

pub fn get_kind_spelling(kind: CXCursorKind) -> Result<String, ClangError> {
    unsafe {
        let spelling = clang_getCursorKindSpelling(kind);
        from_CXString(spelling)
    }
}

pub fn is_expression(cursor: CXCursor) -> bool {
    unsafe { clang_isExpression(get_kind(cursor)) != 0 }
}

pub fn get_children_n<const N: usize>(cursor: CXCursor) -> Result<[CXCursor; N], ClangError> {
    let children = get_children(cursor);
    children.try_into().map_err(|v: Vec<CXCursor>| {
        format!(
            "Children size doesn't match, expected {}, but got {}",
            N,
            v.len()
        )
    })
}

pub fn get_parameters(ty: CXType) -> Vec<CXType> {
    unsafe {
        let argc = clang_getNumArgTypes(ty);
        if argc == -1 {
            panic!("Not a function type")
        } else {
            (0..(argc as u32))
                .map(|i| clang_getArgType(ty, i))
                .collect::<Vec<CXType>>()
        }
    }
}
