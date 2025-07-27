package club.doki7.sennaar.registry

import club.doki7.sennaar.registry.Arch.AArch64
import club.doki7.sennaar.registry.Arch.Custom
import club.doki7.sennaar.registry.Arch.RiscV64
import club.doki7.sennaar.registry.Arch.X86
import club.doki7.sennaar.registry.Arch.X86_64
import club.doki7.sennaar.registry.Endian.Big
import club.doki7.sennaar.registry.Endian.Little
import kotlinx.serialization.KSerializer
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.descriptors.PrimitiveKind
import kotlinx.serialization.descriptors.PrimitiveSerialDescriptor
import kotlinx.serialization.descriptors.SerialDescriptor
import kotlinx.serialization.encoding.Decoder
import kotlinx.serialization.encoding.Encoder

@Suppress("ClassName")
@Serializable(with = ArchSerializer::class)
sealed interface Arch {
    object X86 : Arch { override fun toString(): String = "x86" }
    object X86_64 : Arch { override fun toString(): String = "x86_64" }
    object AArch64 : Arch { override fun toString(): String = "aarch64" }
    object RiscV64 : Arch { override fun toString(): String = "riscv64" }
    data class Custom(val name: String) : Arch {
        override fun toString(): String = name
    }
}

fun parseArch(name: String): Arch {
    return when (name) {
        "x86" -> X86
        "x86_64" -> X86_64
        "aarch64" -> AArch64
        "riscv64" -> RiscV64
        else -> Custom(name)
    }
}

object ArchSerializer : KSerializer<Arch> {
    override val descriptor: SerialDescriptor = PrimitiveSerialDescriptor("Arch", PrimitiveKind.STRING)

    override fun deserialize(decoder: Decoder): Arch {
        val archName = decoder.decodeString()
        return when (archName) {
            "x86" -> X86
            "x86_64" -> X86_64
            "aarch64" -> AArch64
            "riscv64" -> RiscV64
            else -> Custom(archName)
        }
    }

    override fun serialize(encoder: Encoder, value: Arch) {
        encoder.encodeString(value.toString())
    }
}

@Serializable
enum class Endian {
    @SerialName("little") Little,
    @SerialName("big") Big;

    override fun toString(): String {
        return when (this) {
            Little -> "little"
            Big -> "big"
        }
    }
}

fun parseEndian(name: String): Endian {
    return when (name) {
        "little" -> Little
        "big" -> Big
        else -> error("Unknown endian: $name")
    }
}

@Serializable(with = OSSerializer::class)
sealed interface OS {
    object Windows : OS { override fun toString(): String = "windows" }
    object Linux : OS { override fun toString(): String = "linux" }
    object MacOS : OS { override fun toString(): String = "macos" }
    class FreeBSD : OS { override fun toString(): String = "freebsd" }
    data class Custom(val name: String) : OS {
        override fun toString(): String = name
    }
}

fun parseOS(name: String): OS {
    return when (name) {
        "windows" -> OS.Windows
        "linux" -> OS.Linux
        "macos" -> OS.MacOS
        "freebsd" -> OS.FreeBSD()
        else -> OS.Custom(name)
    }
}

object OSSerializer : KSerializer<OS> {
    override val descriptor: SerialDescriptor = PrimitiveSerialDescriptor("OS", PrimitiveKind.STRING)

    override fun deserialize(decoder: Decoder): OS {
        val osName = decoder.decodeString()
        return when (osName) {
            "windows" -> OS.Windows
            "linux" -> OS.Linux
            "macos" -> OS.MacOS
            "freebsd" -> OS.FreeBSD()
            else -> OS.Custom(osName)
        }
    }

    override fun serialize(encoder: Encoder, value: OS) {
        encoder.encodeString(value.toString())
    }
}

@Serializable(with = LibcSerializer::class)
sealed interface LibC {
    object MSFT : LibC { override fun toString(): String = "msft" }
    object MUSL : LibC { override fun toString(): String = "musl" }
    object GLibC : LibC { override fun toString(): String = "glibc" }
    data class Custom(val name: String) : LibC {
        override fun toString(): String = name
    }
}

fun parseLibC(name: String): LibC {
    return when (name) {
        "msft" -> LibC.MSFT
        "musl" -> LibC.MUSL
        "glibc" -> LibC.GLibC
        else -> LibC.Custom(name)
    }
}

object LibcSerializer : KSerializer<LibC> {
    override val descriptor: SerialDescriptor = PrimitiveSerialDescriptor("LibC", PrimitiveKind.STRING)

    override fun deserialize(decoder: Decoder): LibC {
        val libcName = decoder.decodeString()
        return when (libcName) {
            "msft" -> LibC.MSFT
            "musl" -> LibC.MUSL
            "glibc" -> LibC.GLibC
            else -> LibC.Custom(libcName)
        }
    }

    override fun serialize(encoder: Encoder, value: LibC) {
        encoder.encodeString(value.toString())
    }
}

@Serializable
sealed interface PlatformSpecifierState<T> {
    @SerialName("Exact")
    data class Exact<T>(val value: T) : PlatformSpecifierState<T>
    @SerialName("Other")
    class Other<T> : PlatformSpecifierState<T>
    @SerialName("Any")
    class Any<T> : PlatformSpecifierState<T>

    fun toStringWithOtherAndAny(other: String, any: String): String {
        return when (this) {
            is Exact -> value.toString()
            is Other -> other
            is Any -> any
        }
    }
}

fun<T> parsePlatformSpecifierState(
    value: String,
    valueParser: (String) -> T,
    other: String,
    any: String
): PlatformSpecifierState<T> {
    return when (value) {
        other -> PlatformSpecifierState.Other()
        any -> PlatformSpecifierState.Any()
        else -> PlatformSpecifierState.Exact(valueParser(value))
    }
}

const val OTHER_ARCH: String = "other_arch"
const val OTHER_OS: String = "other_os"
const val OTHER_LIBC: String = "other_libc"
const val OTHER_CUSTOM: String = "[other]"

const val ANY_ARCH: String = "any_arch"
const val ANY_ENDIAN: String = "any_endian"
const val ANY_OS: String = "any_os"
const val ANY_LIBC: String = "any_libc"
const val ANY_CUSTOM: String = "[any]"

@Serializable
data class Platform(
    var arch: PlatformSpecifierState<Arch>,
    var endian: Endian?,
    var os: PlatformSpecifierState<OS>,
    var libc: PlatformSpecifierState<LibC>,
    var custom: PlatformSpecifierState<String>
) {
    override fun toString(): String {
        return "${arch.toStringWithOtherAndAny(OTHER_ARCH, ANY_ARCH)}-" +
                "${endian?.toString() ?: ANY_ENDIAN}-" +
                "${os.toStringWithOtherAndAny(OTHER_OS, ANY_OS)}-" +
                "${libc.toStringWithOtherAndAny(OTHER_LIBC, ANY_LIBC)}-" +
                custom.toStringWithOtherAndAny(OTHER_CUSTOM, ANY_CUSTOM)
    }
}

fun parsePlatform(s: String): Platform {
    val s = s.lowercase()
    val parts = s.split(":", limit = 5)
    if (parts.size != 5) {
        error("Platform string must have 5 parts")
    }

    val arch: PlatformSpecifierState<Arch> = parsePlatformSpecifierState(
        parts[0],
        ::parseArch,
        OTHER_ARCH,
        ANY_ARCH
    )
    val endian = if (parts[1] == ANY_ENDIAN) {
        null
    } else {
        parseEndian(parts[1])
    }
    val os = parsePlatformSpecifierState(
        parts[2],
        ::parseOS,
        OTHER_OS,
        ANY_OS
    )
    val libc = parsePlatformSpecifierState(
        parts[3],
        ::parseLibC,
        OTHER_OS,
        ANY_OS
    )
    val custom = parsePlatformSpecifierState(
        parts[4],
        { it },
        OTHER_CUSTOM,
        ANY_CUSTOM
    )

    return Platform(arch, endian, os, libc, custom)
}
