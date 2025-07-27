package club.doki7.sennaar.registry

import club.doki7.sennaar.Identifier
import club.doki7.sennaar.cpl.CExpr
import kotlinx.serialization.Serializable

@Serializable
sealed interface Type

@Serializable
class IdentifierType(val ident: Identifier) : Type

@Serializable
class ArrayType(val element: Type, val length: CExpr) : Type

@Serializable
class PointerType(
    val pointee: Type,
    var isConst: Boolean,
    var pointerToOne: Boolean,
    var nullable: Boolean
)
