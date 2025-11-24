#![allow(non_upper_case_globals)]

use clang_sys::*;

use crate::cpl::{CSign, CType};
use crate::rossetta::clang_utils::*;
use crate::{Internalize};

// TODO: safety section??
pub unsafe fn map_cursor_ty(cursor: CXCursor) -> Result<CType, ClangError> {
    unsafe {
        let ty = clang_getCursorType(cursor);
        map_ty(ty)
    }
}

pub unsafe fn map_ty(ty: CXType) -> Result<CType, ClangError> {
    unsafe {
        // TODO: handle "const", see clang_isConstQualifiedType
        if let Some(prime) = try_map_primitive(ty) {
            return Ok(prime);
        }

        let cty = match ty.kind {
            CXType_Pointer => {
                let pointee = clang_getPointeeType(ty);
                let mapped = map_ty(pointee)?;
                CType::Pointer(Box::new(mapped))
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

                CType::FunProto(Box::new(mapped_result), mapped_params)
            }

            // function with no parameters
            CXType_FunctionNoProto => {
                let result = clang_getResultType(ty);
                let mapped_result = map_ty(result)?;

                CType::FunProto(Box::new(mapped_result), Vec::new())
            }

            CXType_ConstantArray => {
                let element_ty = clang_getArrayElementType(ty);
                let size = clang_getArraySize(ty);
                if size == -1 {
                    unreachable!()
                }

                let mapped_element_ty = map_ty(element_ty)?;

                CType::Array(Box::new(mapped_element_ty), size as u64)
            }

            // struct Foo/enum Bar/typedef things
            CXType_Elaborated => {
                let inner = clang_Type_getNamedType(ty);
                map_ty(inner)?
            }

            CXType_Typedef => {
                let raw_name = clang_getTypedefName(ty);
                let name = from_CXString(raw_name)?;
                CType::Typedef(name.interned())
            }

            CXType_Record => {
                // dont think this works
                // FIXME: doesn't work, removing leading 'struct '
                let raw_name = clang_getTypeSpelling(ty);
                let name_with_struct = from_CXString(raw_name)?;
                if let Some(name) = name_with_struct.strip_prefix("struct ") {
                    CType::Struct(name.interned())
                } else {
                    unreachable!();
                }
            }

            CXType_Enum => {
                let raw_name = clang_getTypeSpelling(ty);
                let name_with_enum = from_CXString(raw_name)?;
                if let Some(name) = name_with_enum.strip_prefix("enum ") {
                    CType::Enum(name.interned())
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

        Ok(cty)
    }
}

fn try_map_primitive(ty: CXType) -> Option<CType> {
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

    Some(CType::Primitive {
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
