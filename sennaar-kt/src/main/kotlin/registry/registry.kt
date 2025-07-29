package club.doki7.sennaar.registry

import club.doki7.sennaar.Identifier
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonNull

interface IRegistry {
    var name: String
    var imports: MutableSet<Import>
    var metadefs: MutableMap<String, String>
    var aliases: MutableMap<Identifier, Typedef>
    var bitmasks: MutableMap<Identifier, Bitmask>
    var commands: MutableMap<Identifier, Command>
    var constants: MutableMap<Identifier, Constant>
    var enumerations: MutableMap<Identifier, Enumeration>
    var functionTypedefs: MutableMap<Identifier, FunctionTypedef>
    var opaqueTypedefs: MutableMap<Identifier, OpaqueTypedef>
    var opaqueHandleTypedefs: MutableMap<Identifier, OpaqueHandleTypedef>
    var structs: MutableMap<Identifier, Structure>
    var unions: MutableMap<Identifier, Structure>

    fun sanitize() {
        commands.values.forEach { it.sanitize() }
        functionTypedefs.values.forEach { it.sanitize() }
    }

    fun sanitizeFix() {
        commands.values.forEach { it.sanitizeFix() }
        functionTypedefs.values.forEach { it.sanitizeFix() }
    }

    fun mergeBaseWith(other: IRegistry) {
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
    }
}

@Serializable
data class Registry(
    override var name: String,
    override var imports: MutableSet<Import>,
    override var metadefs: MutableMap<String, String>,
    override var aliases: MutableMap<Identifier, Typedef>,
    override var bitmasks: MutableMap<Identifier, Bitmask>,
    override var commands: MutableMap<Identifier, Command>,
    override var constants: MutableMap<Identifier, Constant>,
    override var enumerations: MutableMap<Identifier, Enumeration>,
    override var functionTypedefs: MutableMap<Identifier, FunctionTypedef>,
    override var opaqueTypedefs: MutableMap<Identifier, OpaqueTypedef>,
    override var opaqueHandleTypedefs: MutableMap<Identifier, OpaqueHandleTypedef>,
    override var structs: MutableMap<Identifier, Structure>,
    override var unions: MutableMap<Identifier, Structure>,
    var ext: JsonElement
) : IRegistry {
    constructor(name: String) : this(
        name,
        imports = mutableSetOf(),
        metadefs = mutableMapOf(),
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
}

data class RegistryTE<EXT>(
    override var name: String,
    override var imports: MutableSet<Import>,
    override var metadefs: MutableMap<String, String>,
    override var aliases: MutableMap<Identifier, Typedef>,
    override var bitmasks: MutableMap<Identifier, Bitmask>,
    override var commands: MutableMap<Identifier, Command>,
    override var constants: MutableMap<Identifier, Constant>,
    override var enumerations: MutableMap<Identifier, Enumeration>,
    override var functionTypedefs: MutableMap<Identifier, FunctionTypedef>,
    override var opaqueTypedefs: MutableMap<Identifier, OpaqueTypedef>,
    override var opaqueHandleTypedefs: MutableMap<Identifier, OpaqueHandleTypedef>,
    override var structs: MutableMap<Identifier, Structure>,
    override var unions: MutableMap<Identifier, Structure>,
    var ext: EXT
) : IRegistry {
    constructor(name: String, ext: EXT) : this(
        name,
        imports = mutableSetOf(),
        metadefs = mutableMapOf(),
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
        ext = ext
    )
}
