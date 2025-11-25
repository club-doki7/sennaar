#![allow(non_upper_case_globals)]

use clang_sys::*;

use crate::Internalize;
use crate::cpl::{CBaseType, CSign, CType};
use crate::rossetta::clang_utils::*;

// TODO: safety section??
pub unsafe fn map_cursor_ty(cursor: CXCursor) -> Result<CType, ClangError> {
    unsafe {
        let ty = clang_getCursorType(cursor);
        map_ty(ty)
    }
}

pub unsafe fn map_ty(ty: CXType) -> Result<CType, ClangError> {
    unsafe {
        let is_const = clang_isConstQualifiedType(ty) != 0;

        if let Some(prime) = try_map_primitive(ty) {
            return Ok(CType { is_const, ty: prime });
        }

        let cty: CBaseType = match ty.kind {
            CXType_Pointer => {
                let pointee = clang_getPointeeType(ty);
                let mapped = map_ty(pointee)?;
                CBaseType::Pointer(Box::new(mapped))
            }

            // function with parameters
            CXType_FunctionProto => {
                let result = clang_getResultType(ty);
                let params = get_parameters(ty);

                let mapped_result = map_ty(result)?;
                let mapped_params = params
                    .into_iter()
                    .map(|p| map_ty(p))
                    .collect::<Result<Vec<CType>, String>>()?;

                CBaseType::FunProto(Box::new(mapped_result), mapped_params)
            }

            // function with no parameters
            CXType_FunctionNoProto => {
                let result = clang_getResultType(ty);
                let mapped_result = map_ty(result)?;

                CBaseType::FunProto(Box::new(mapped_result), Vec::new())
            }

            CXType_ConstantArray => {
                let element_ty = clang_getArrayElementType(ty);
                let size = clang_getArraySize(ty);
                if size == -1 {
                    unreachable!()
                }

                let mapped_element_ty = map_ty(element_ty)?;

                CBaseType::Array(Box::new(mapped_element_ty), size as u64)
            }

            CXType_IncompleteArray => {
                // We can't handle `int arr[const]`, ty.is_const and element_ty.is_const are both false.
                let element_ty = clang_getArrayElementType(ty);
                let mapped = map_ty(element_ty)?;
                let size = clang_getArraySize(ty);

                println!("My const: {}", is_const);
                println!("My element const: {}", clang_isConstQualifiedType(element_ty) != 0);
                println!("My element const 2: {}", clang_isConstQualifiedType(clang_getElementType(ty)) != 0);
                println!("My size: {}", size);

                CBaseType::Pointer(Box::new(mapped))
            }

            // struct Foo/enum Bar/typedef things
            CXType_Elaborated => {
                let inner = clang_Type_getNamedType(ty);
                // i guess `is_const` is always false
                let mapped = map_ty(inner)?;
                assert!(! mapped.is_const);
                mapped.ty
            }

            CXType_Typedef => {
                let raw_name = clang_getTypedefName(ty);
                let name = from_CXString(raw_name)?;
                CBaseType::Typedef(name.interned())
            }

            CXType_Record => {
                // dont think this works
                // FIXME: doesn't work, removing leading 'struct '
                let raw_name = clang_getTypeSpelling(ty);
                let name_with_struct = from_CXString(raw_name)?;
                if let Some(name) = name_with_struct.strip_prefix("struct ") {
                    CBaseType::Struct(name.interned())
                } else {
                    unreachable!();
                }
            }

            CXType_Enum => {
                let raw_name = clang_getTypeSpelling(ty);
                let name_with_enum = from_CXString(raw_name)?;
                if let Some(name) = name_with_enum.strip_prefix("enum ") {
                    CBaseType::Enum(name.interned())
                } else {
                    unreachable!();
                }
            }

            _ => {
                let raw_type_display = clang_getTypeSpelling(ty);
                let raw_kind_display = clang_getTypeKindSpelling(ty.kind);
                let type_display = from_CXString(raw_type_display)?;
                let kind_display = from_CXString(raw_kind_display)?;
                todo!(
                    "Unhandled type '{}' with kind '{}'",
                    type_display,
                    kind_display
                );
            }
        };

        Ok(CType { is_const, ty: cty })
    }
}

fn try_map_primitive(ty: CXType) -> Option<CBaseType> {
    let ident = match ty.kind {
        CXType_Void => "void",
        CXType_Bool => "bool", // ??
        CXType_UChar | CXType_Char_S | CXType_SChar => "char",
        CXType_UShort | CXType_Short => "short",
        CXType_UInt | CXType_Int => "int",
        CXType_ULong | CXType_Long => "long",
        CXType_ULongLong | CXType_LongLong => "long long",
        CXType_Float => "float",
        CXType_Double => "double",
        CXType_LongDouble => "long double",
        _ => return None,
    };

    let sign = map_primitive_sign(ty);

    Some(CBaseType::Primitive {
        signed: sign,
        ident: ident.interned(),
    })
}

/// If the sign is determined by the type (such as uint128), the implementation should return `CSign::Signed`
fn map_primitive_sign(ty: CXType) -> CSign {
    match ty.kind {
        CXType_UChar | CXType_UShort | CXType_UInt | CXType_ULong | CXType_ULongLong => {
            CSign::Unsigned
        }
        CXType_SChar => CSign::ExplicitSigned,
        _ => CSign::Signed,
    }
}
