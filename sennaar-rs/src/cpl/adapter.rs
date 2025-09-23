use clang_sys::*;
use std::{borrow::Cow, ffi::{c_void, CStr, CString}};

use crate::{cpl::{CBinaryExpr, CBinaryOp}, Identifier, Internalize};
use crate::cpl::{CCallExpr, CCharLiteralExpr, CExpr, CIdentifierExpr, CIndexExpr, CIntLiteralExpr, CMemberExpr};

// TODO: improve error reporting
// TODO: improve life time
pub unsafe fn map_nodes(cursor: CXCursor) -> Result<CExpr<'static>, String> {
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
          clang_EvalResult_getAsUnsigned(result).to_string()
        } else {
          clang_EvalResult_getAsLongLong(result).to_string()
        };

        clang_EvalResult_dispose(result);
        

        CExpr::IntLiteral(Box::new(CIntLiteralExpr {
          value: Cow::Owned(str),
          suffix: Cow::Borrowed("")
        }))
      }
      CXCursor_CharacterLiteral => {
        let result = clang_Cursor_Evaluate(cursor);
        if result.is_null() { return Err("Unable to evaluate a character literal.".to_string()); }
        let result_kind = clang_EvalResult_getKind(result);

        if result_kind != CXEval_StrLiteral { return Err("Unable to evaluate a character literal to string.".to_string()); }

        let raw_cs = clang_EvalResult_getAsStr(result);
        if raw_cs.is_null() { return Err("Failed to get string.".to_string()); }

        let cs = CStr::from_ptr(raw_cs).to_owned();
        let s = cs.into_string().map_err(|_| "Failed to convert string")?;

        clang_EvalResult_dispose(result);

        CExpr::CharLiteral(Box::new(CCharLiteralExpr {
          value: Cow::Owned(s)
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
          _ => todo!()
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
unsafe fn get_identifier(cursor: CXCursor) -> Result<Identifier, String> {
  unsafe {
    let display = clang_getCursorDisplayName(cursor);
    let raw_cs = clang_getCString(display);
    let cs = CStr::from_ptr(raw_cs).to_owned().into_string().map_err(|_| "Failed to convert string")?;

    Ok(cs.interned())
  }
}

fn get_children(cursor: CXCursor) -> Vec<CXCursor> {
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
