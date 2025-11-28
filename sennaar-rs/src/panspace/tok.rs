use std::borrow::Cow;
use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::SourceLoc;

/// C programming language token kinds.
/// 
/// ## Reference (C23 standard)
/// 
/// [ISO/IEC 9899:2024 - N3320 working draft](https://www.open-std.org/jtc1/sc22/wg14/www/docs/n3220.pdf)
/// 
/// ### A.2.1 Lexical elements
/// 
/// ```bnf
/// (6.4) token:
///      keyword
///      identifier
///      constant
///      string-literal
///      punctuator
/// ```
/// 
/// ### A.2.1 Keywords
/// 
/// ```bnf
/// (6.4.1) keyword: one of
///     alignas       do            int            struct         while
///     alignof       double        long           switch         _Atomic
///     auto          else          nullptr        thread_local   _BigInt
///     bool          enum          register       true           _Complex
///     break         extern        restrict       typedef        _Decimal128
///     case          false         return         typeof         _Decimal32
///     char          float         short          typeof_unqual  _Decimal64
///     const         for           signed         union          _Generic
///     constexpr     goto          sizeof         unsigned       _Imaginary
///     continue      if            static         void           _Noreturn
///     default       inline        static_assert  volatile
/// ```
/// 
/// ### A.2.5 Constants
/// 
/// ```bnf
/// (6.4.4.1) constant:
///      integer-constant
///      floating-constant
///      enumeration-constant // <-- handled as identifier, no separate token kind
///      character-constant
///      predefined-constant  // <-- handled as three keywords
/// 
/// (6.4.4.6) predefined-constant:
///      false
///      true
///      nullptr
/// 
/// ### A.2.7 Punctuators
/// 
/// ```bnf
/// (6.4.6) punctuator: one of
///      [ ] ( ) { } . ->
///      ++ -- & * + - ~ !
///      / % << >> < <= > >= == != ^ | && ||
///      ? : :: ; ...
///      = *= /= %= += -= <<= >>= &= ^= |=
///      , # ##
///      <: :> <% %> %: %:%: // <-- forwarded to their single-character equivalents 
/// ```
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize)]
pub enum TokenKind {
    Identifier,
    StringLiteral,
    IntLiteral,
    FloatLiteral,
    CharLiteral,

    P_LBracket,
    P_RBracket,
    P_LParen,
    P_RParen,
    P_LBrace,
    P_RBrace,
    P_Dot,
    P_Arrow,
    P_DPlus,
    P_DMinus,
    P_Amp,
    P_Aster,
    P_Plus,
    P_Minus,
    P_Tilde,
    P_Excl,
    P_Slash,
    P_Percent,
    P_DLt,
    P_DGt,
    P_Lt,
    P_Lte,
    P_Gt,
    P_Gte,
    P_DEq,
    P_ExclEq,
    P_Caret,
    P_Pipe,
    P_DAmp,
    P_DPipe,
    P_Ques,
    P_Colon,
    P_DColon,
    P_Semicolon,
    P_Ellipsis,
    P_Eq,
    P_AsterEq,
    P_SlashEq,
    P_PercentEq,
    P_PlusEq,
    P_MinusEq,
    P_DLtEq,
    P_DGtEq,
    P_AmpEq,
    P_CaretEq,
    P_PipeEq,
    P_Comma,
    P_Hash,
    P_DHash,

    KW_Alignas,
    KW_Alignof,
    KW_Auto,
    KW_Bool,
    KW_Break,
    KW_Case,
    KW_Char,
    KW_Const,
    KW_Constexpr,
    KW_Continue,
    KW_Default,
    KW_Do,
    KW_Double,
    KW_Else,
    KW_Enum,
    KW_Extern,
    KW_False,
    KW_Float,
    KW_For,
    KW_Goto,
    KW_If,
    KW_Inline,
    KW_Int,
    KW_Long,
    KW_Nullptr,
    KW_Register,
    KW_Restrict,
    KW_Return,
    KW_Short,
    KW_Signed,
    KW_Sizeof,
    KW_Static,
    KW_StaticAssert,
    KW_Struct,
    KW_Switch,
    KW_ThreadLocal,
    KW_True,
    KW_Typedef,
    KW_Typeof,
    KW_TypeofUnqual,
    KW_Union,
    KW_Unsigned,
    KW_Void,
    KW_Volatile,
    KW_While,
    KW_Atomic,
    KW_BigInt,
    KW_Complex,
    KW_Decimal128,
    KW_Decimal32,
    KW_Decimal64,
    KW_Generic,
    KW_Imaginary,
    KW_Noreturn,
}

/// C programming language token.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize)]
pub struct Token<'a> {
    /// The kind of token. See [`TokenKind`].
    pub kind: TokenKind,
    /// The lexeme of the token, if applicable. By this moment, only the following tokens have
    /// lexemes:
    /// - Identifiers
    /// - String literals
    /// - Integer literals
    /// - Floating-point literals
    /// - Character literals
    /// 
    /// For other token kinds, this field is `None`.
    pub lexeme: Option<Cow<'a, str>>,
    /// The source location of the token.
    pub loc: SourceLoc<'a>
}

impl<'a> Token<'a> {
    /// Creates a new token without a lexeme.
    pub fn new(kind: TokenKind, loc: SourceLoc<'a>) -> Self {
        debug_assert!(
            match kind {
                TokenKind::Identifier
                | TokenKind::StringLiteral
                | TokenKind::IntLiteral
                | TokenKind::FloatLiteral
                | TokenKind::CharLiteral => false,
                _ => true
            },
            "TokenKind {:?} requires a lexeme",
            kind
        );

        Self {
            kind,
            lexeme: None,
            loc
        }
    }

    /// Creates a new token with a lexeme.
    pub fn with_lexeme(
        kind: TokenKind,
        lexeme: Cow<'a, str>,
        loc: SourceLoc<'a>
    ) -> Self {
        debug_assert!(
            match kind {
                TokenKind::Identifier
                | TokenKind::StringLiteral
                | TokenKind::IntLiteral
                | TokenKind::FloatLiteral
                | TokenKind::CharLiteral => true,
                _ => false
            },
            "TokenKind {:?} does not support a lexeme",
            kind
        );

        Self {
            kind,
            lexeme: Some(lexeme),
            loc
        }
    }
}

impl<'a> Display for Token<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.lexeme {
            Some(lexeme) => write!(f, "{:?}({})", self.kind, lexeme),
            None => write!(f, "{:?}", self.kind),
        }
    }
}
