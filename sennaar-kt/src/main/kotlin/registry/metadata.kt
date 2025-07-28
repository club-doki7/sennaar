package club.doki7.sennaar.registry

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
sealed interface Metadata {
    @Serializable
    object None : Metadata

    @Serializable
    @SerialName("String")
    data class StringValue(var value: String) : Metadata

    @Serializable
    data class KeyValues(var kvs: MutableMap<String, Metadata>) : Metadata
}

class KeyValuesBuilder {
    val kvs = mutableMapOf<String, Metadata>()

    infix fun String.to(value: String) {
        kvs[this] = Metadata.StringValue(value)
    }

    infix fun String.to(value: Metadata) {
        kvs[this] = value
    }
}

fun metadata(init: KeyValuesBuilder.() -> Unit): Metadata.KeyValues {
    val builder = KeyValuesBuilder()
    builder.init()
    return Metadata.KeyValues(builder.kvs)
}

fun none(): Metadata.None = Metadata.None
