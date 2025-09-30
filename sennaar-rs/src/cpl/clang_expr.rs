#![allow(non_upper_case_globals)]

use clang_sys::*;
use std::{borrow::Cow};

use crate::cpl::clang_ty::map_ty;
use crate::{Identifier, Internalize};
use crate::cpl::expr::*;
use crate::cpl::clang_utils::*;

// TODO: improve error reporting
// TODO: improve life time
pub unsafe fn map_nodes(cursor: CXCursor) -> Result<CExpr<'static>, ClangError> {
  unsafe {
    let cursor_kind = clang_getCursorKind(cursor);

    if clang_isExpression(cursor_kind) == 0 { return Err("Cursor doesn't point to an expression".to_string()); }

    let mapped: CExpr = match cursor_kind {
      CXCursor_IntegerLiteral => {
        let result = clang_Cursor_Evaluate(cursor);
        if result.is_null() { return Err("Unable to evaluate an integer literal.".to_string()); }
        let result_kind = clang_EvalResult_getKind(result);

        if result_kind != CXEval_Int { return Err("Unable to evaluate an integer literal to integer.".to_string()) }
        
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
          suffix: Cow::Borrowed(suffix)
        }))
      }
      CXCursor_CharacterLiteral => {
        let result = clang_Cursor_Evaluate(cursor);
        if result.is_null() { return Err("Unable to evaluate a character literal.".to_string()); }
        let result_kind = clang_EvalResult_getKind(result);

        if result_kind != CXEval_Int { return Err("Unable to evaluate a character literal to integer.".to_string()); }

        let codepoint = clang_EvalResult_getAsUnsigned(result);
        let c = char::from_u32(codepoint as u32).ok_or("Unable to convert i32 to char.".to_string())?;

        // let cs = CStr::from_ptr(raw_cs).to_owned();
        // let s = cs.into_string().map_err(|_| "Failed to convert string")?;

        clang_EvalResult_dispose(result);

        CExpr::CharLiteral(Box::new(CCharLiteralExpr {
          value: Cow::Owned(c.escape_default().to_string())
        }))
      }
      CXCursor_DeclRefExpr => {
        CExpr::Identifier(Box::new(CIdentifierExpr {
          ident: get_identifier(cursor)?
        }))
      }
      CXCursor_ArraySubscriptExpr => {
        let children = get_children(cursor);
        if children.len() != 2 { 
          return Err("Size doesn't match(ArraySub)".to_string());
        } else {
          let base = map_nodes(children[0])?;
          let index = map_nodes(children[1])?;

          CExpr::Index(Box::new(CIndexExpr {
            base, index
          }))
        }
      }
      CXCursor_CallExpr => {
        let children = get_children(cursor);
        if children.len() == 0 {
          return Err("Size doesn't match(CallExpr)".to_string());
        } else {
          let callee = map_nodes(children[0])?;
          let args = children.into_iter().skip(1).map(|e| {
            map_nodes(e)
          }).collect::<Result<Vec<CExpr>, String>>()?;

          CExpr::Call(Box::new(CCallExpr {
            callee, args
          }))
        }
      }
      CXCursor_MemberRefExpr => {
        let member = get_identifier(cursor)?;
        let children = get_children(cursor);
        if children.len() != 1 {
          return Err("Size doesn't match(MemberRef)".to_string());
        } else {
          let obj = map_nodes(children[0])?;

          CExpr::Member(Box::new(CMemberExpr {
            obj, member
          }))
        }
      }
      CXCursor_UnaryOperator => {
        let kind = clang_getCursorUnaryOperatorKind(cursor);
        let op_code = match kind {
          CXUnaryOperator_PostInc => either::Left(CPostfixIncDecOp::Inc),
          CXUnaryOperator_PostDec => either::Left(CPostfixIncDecOp::Dec),
          CXUnaryOperator_PreInc => either::Right(CUnaryOp::Inc),
          CXUnaryOperator_PreDec => either::Right(CUnaryOp::Dec),
          CXUnaryOperator_AddrOf=> either::Right(CUnaryOp::AddrOf),
          CXUnaryOperator_Deref => either::Right(CUnaryOp::Deref),
          CXUnaryOperator_Plus => either::Right(CUnaryOp::Plus),
          CXUnaryOperator_Minus => either::Right(CUnaryOp::Minus),
          CXUnaryOperator_Not => either::Right(CUnaryOp::BitNot),
          CXUnaryOperator_LNot => either::Right(CUnaryOp::Not),
          // TODO: there are unhandled operators, but we doesn't expect them.
          _ => unreachable!()
        };

        let children = get_children(cursor);
        if children.len() != 1 { return Err("Size doesn't match(UnaryOperator)".to_string()) }
        let child = map_nodes(children[0])?;

        op_code.either_with(child, |child, op| CExpr::PostfixIncDec(Box::new(CPostfixIncDecExpr {
          expr: child, op
        })), |child, op| CExpr::Unary(Box::new(CUnaryExpr {
          expr: child, op
        })))
      }
      // what?
      CXCursor_UnaryExpr => {
        let children = get_children(cursor);
        let ty = clang_getCursorType(cursor);
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
        let [ casted ] = get_children_n::<1>(cursor)?;
        let ty = clang_getCursorType(cursor);
        let cty = map_ty(ty)?;

        let mapped = map_nodes(casted)?;

        CExpr::Cast(Box::new(CCastExpr {
          expr: mapped,
          ty: CExpr::identifier(format!("{}", cty).interned()),
        }))
      }
      
      // https://clang.llvm.org/doxygen/group__CINDEX__HIGH.html
      CXCursor_BinaryOperator => {
        let kind = clang_getCursorBinaryOperatorKind(cursor);
        let op_code = match kind {
          CXBinaryOperator_Mul => CBinaryOp::Mul,
          CXBinaryOperator_Div => CBinaryOp::Div,
          CXBinaryOperator_Rem => CBinaryOp::Mod,
          CXBinaryOperator_Add => CBinaryOp::Add,
          CXBinaryOperator_Sub => CBinaryOp::Sub,
          CXBinaryOperator_Shl => CBinaryOp::Shl,
          CXBinaryOperator_Shr => CBinaryOp::Shr,
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
          CXBinaryOperator_AndAssign => CBinaryOp::AndAssign,
          CXBinaryOperator_XorAssign => CBinaryOp::XorAssign,
          CXBinaryOperator_OrAssign => CBinaryOp::OrAssign,
          CXBinaryOperator_Comma => CBinaryOp::Comma,
          _ => unreachable!()
        };

        let children = get_children(cursor);
        if children.len() != 2 {
          return Err("Size doesn't match(BinaryOperator)".to_string());
        }

        let lhs = map_nodes(children[0])?;
        let rhs = map_nodes(children[1])?;

        CExpr::Binary(Box::new(CBinaryExpr {
          op: op_code,
          lhs, rhs
        }))
      }

      CXCursor_ConditionalOperator => {
        let children = get_children(cursor);
        if children.len() != 3 { return Err("Size doesn't match(ConditionalOperator)".to_string()); }

        let cond = map_nodes(children[0])?;
        let then = map_nodes(children[1])?;
        let otherwise = map_nodes(children[2])?;

        CExpr::Conditional(Box::new(CConditionalExpr {
          cond, then, otherwise
        }))
      }
      CXCursor_ParenExpr => {
        let children = get_children(cursor);
        if children.len() != 1 { return Err("Size doesn't match(ParenExpr)".to_string()) }
        let expr = map_nodes(children[0])?;

        CExpr::Paren(Box::new(CParenExpr { expr }))
      }
      // We don't know that it is, so let's hope it has only one child.
      // This is typically a implicit cast.
      CXCursor_UnexposedExpr => {
        let children = get_children(cursor);
        if children.len() != 1 {
          return Err("Size doesn't match(UnexposedExpr)".to_string());
        } else {
          map_nodes(children[0])?
        }
      }
      _ => todo!("{}", cursor_kind)
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
      _ => unreachable!()
    }
  }
}