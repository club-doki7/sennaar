package club.doki7.sennaar.registry

import club.doki7.sennaar.Identifier
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.JsonObject

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
) {
    constructor(name: String) : this(
        name,
        imports = mutableSetOf(),
        aliases = mutableMapOf(),
        bitmasks = mutableMapOf(),
        commands = mutableMapOf(),
        constants = mutableMapOf(),
        enumerations = mutableMapOf(),
        functionTypedefs = mutableMapOf(),
        opaqueTypedefs = mutableMapOf(),
        opaqueHandleTypedefs = mutableMapOf(),
        structs = mutableMapOf(),
        unions = mutableMapOf(),
        ext = JsonNull
    )

    fun sanitize() {
        commands.values.forEach { it.sanitize() }
        functionTypedefs.values.forEach { it.sanitize() }
    }

    fun sanitizeFix() {
        commands.values.forEach { it.sanitizeFix() }
        functionTypedefs.values.forEach { it.sanitizeFix() }
    }

    fun mergeWith(other: Registry) {
        // TODO: unlikely, but how to deal with colliding items?
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
