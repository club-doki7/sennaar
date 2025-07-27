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
