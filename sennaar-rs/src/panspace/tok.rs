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
///      ,
///      # ##                // <-- handled in macro expansion (mcr.rs) only, no token kinds
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

impl Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::Identifier => write!(f, "<identifier>"),
            TokenKind::StringLiteral => write!(f, "<string>"),
            TokenKind::IntLiteral => write!(f, "<int>"),
            TokenKind::FloatLiteral => write!(f, "<float>"),
            TokenKind::CharLiteral => write!(f, "<char>"),

            TokenKind::P_LBracket => write!(f, "["),
            TokenKind::P_RBracket => write!(f, "]"),
            TokenKind::P_LParen => write!(f, "("),
            TokenKind::P_RParen => write!(f, ")"),
            TokenKind::P_LBrace => write!(f, "{{"),
            TokenKind::P_RBrace => write!(f, "}}"),
            TokenKind::P_Dot => write!(f, "."),
            TokenKind::P_Arrow => write!(f, "->"),
            TokenKind::P_DPlus => write!(f, "++"),
            TokenKind::P_DMinus => write!(f, "--"),
            TokenKind::P_Amp => write!(f, "&"),
            TokenKind::P_Aster => write!(f, "*"),
            TokenKind::P_Plus => write!(f, "+"),
            TokenKind::P_Minus => write!(f, "-"),
            TokenKind::P_Tilde => write!(f, "~"),
            TokenKind::P_Excl => write!(f, "!"),
            TokenKind::P_Slash => write!(f, "/"),
            TokenKind::P_Percent => write!(f, "%"),
            TokenKind::P_DLt => write!(f, "<<"),
            TokenKind::P_DGt => write!(f, ">>"),
            TokenKind::P_Lt => write!(f, "<"),
            TokenKind::P_Lte => write!(f, "<="),
            TokenKind::P_Gt => write!(f, ">"),
            TokenKind::P_Gte => write!(f, ">="),
            TokenKind::P_DEq => write!(f, "=="),
            TokenKind::P_ExclEq => write!(f, "!="),
            TokenKind::P_Caret => write!(f, "^"),
            TokenKind::P_Pipe => write!(f, "|"),
            TokenKind::P_DAmp => write!(f, "&&"),
            TokenKind::P_DPipe => write!(f, "||"),
            TokenKind::P_Ques => write!(f, "?"),
            TokenKind::P_Colon => write!(f, ":"),
            TokenKind::P_DColon => write!(f, "::"),
            TokenKind::P_Semicolon => write!(f, ";"),
            TokenKind::P_Ellipsis => write!(f, "..."),
            TokenKind::P_Eq => write!(f, "="),
            TokenKind::P_AsterEq => write!(f, "*="),
            TokenKind::P_SlashEq => write!(f, "/="),
            TokenKind::P_PercentEq => write!(f, "%="),
            TokenKind::P_PlusEq => write!(f, "+="),
            TokenKind::P_MinusEq => write!(f, "-="),
            TokenKind::P_DLtEq => write!(f, "<<="),
            TokenKind::P_DGtEq => write!(f, ">>="),
            TokenKind::P_AmpEq => write!(f, "&="),
            TokenKind::P_CaretEq => write!(f, "^="),
            TokenKind::P_PipeEq => write!(f, "|="),
            TokenKind::P_Comma => write!(f, ","),

            TokenKind::KW_Alignas => write!(f, "alignas"),
            TokenKind::KW_Alignof => write!(f, "alignof"),
            TokenKind::KW_Auto => write!(f, "auto"),
            TokenKind::KW_Bool => write!(f, "bool"),
            TokenKind::KW_Break => write!(f, "break"),
            TokenKind::KW_Case => write!(f, "case"),
            TokenKind::KW_Char => write!(f, "char"),
            TokenKind::KW_Const => write!(f, "const"),
            TokenKind::KW_Constexpr => write!(f, "constexpr"),
            TokenKind::KW_Continue => write!(f, "continue"),
            TokenKind::KW_Default => write!(f, "default"),
            TokenKind::KW_Do => write!(f, "do"),
            TokenKind::KW_Double => write!(f, "double"),
            TokenKind::KW_Else => write!(f, "else"),
            TokenKind::KW_Enum => write!(f, "enum"),
            TokenKind::KW_Extern => write!(f, "extern"),
            TokenKind::KW_False => write!(f, "false"),
            TokenKind::KW_Float => write!(f, "float"),
            TokenKind::KW_For => write!(f, "for"),
            TokenKind::KW_Goto => write!(f, "goto"),
            TokenKind::KW_If => write!(f, "if"),
            TokenKind::KW_Inline => write!(f, "inline"),
            TokenKind::KW_Int => write!(f, "int"),
            TokenKind::KW_Long => write!(f, "long"),
            TokenKind::KW_Nullptr => write!(f, "nullptr"),
            TokenKind::KW_Register => write!(f, "register"),
            TokenKind::KW_Restrict => write!(f, "restrict"),
            TokenKind::KW_Return => write!(f, "return"),
            TokenKind::KW_Short => write!(f, "short"),
            TokenKind::KW_Signed => write!(f, "signed"),
            TokenKind::KW_Sizeof => write!(f, "sizeof"),
            TokenKind::KW_Static => write!(f, "static"),
            TokenKind::KW_StaticAssert => write!(f, "static_assert"),
            TokenKind::KW_Struct => write!(f, "struct"),
            TokenKind::KW_Switch => write!(f, "switch"),
            TokenKind::KW_ThreadLocal => write!(f, "thread_local"),
            TokenKind::KW_True => write!(f, "true"),
            TokenKind::KW_Typedef => write!(f, "typedef"),
            TokenKind::KW_Typeof => write!(f, "typeof"),
            TokenKind::KW_TypeofUnqual => write!(f, "typeof_unqual"),
            TokenKind::KW_Union => write!(f, "union"),
            TokenKind::KW_Unsigned => write!(f, "unsigned"),
            TokenKind::KW_Void => write!(f, "void"),
            TokenKind::KW_Volatile => write!(f, "volatile"),
            TokenKind::KW_While => write!(f, "while"),
            TokenKind::KW_Atomic => write!(f, "_Atomic"),
            TokenKind::KW_BigInt => write!(f, "_BigInt"),
            TokenKind::KW_Complex => write!(f, "_Complex"),
            TokenKind::KW_Decimal128 => write!(f, "_Decimal128"),
            TokenKind::KW_Decimal32 => write!(f, "_Decimal32"),
            TokenKind::KW_Decimal64 => write!(f, "_Decimal64"),
            TokenKind::KW_Generic => write!(f, "_Generic"),
            TokenKind::KW_Imaginary => write!(f, "_Imaginary"),
            TokenKind::KW_Noreturn => write!(f, "_Noreturn"),
        }
    }
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(lexeme) = &self.lexeme {
            write!(f, "{}", lexeme)
        } else {
            write!(f, "{}", self.kind)
        }
    }
}
