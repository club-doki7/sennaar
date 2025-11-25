use std::{
    ffi::c_void,
    ptr::{null, null_mut},
};

use clang_sys::*;
use sennaar::rossetta::{
    clang_expr::{self, map_expr}, clang_ty::{map_cursor_ty, map_ty}, clang_utils::{CXStringToString, from_CXString, get_children, get_cursor_spelling, is_expression}
};

mod prelude;
use prelude::*;

// not really a test, this is used for showing how the ast look like
// also use `clang -Xclang -ast-dump -fsyntax-only sample.c` to display the ast (not identical to the one we get)
#[test]
fn traversal() {
    unsafe {
        let cursor = test_resource();
        clang_visitChildren(
            cursor,
            visitor,
            ((&mut ClientData { level: 0 }) as *mut ClientData).cast(),
        );
    }
}

#[test]
fn real_test_with_assertion() {
    unsafe {
        let cursor = test_resource();

        let fn_foo = find_fn(cursor, "foo");
        let actual = get_children(fn_foo)
            .into_iter()
            .filter(|c| is_expression(*c))
            .map(|e| format!("{}", map_expr(e).unwrap_or_else(|err| error(err, e))))
            .collect::<Vec<String>>();

        let expected = vec![
            "arr[0x0] = a++",
            "arr[0x1] = ++b",
            "arr[0x2] = 0x1 ? !(0x1BF52) : 0x0",
            "foo(&arr, *arr)",
            "(int) 0x114514",
            "a += b",
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
            panic!(
                "Expected {} expressions, but got {} expressions, remaining: {:?}",
                expected.len(),
                actual.len(),
                &actual[expected.len()..]
            )
        }
    }
}

struct ClientData {
    level: u32
}

// traversal the ast
#[allow(non_upper_case_globals)]
extern "C" fn visitor(e: CXCursor, _p: CXCursor, data: *mut c_void) -> CXChildVisitResult {
    unsafe {
        let client_data = &mut *(data as *mut ClientData);
        let level = client_data.level;

        let cursor_kind = clang_getCursorKind(e);

        let kind_spelling = clang_getCursorKindSpelling(cursor_kind);
        let s = from_CXString(kind_spelling).unwrap_or_else(|err| error(err, e));
        print_padding(level);
        println!("Visiting cursor: {}", s);

        if clang_isExpression(cursor_kind) != 0 {
            let mapped = clang_expr::map_expr(e).unwrap_or_else(|err| error(err, e));

            print_padding(level);
            println!("Expr: {}", mapped);
        } else {
            let usr = clang_getCursorUSR(e).try_to_string().unwrap_or_error(e);
            println_with_padding!(level, "USR: {}", usr);
            let anonymous = clang_Cursor_isAnonymous(e) != 0;
            let name = if cursor_kind == CXCursor_StructDecl && anonymous {
                "unnamed".to_string()
            } else {
                get_cursor_spelling(e).unwrap_or_error(e)
            };

            match cursor_kind {
                CXCursor_FunctionDecl => {
                    let ty = clang_getCursorType(e);
                    let cty = map_ty(ty).unwrap_or_error(e);

                    println_with_padding!(level, "Function Name: {}", name);
                    println_with_padding!(level, "Function Type: {}", cty);
                }

                CXCursor_ParmDecl => {
                    let cty = map_cursor_ty(e).unwrap_or_error(e);
                    println_with_padding!(level, "Param Name: {}", name);
                    println_with_padding!(level, "Param Type: {}", cty);
                }

                CXCursor_TypedefDecl => {
                    println_with_padding!(level, "Typedef Name: {}", name);

                    // this is the typedef itself, i.e. `bar` in `typedef foo bar`
                    // let ty = clang_getCursorType(e);
                    // let cty = map_ty(ty).unwrap_or_else(|err| error(err, e));
                    let underlying = clang_getTypedefDeclUnderlyingType(e);
                    // make sure underlying is not unnamed, i really don't want to handle this case
                    let decl = clang_getTypeDeclaration(underlying);
                    let underlying_anonymous = clang_Cursor_isAnonymous(decl) != 0;

                    if ! underlying_anonymous {
                        let cunderlying = map_ty(underlying).unwrap_or_error(e);   
                        println_with_padding!(level, "Typedef Underlying: {}", cunderlying);
                    } else {
                        println_with_padding!(level, "Typedef Underlying: ANONYMOUS");
                    }
                }

                CXCursor_StructDecl => {
                    println_with_padding!(level, "Struct Name: {}", name);
                }

                CXCursor_FieldDecl => {
                    let ty = map_cursor_ty(e).unwrap_or_error(e);
                    println_with_padding!(level, "Field Name: {}", name);
                    println_with_padding!(level, "Field Type: {}", ty);
                }

                CXCursor_EnumDecl => {
                    let ty = clang_getEnumDeclIntegerType(e);
                    let cty = map_ty(ty).unwrap_or_error(e);

                    println_with_padding!(level, "Enum Name: {}", name);
                    println_with_padding!(level, "Enum Type: {}", cty);
                }

                CXCursor_EnumConstantDecl => {
                    let value = clang_getEnumConstantDeclUnsignedValue(e);

                    println_with_padding!(level, "Enum Member Name: {}", name);
                    println_with_padding!(level, "Enum Value: {}", value);
                }

                _ => {}
            }

            client_data.level = level + 1;

            clang_visitChildren(
                e,
                visitor,
                data,
            );

            client_data.level = level;
        }

        CXChildVisit_Continue
    }
}

fn find_fn<S>(root: CXCursor, name: S) -> CXCursor
where
    S: ToString,
{
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
        let found = children.iter().find(|&&c| unsafe {
            let kind = clang_getCursorKind(c);
            kind == CXCursor_CompoundStmt
        });

        if let Some(&found) = found {
            return found;
        } else {
            panic!("Function body of '{}' is not found", name.to_string());
        }
    } else {
        panic!(
            "Function declaration with name '{}' is not found",
            name.to_string()
        );
    }
}
