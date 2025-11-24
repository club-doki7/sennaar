#![allow(non_upper_case_globals)]

use clang_sys::*;
use std::borrow::Cow;
use std::ptr::null_mut;

use crate::cpl::clang_ty::map_ty;
use crate::cpl::clang_utils::*;
use crate::cpl::expr::*;
use crate::{Identifier, Internalize};

// TODO: improve error reporting
// TODO: improve life time
pub unsafe fn map_nodes(cursor: CXCursor) -> Result<CExpr<'static>, ClangError> {
    unsafe {
        let cursor_kind = clang_getCursorKind(cursor);

        if clang_isExpression(cursor_kind) == 0 {
            return Err("Cursor doesn't point to an expression".to_string());
        }

        let mapped: CExpr = match cursor_kind {
            CXCursor_IntegerLiteral => {
                let result = clang_Cursor_Evaluate(cursor);
                if result.is_null() {
                    return Err("Unable to evaluate an integer literal.".to_string());
                }
                let result_kind = clang_EvalResult_getKind(result);

                if result_kind != CXEval_Int {
                    return Err("Unable to evaluate an integer literal to integer.".to_string());
                }

                let str = if clang_EvalResult_isUnsignedInt(result) != 0 {
                    let u = clang_EvalResult_getAsUnsigned(result);
                    format!("{:#X}", u)
                } else {
                    let i = clang_EvalResult_getAsLongLong(result);
                    format!("{:#X}", i)
                };

                let suffix = get_suffix(cursor);

                clang_EvalResult_dispose(result);

                CExpr::IntLiteral(Box::new(CIntLiteralExpr {
                    value: Cow::Owned(str),
                    suffix: Cow::Borrowed(suffix),
                }))
            }
            CXCursor_CharacterLiteral => {
                let result = clang_Cursor_Evaluate(cursor);
                if result.is_null() {
                    return Err("Unable to evaluate a character literal.".to_string());
                }
                let result_kind = clang_EvalResult_getKind(result);

                if result_kind != CXEval_Int {
                    return Err("Unable to evaluate a character literal to integer.".to_string());
                }

                let codepoint = clang_EvalResult_getAsUnsigned(result);
                let c = char::from_u32(codepoint as u32)
                    .ok_or("Unable to convert i32 to char.".to_string())?;

                // let cs = CStr::from_ptr(raw_cs).to_owned();
                // let s = cs.into_string().map_err(|_| "Failed to convert string")?;

                clang_EvalResult_dispose(result);

                CExpr::CharLiteral(Box::new(CCharLiteralExpr {
                    value: Cow::Owned(c.escape_default().to_string()),
                }))
            }
            CXCursor_DeclRefExpr => CExpr::Identifier(Box::new(CIdentifierExpr {
                ident: get_identifier(cursor)?,
            })),
            CXCursor_ArraySubscriptExpr => {
                let [raw_base, raw_index] = get_children_n::<2>(cursor)?;
                let base = map_nodes(raw_base)?;
                let index = map_nodes(raw_index)?;

                CExpr::Index(Box::new(CIndexExpr { base, index }))
            }
            CXCursor_CallExpr => {
                let children = get_children(cursor);
                if children.len() == 0 {
                    return Err("Size doesn't match(CallExpr)".to_string());
                } else {
                    let callee = map_nodes(children[0])?;
                    let args = children
                        .into_iter()
                        .skip(1)
                        .map(|e| map_nodes(e))
                        .collect::<Result<Vec<CExpr>, String>>()?;

                    CExpr::Call(Box::new(CCallExpr { callee, args }))
                }
            }
            CXCursor_MemberRefExpr => {
                let member = get_identifier(cursor)?;
                let [raw_obj] = get_children_n::<1>(cursor)?;
                let obj = map_nodes(raw_obj)?;

                CExpr::Member(Box::new(CMemberExpr { obj, member }))
            }
            CXCursor_UnaryOperator => {
                let kind = clang_getCursorUnaryOperatorKind(cursor);
                let op_code = match kind {
                    CXUnaryOperator_PostInc => either::Left(CPostfixIncDecOp::Inc),
                    CXUnaryOperator_PostDec => either::Left(CPostfixIncDecOp::Dec),
                    CXUnaryOperator_PreInc => either::Right(CUnaryOp::Inc),
                    CXUnaryOperator_PreDec => either::Right(CUnaryOp::Dec),
                    CXUnaryOperator_AddrOf => either::Right(CUnaryOp::AddrOf),
                    CXUnaryOperator_Deref => either::Right(CUnaryOp::Deref),
                    CXUnaryOperator_Plus => either::Right(CUnaryOp::Plus),
                    CXUnaryOperator_Minus => either::Right(CUnaryOp::Minus),
                    CXUnaryOperator_Not => either::Right(CUnaryOp::BitNot),
                    CXUnaryOperator_LNot => either::Right(CUnaryOp::Not),
                    // TODO: there are unhandled operators, but we doesn't expect them.
                    _ => unreachable!(),
                };

                let [child] = get_children_n(cursor)?;
                let expr = map_nodes(child)?;

                op_code.either_with(
                    expr,
                    |expr, op| CExpr::PostfixIncDec(Box::new(CPostfixIncDecExpr { expr, op })),
                    |expr, op| CExpr::Unary(Box::new(CUnaryExpr { expr, op })),
                )
            }
            // what?
            CXCursor_UnaryExpr => {
                let kind = clang_getCursorUnaryOperatorKind(cursor);
                println!("UnaryExpr kind: {}", kind);

                let range = clang_getCursorExtent(cursor);
                let start = clang_getRangeStart(range);
                let end = clang_getRangeEnd(range);

                let mut line_start = 0u32;
                let mut line_end = 0u32;
                let mut column_start = 0u32;
                let mut column_end = 0u32;
                let mut offset_start = 0u32;
                let mut offset_end = 0u32;

                clang_getFileLocation(
                    start,
                    null_mut(),
                    &mut line_start,
                    &mut column_start,
                    &mut offset_start,
                );
                clang_getFileLocation(
                    end,
                    null_mut(),
                    &mut line_end,
                    &mut column_end,
                    &mut offset_end,
                );

                eprintln!(
                    "UnaryExpr location: {}:{} - {}:{}",
                    line_start, column_start, line_end, column_end
                );
                eprintln!("Offsets: {} - {}", offset_start, offset_end);

                // TODO: @chuigda determine what we should do here. maybe retrieve the original source and parse it ourselves.

                let children = get_children(cursor);
                let ty = clang_getCursorType(cursor);
                println!("Type kind: {}", ty.kind);
                let argc = clang_Cursor_getNumArguments(cursor);
                println!("Children count: {}", children.len());
                println!("argc: {}", argc);
                let result = clang_Cursor_Evaluate(cursor);
                let result_kind = clang_EvalResult_getKind(result);
                println!("result kind: {}", result_kind);
                println!("result value: {}", clang_EvalResult_getAsInt(result));
                clang_EvalResult_dispose(result);

                todo!()
            }
            CXCursor_CStyleCastExpr => {
                let [casted] = get_children_n::<1>(cursor)?;
                let ty = clang_getCursorType(cursor);
                let cty = map_ty(ty)?;

                let mapped = map_nodes(casted)?;

                CExpr::Cast(Box::new(CCastExpr {
                    expr: mapped,
                    ty: CExpr::identifier(format!("{}", cty).interned()),
                }))
            }

            // https://clang.llvm.org/doxygen/group__CINDEX__HIGH.html
            CXCursor_BinaryOperator | CXCursor_CompoundAssignOperator => {
                let kind = clang_getCursorBinaryOperatorKind(cursor);
                let op_code = match kind {
                    CXBinaryOperator_Mul => CBinaryOp::Mul,
                    CXBinaryOperator_Div => CBinaryOp::Div,
                    CXBinaryOperator_Rem => CBinaryOp::Mod,
                    CXBinaryOperator_Add => CBinaryOp::Add,
                    CXBinaryOperator_Sub => CBinaryOp::Sub,
                    CXBinaryOperator_Shl => CBinaryOp::Shl,
                    CXBinaryOperator_Shr => CBinaryOp::Shr,
                    // C++ three-way comparison ("spaceshuttle") operator. We don't support C++ yet so this is okay.
                    // CXBinaryOperator_Cmp => CBinaryOp::Cmp,
                    CXBinaryOperator_LT => CBinaryOp::Less,
                    CXBinaryOperator_GT => CBinaryOp::Greater,
                    CXBinaryOperator_LE => CBinaryOp::LessEq,
                    CXBinaryOperator_GE => CBinaryOp::GreaterEq,
                    CXBinaryOperator_EQ => CBinaryOp::Eq,
                    CXBinaryOperator_NE => CBinaryOp::NotEq,
                    CXBinaryOperator_And => CBinaryOp::BitAnd,
                    CXBinaryOperator_Xor => CBinaryOp::BitXor,
                    CXBinaryOperator_Or => CBinaryOp::BitOr,
                    CXBinaryOperator_LAnd => CBinaryOp::And,
                    CXBinaryOperator_LOr => CBinaryOp::Or,
                    CXBinaryOperator_Assign => CBinaryOp::Assign,
                    CXBinaryOperator_MulAssign => CBinaryOp::MulAssign,
                    CXBinaryOperator_DivAssign => CBinaryOp::DivAssign,
                    CXBinaryOperator_RemAssign => CBinaryOp::ModAssign,
                    CXBinaryOperator_AddAssign => CBinaryOp::AddAssign,
                    CXBinaryOperator_SubAssign => CBinaryOp::SubAssign,
                    CXBinaryOperator_ShlAssign => CBinaryOp::ShlAssign,
                    CXBinaryOperator_ShrAssign => CBinaryOp::ShrAssign,
                    CXBinaryOperator_AndAssign => CBinaryOp::BitAndAssign,
                    CXBinaryOperator_XorAssign => CBinaryOp::BitXorAssign,
                    CXBinaryOperator_OrAssign => CBinaryOp::BitOrAssign,
                    CXBinaryOperator_Comma => CBinaryOp::Comma,
                    _ => unreachable!(),
                };

                let [raw_lhs, raw_rhs] = get_children_n(cursor)?;

                let lhs = map_nodes(raw_lhs)?;
                let rhs = map_nodes(raw_rhs)?;

                CExpr::Binary(Box::new(CBinaryExpr {
                    op: op_code,
                    lhs,
                    rhs,
                }))
            }

            CXCursor_ConditionalOperator => {
                let [raw_cond, raw_then, raw_otherwise] = get_children_n(cursor)?;

                let cond = map_nodes(raw_cond)?;
                let then = map_nodes(raw_then)?;
                let otherwise = map_nodes(raw_otherwise)?;

                CExpr::Conditional(Box::new(CConditionalExpr {
                    cond,
                    then,
                    otherwise,
                }))
            }
            CXCursor_ParenExpr => {
                let [child] = get_children_n(cursor)?;
                let expr = map_nodes(child)?;

                CExpr::Paren(Box::new(CParenExpr { expr }))
            }
            // We don't know that it is, so let's hope it has only one child.
            // This is typically a implicit cast.
            // TODO @chuigda: to summarize what CXCursor_UnexposedExpr represents and handle various cases properly.
            CXCursor_UnexposedExpr => {
                let spelling = clang_getCursorKindSpelling(cursor_kind);
                let s = from_CXString(spelling).unwrap();
                let argc = clang_Cursor_getNumArguments(cursor);
                let children_count = get_children(cursor).len();

                eprintln!(
                    "UnexposedExpr encountered: {}, argc: {}, children_count: {}",
                    s, argc, children_count
                );

                let [child] = get_children_n(cursor)?;
                map_nodes(child)?
            }
            _ => {
                let cs = clang_getCursorKindSpelling(cursor_kind);
                let s = from_CXString(cs).unwrap();
                todo!("unknown cursor kind: {}", s);
            }
        };

        Ok(mapped)
    }
}

// fn map_children<const N: usize>(cursor: Entity) -> Option<[ CExpr ; N ]> {
//   let children = cursor.get_children();
//   if children.len() != N {
//     return None
//   }

//   let mapped_children = children.into_iter()
//     .map(|e| map_nodes(e))
//     .collect::<Option<Vec<CExpr>>>()?;

//   // only fail when the length of [mapped_children] doesn't match, which is impossible here.
//   Some(mapped_children.try_into().unwrap())
// }

/// Get identifier from the display name of [cursor]
unsafe fn get_identifier(cursor: CXCursor) -> Result<Identifier, ClangError> {
    unsafe {
        let display = clang_getCursorDisplayName(cursor);
        let s = from_CXString(display)?;
        Ok(s.interned())
    }
}

unsafe fn get_suffix(cursor: CXCursor) -> &'static str {
    unsafe {
        let ty = clang_getCursorType(cursor);
        match ty.kind {
            CXType_Int => "",
            CXType_UInt => "U",
            CXType_ULong => "UL",
            CXType_ULongLong => "ULL",
            CXType_Long => "L",
            CXType_LongLong => "LL",
            CXType_Float => "F",
            CXType_Double => "",
            CXType_LongDouble => "L",
            _ => unreachable!(),
        }
    }
}
