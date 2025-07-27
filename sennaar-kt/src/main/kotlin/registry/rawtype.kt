package club.doki7.sennaar.registry

import club.doki7.sennaar.Identifier
import club.doki7.sennaar.cpl.CExpr
import kotlinx.serialization.Serializable

@Serializable
sealed interface Type

@Serializable
data class IdentifierType(val ident: Identifier) : Type

@Serializable
data class ArrayType(val element: Type, val length: CExpr) : Type

@Serializable
data class PointerType(
    val pointee: Type,
    var isConst: Boolean,
    var pointerToOne: Boolean,
    var nullable: Boolean
) : Type
