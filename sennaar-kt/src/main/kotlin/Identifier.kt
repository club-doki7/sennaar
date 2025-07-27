package club.doki7.sennaar

import kotlinx.serialization.KSerializer
import kotlinx.serialization.Serializable
import kotlinx.serialization.descriptors.PrimitiveKind
import kotlinx.serialization.descriptors.PrimitiveSerialDescriptor
import kotlinx.serialization.encoding.Decoder
import kotlinx.serialization.encoding.Encoder

@Serializable(with = IdentifierSerializer::class)
class Identifier internal constructor(val original: String) : Comparable<Identifier> {
    private var _value = original
    private var _renamed = false

    val value get() = _value
    val renamed get() = _renamed

    fun rename(value: String) {
        if (value == _value) {
            return
        }

        if (_renamed) {
            error("Identifier \"$original\" has already been renamed to \"$_value\", colliding with fresh \"$value\"")
        }

        _value = value
        _renamed = true
    }

    override fun toString(): String {
        return _value
    }

    override fun equals(other: Any?) = other is Identifier && original == other.original

    override fun hashCode() = original.hashCode()

    override fun compareTo(other: Identifier): Int = original.compareTo(other.original)
}

object IdentifierSerializer : KSerializer<Identifier> {
    override val descriptor = PrimitiveSerialDescriptor("Identifier", PrimitiveKind.STRING)

    override fun deserialize(decoder: Decoder): Identifier {
        val s = decoder.decodeString()
        if (s.contains(":")) {
            val original = s.substringBefore(":")
            val renamed = s.substringAfter(":", "")

            val ident = original.interned()
            ident.rename(renamed)
            return ident
        }

        return s.interned()
    }

    override fun serialize(encoder: Encoder, value: Identifier) {
        if (value.renamed) {
            encoder.encodeString("${value.original}:${value.value}")
        } else {
            encoder.encodeString(value.original)
        }
    }
}

private val identifiers = mutableMapOf<String, Identifier>()

fun String.interned(): Identifier = identifiers.getOrPut(this) { Identifier(this) }
