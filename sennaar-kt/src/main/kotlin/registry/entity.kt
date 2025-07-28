package club.doki7.sennaar.registry

import club.doki7.sennaar.Identifier
import club.doki7.sennaar.cpl.CExpr
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.JsonObject

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
}

@Serializable
data class Typedef(
    override var name: Identifier,
    var target: Type
) : Entity()

@Serializable
enum class Bitwidth { Bit32, Bit64 }

@Serializable
data class Bitmask(
    override var name: Identifier,
    var bitwidth: Bitwidth,
    var bitflags: MutableList<Bitflag>
) : Entity()

@Serializable
data class Bitflag(
    override var name: Identifier,
    var value: CExpr
) : Entity()

@Serializable
data class Command(
    override var name: Identifier,
    var params: MutableList<Param>,
    var result: Type,
    var successCodes: MutableList<CExpr>,
    var errorCodes: MutableList<CExpr>,
    var aliasTo: Identifier?
) : Entity() {
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
    var len: CExpr?,
    var argLen: CExpr?
) : Entity() {
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
) : Entity()

@Serializable
data class Enumeration(
    override var name: Identifier,
    var variants: MutableList<EnumVariant>
) : Entity()

@Serializable
data class EnumVariant(
    override var name: Identifier,
    var value: CExpr
) : Entity()

@Serializable
data class FunctionTypedef(
    override var name: Identifier,
    var params: MutableList<Param>,
    var result: Type,
    var isPointer: Boolean,
    var isNativeAPI: Boolean
) : Entity() {
    fun sanitize() {
        params.forEach { it.sanitize() }
    }

    fun sanitizeFix() {
        params.forEach { it.sanitizeFix() }
    }
}

@Serializable
data class OpaqueTypedef(override var name: Identifier) : Entity()

@Serializable
data class OpaqueHandleTypedef(override var name: Identifier) : Entity()

@Serializable
data class Structure(
    override var name: Identifier,
    var members: MutableList<Member>
) : Entity()

@Serializable
data class Member(
    override var name: Identifier,
    var ty: Type,
    var bits: Int,
    var init: CExpr?,
    var optional: Boolean,
    var len: CExpr?,
    var altLen: CExpr?
) : Entity()

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

@Serializable
data class Registry(
    var name: String,
    var imports: MutableSet<Import>,
    var aliases: MutableMap<Identifier, Typedef>,
    var bitmasks: MutableMap<Identifier, Bitmask>,
    var commands: MutableMap<Identifier, Command>,
    var constants: MutableMap<Identifier, Constant>,
    var enumerations: MutableMap<Identifier, Enumeration>,
    var functionTypedefs: MutableMap<Identifier, FunctionTypedef>,
    var opaqueTypedefs: MutableMap<Identifier, OpaqueTypedef>,
    var opaqueHandleTypedefs: MutableMap<Identifier, OpaqueHandleTypedef>,
    var structs: MutableMap<Identifier, Structure>,
    var unions: MutableMap<Identifier, Structure>,
    var ext: JsonElement
)  {
    fun sanitize() {
        commands.values.forEach { it.sanitize() }
        functionTypedefs.values.forEach { it.sanitize() }
    }

    fun sanitizeFix() {
        commands.values.forEach { it.sanitizeFix() }
        functionTypedefs.values.forEach { it.sanitizeFix() }
    }

    // TODO: unlikely, but how to deal with colliding items?
    fun mergeWith(other: Registry) {
        imports.addAll(other.imports)
        aliases.putAll(other.aliases)
        bitmasks.putAll(other.bitmasks)
        commands.putAll(other.commands)
        constants.putAll(other.constants)
        enumerations.putAll(other.enumerations)
        functionTypedefs.putAll(other.functionTypedefs)
        opaqueTypedefs.putAll(other.opaqueTypedefs)
        opaqueHandleTypedefs.putAll(other.opaqueHandleTypedefs)
        structs.putAll(other.structs)
        unions.putAll(other.unions)

        when (ext) {
            is JsonNull -> {
                ext = other.ext
            }
            is JsonArray if other.ext is JsonArray -> {
                val extArray = ext as JsonArray
                val otherExtArray = other.ext as JsonArray
                ext = JsonArray(extArray + otherExtArray)
            }
            is JsonObject if other.ext is JsonObject -> {
                val extObject = ext as JsonObject
                val otherExtObject = other.ext as JsonObject
                val mergedMap = extObject.toMutableMap()
                mergedMap.putAll(otherExtObject)
                ext = JsonObject(mergedMap)
            }
            else -> {
                error("cannot merge registry $name and ${other.name}: ext $ext and ${other.ext} are not compatible")
            }
        }
    }
}
