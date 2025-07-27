package club.doki7.sennaar.cpl

import club.doki7.sennaar.Identifier
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
sealed interface CExpr

@Serializable
@SerialName("IntLiteral")
class CIntLiteralExpr(var value: String, var suffix: String) : CExpr

@Serializable
@SerialName("FloatLiteral")
class CFloatLiteralExpr(var value: String, var suffix: String) : CExpr

@Serializable
@SerialName("CharLiteral")
class CCharLiteralExpr(var value: String) : CExpr

@Serializable
@SerialName("StringLiteral")
class CStringLiteralExpr(var value: String) : CExpr

@Serializable
@SerialName("Identifier")
class CIdentifierExpr(var ident: Identifier) : CExpr

@Serializable
@SerialName("Index")
class CIndexExpr(var base: CExpr, var index: CExpr) : CExpr

@Serializable
@SerialName("Call")
class CCallExpr(var callee: CExpr, var args: List<CExpr>) : CExpr

@Serializable
@SerialName("Member")
class CMemberExpr(var obj: CExpr, var member: Identifier) : CExpr

@Serializable
@SerialName("PtrMember")
class CPtrMemberExpr(var obj: CExpr, var member: Identifier) : CExpr

@Serializable
enum class CPostfixIncDecOp { Inc, Dec }

@Serializable
@SerialName("PostfixIncDec")
class CPostfixIncDecExpr(var base: CExpr, var op: CPostfixIncDecOp) : CExpr

@Serializable
enum class CUnaryOp {
    Plus, Minus, Not, BitNot, Deref, AddrOf, SizeOf, AlignOf, Inc, Dec
}

@Serializable
@SerialName("Unary")
class CUnaryExpr(var expr: CExpr, var op: CUnaryOp) : CExpr

@Serializable
@SerialName("Cast")
class CCastExpr(var expr: CExpr, var type: CExpr) : CExpr

@Serializable
enum class CBinaryOp {
    Mul, Div, Mod,
    Add, Sub,
    Shl, Shr,
    Less, Greater, LessEq, GreaterEq,
    Eq, NotEq,
    BitAnd, BitXor, BitOr,
    And, Or, Xor,
    Assign, MulAssign, DivAssign, ModAssign,
    AddAssign, SubAssign,
    ShlAssign, ShrAssign,
    BitAndAssign, BitXorAssign, BitOrAssign,
    AndAssign, OrAssign, XorAssign,
    Comma
}

@Serializable
@SerialName("Binary")
class CBinaryExpr(var op: CBinaryOp, var left: CExpr, var right: CExpr) : CExpr

@Serializable
@SerialName("Conditional")
class CConditionalExpr(var cond: CExpr, var then: CExpr, var otherwise: CExpr) : CExpr
