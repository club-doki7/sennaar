package club.doki7.sennaar.registry

import club.doki7.sennaar.Identifier
import club.doki7.sennaar.cpl.CExpr
import kotlinx.serialization.Serializable

@Serializable
sealed interface Type

@Serializable
data class IdentifierType(var ident: Identifier) : Type {
    constructor(ident: String) : this(Identifier(ident))
}

@Serializable
data class ArrayType(var element: Type, var length: CExpr?) : Type

@Serializable
data class PointerType(
    var pointee: Type,
    var isConst: Boolean,
    var pointerToOne: Boolean,
    var nullable: Boolean
) : Type
