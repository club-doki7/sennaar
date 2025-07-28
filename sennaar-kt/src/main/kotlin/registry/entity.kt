package club.doki7.sennaar.registry

import club.doki7.sennaar.Identifier
import club.doki7.sennaar.cpl.CExpr
import club.doki7.sennaar.interned
import kotlinx.serialization.Serializable

sealed class Entity {
    abstract var name: Identifier

    var metadata: MutableMap<String, Metadata> = mutableMapOf()
    var doc: MutableList<String> = mutableListOf()
    var platform: Platform? = null

    final override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (other !is Entity || this.javaClass != other.javaClass) return false
        return name == other.name
    }

    final override fun hashCode(): Int = name.hashCode()

    fun rename(newName: String) {
        name.rename(newName)
    }

    fun rename(renamer: (String) -> String) {
        name.rename(renamer(name.value))
    }
}

@Serializable
data class Typedef(
    override var name: Identifier,
    var target: Type
) : Entity() {
    constructor(
        name: String,
        target: Type
    ) : this(name.interned(), target)
}

@Serializable
enum class Bitwidth { Bit32, Bit64 }

@Serializable
data class Bitmask(
    override var name: Identifier,
    var bitwidth: Bitwidth,
    var bitflags: MutableList<Bitflag>
) : Entity() {
    constructor(
        name: String,
        bitwidth: Bitwidth,
        bitflags: MutableList<Bitflag>
    ) : this(name.interned(), bitwidth, bitflags)
}

@Serializable
data class Bitflag(
    override var name: Identifier,
    var value: CExpr
) : Entity() {
    constructor(
        name: String,
        value: CExpr
    ) : this(name.interned(), value)
}

@Serializable
data class Command(
    override var name: Identifier,
    var params: MutableList<Param>,
    var result: Type,
    var successCodes: MutableList<CExpr>,
    var errorCodes: MutableList<CExpr>,
    var aliasTo: Identifier?
) : Entity() {
    constructor(
        name: String,
        params: MutableList<Param>,
        result: Type,
        successCodes: MutableList<CExpr>,
        errorCodes: MutableList<CExpr>,
        aliasTo: Identifier? = null
    ) : this(name.interned(), params, result, successCodes, errorCodes, aliasTo)

    fun sanitize() {
        params.forEach { it.sanitize() }
    }

    fun sanitizeFix() {
        params.forEach { it.sanitizeFix() }
    }
}

@Serializable
data class Param(
    override var name: Identifier,
    var ty: Type,
    var optional: Boolean,
    var len: CExpr?
) : Entity() {
    constructor(
        name: String,
        ty: Type,
        optional: Boolean,
        len: CExpr?
    ) : this(name.interned(), ty, optional, len)

    fun sanitize() {
        val ptrType = ty as? PointerType
        if (ptrType != null && ptrType.nullable != optional) {
            error("for parameter $name of pointer type, pointer nullability (${ptrType.nullable}) does not match parameter optionality (${optional})")
        }
    }

    fun sanitizeFix() {
        val ptrType = ty as? PointerType
        if (ptrType != null && ptrType.nullable != optional) {
            ptrType.nullable = optional
        }
    }
}

@Serializable
data class Constant(
    override var name: Identifier,
    var ty: Type,
    var expr: CExpr
) : Entity() {
    constructor(
        name: String,
        ty: Type,
        expr: CExpr
    ) : this(name.interned(), ty, expr)
}

@Serializable
data class Enumeration(
    override var name: Identifier,
    var variants: MutableList<EnumVariant>
) : Entity() {
    constructor(
        name: String,
        variants: MutableList<EnumVariant>
    ) : this(name.interned(), variants)
}

@Serializable
data class EnumVariant(
    override var name: Identifier,
    var value: CExpr
) : Entity() {
    constructor(
        name: String,
        value: CExpr
    ) : this(name.interned(), value)
}

@Serializable
data class FunctionTypedef(
    override var name: Identifier,
    var params: MutableList<Param>,
    var result: Type,
    var isPointer: Boolean,
    var isNativeAPI: Boolean
) : Entity() {
    constructor(
        name: String,
        params: MutableList<Param>,
        result: Type,
        isPointer: Boolean,
        isNativeAPI: Boolean
    ) : this(name.interned(), params, result, isPointer, isNativeAPI)

    fun sanitize() {
        params.forEach { it.sanitize() }
    }

    fun sanitizeFix() {
        params.forEach { it.sanitizeFix() }
    }
}

@Serializable
data class OpaqueTypedef(override var name: Identifier) : Entity() {
    constructor(name: String) : this(name.interned())
}

@Serializable
data class OpaqueHandleTypedef(override var name: Identifier) : Entity() {
    constructor(name: String) : this(name.interned())
}

@Serializable
data class Structure(
    override var name: Identifier,
    var members: MutableList<Member>
) : Entity() {
    constructor(
        name: String,
        members: MutableList<Member>
    ) : this(name.interned(), members)
}

@Serializable
data class Member(
    override var name: Identifier,
    var ty: Type,
    var bits: Int?,
    var init: CExpr?,
    var optional: Boolean,
    var len: CExpr?
) : Entity() {
    constructor(
        name: String,
        ty: Type,
        bits: Int?,
        init: CExpr?,
        optional: Boolean,
        len: CExpr?
    ) : this(name.interned(), ty, bits, init, optional, len)
}

@Serializable
data class Import(
    var name: String,
    var version: String?,
    var depend: Boolean
) {
    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (other !is Import) return false
        return name == other.name && version == other.version && depend == other.depend
    }

    override fun hashCode(): Int {
        var result = name.hashCode()
        result = 31 * result + (version?.hashCode() ?: 0)
        result = 31 * result + depend.hashCode()
        return result
    }

    operator fun compareTo(other: Import): Int {
        return name.compareTo(other.name)
    }
}
