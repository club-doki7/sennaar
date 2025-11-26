#![allow(non_upper_case_globals)]

use clang_sys::*;

use crate::Internalize;
use crate::cpl::{CBaseType, CParam, CPrimitive, CType};
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
                    // It is impossible to get the name of parameter, the information is stored in where this type is used
                    // i.e. for typedef void (*f)(int p);, there is a ParmDecl in the children of typedef.
                    .map(|p| map_ty(p).map(|t| CParam { name: None, ty: t })) 
                    .collect::<Result<Vec<CParam>, String>>()?;

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

                CBaseType::Array(Box::new(mapped_element_ty), Some(size as u64))
            }

            CXType_IncompleteArray => {
                // We can't handle `int arr[const]`, ty.is_const and element_ty.is_const are both false.
                let element_ty = clang_getArrayElementType(ty);
                let mapped = map_ty(element_ty)?;
                // let size = clang_getArraySize(ty);

                // println!("My const: {}", is_const);
                // println!("My element const: {}", clang_isConstQualifiedType(element_ty) != 0);
                // println!("My element const 2: {}", clang_isConstQualifiedType(clang_getElementType(ty)) != 0);
                // println!("My size: {}", size);

                CBaseType::Array(Box::new(mapped), None)
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
                let decl = clang_getTypeDeclaration(ty);
                // TODO: not sure if decl will be invalid
                assert!(clang_isInvalid(decl.kind()) == 0);

                if decl.is_anonymous() {
                    // if the referenced struct is anonymous, we use usr
                    CBaseType::UnnamedStruct(decl.get_usr()?)
                } else {   
                    let raw_name = clang_getTypeSpelling(ty);
                    let name_with_struct = from_CXString(raw_name)?;
                    if let Some(name) = name_with_struct.strip_prefix("struct ") {
                        CBaseType::Struct(name.interned())
                    } else {
                        // this is possible that a record doesnt start with "struct ", like the use of `Foo` in `struct { ... } Foo`
                        assert!(! name_with_struct.is_empty());
                        CBaseType::Struct(name_with_struct.interned())
                    }
                }
            }

            CXType_Enum => {
                let raw_name = clang_getTypeSpelling(ty);
                let name_with_enum = from_CXString(raw_name)?;
                if let Some(name) = name_with_enum.strip_prefix("enum ") {
                    CBaseType::Enum(name.interned())
                } else {
                    // TODO: so this case is also reachable like struct?
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
    let primitive = match ty.kind {
        CXType_Void => CPrimitive::Void,
        CXType_Bool => CPrimitive::Bool,
        CXType_UChar => CPrimitive::UChar,
        CXType_Char_S => CPrimitive::CharS,
        CXType_SChar => CPrimitive::SChar,
        CXType_UShort => CPrimitive::UShort,
        CXType_Short => CPrimitive::Short,
        CXType_UInt => CPrimitive::UInt,
        CXType_Int => CPrimitive::Int,
        CXType_ULong => CPrimitive::ULong,
        CXType_Long => CPrimitive::Long,
        CXType_ULongLong => CPrimitive::ULongLong,
        CXType_LongLong => CPrimitive::LongLong,
        CXType_Float => CPrimitive::Float,
        CXType_Double => CPrimitive::Double,
        CXType_LongDouble => CPrimitive::LongDouble,
        _ => return None,
    };

    Some(CBaseType::Primitive(primitive))
}
