package com.elid

/**
 * Kotlin-idiomatic extensions and wrappers for ELID operations.
 *
 * This file provides convenience functions and extensions to make working
 * with ELID more natural in Kotlin code.
 */

/**
 * Extension function to encode a FloatArray as an ELID.
 *
 * Example:
 * ```
 * val embedding = floatArrayOf(0.1f, 0.2f, 0.3f, ...)
 * val elid = embedding.toElid(ElidProfile.MINI_128)
 * ```
 */
fun FloatArray.toElid(profile: ElidProfile): String {
    return uniEncode(this.toList(), profile.toUniProfile())
}

/**
 * Extension function to encode a List<Float> as an ELID.
 *
 * Example:
 * ```
 * val embedding = listOf(0.1f, 0.2f, 0.3f, ...)
 * val elid = embedding.toElid(ElidProfile.MINI_128)
 * ```
 */
fun List<Float>.toElid(profile: ElidProfile): String {
    return uniEncode(this, profile.toUniProfile())
}

/**
 * Extension function to decode an ELID string to a ByteArray.
 *
 * Example:
 * ```
 * val elid = "0ab12cd34..."
 * val bytes = elid.decodeElid()
 * ```
 */
fun String.decodeElid(): ByteArray {
    return uniDecode(this).map { it.toByte() }.toByteArray()
}

/**
 * Extension function to compute Hamming distance to another ELID.
 *
 * Example:
 * ```
 * val elid1 = "0ab12cd34..."
 * val elid2 = "0ab12ef56..."
 * val distance = elid1.hammingDistanceTo(elid2)
 * ```
 */
fun String.hammingDistanceTo(other: String): UInt {
    return uniHammingDistance(this, other)
}

/**
 * Encode a batch of embeddings in parallel.
 *
 * Example:
 * ```
 * val embeddings = listOf(
 *     listOf(0.1f, 0.2f, ...),
 *     listOf(0.3f, 0.4f, ...),
 * )
 * val elids = encodeBatch(embeddings, ElidProfile.MINI_128)
 * ```
 */
fun encodeBatch(embeddings: List<List<Float>>, profile: ElidProfile): List<String> {
    return uniEncodeBatch(embeddings, profile.toUniProfile())
}

/**
 * Encode a batch of FloatArray embeddings.
 */
fun encodeBatch(embeddings: List<FloatArray>, profile: ElidProfile): List<String> {
    return uniEncodeBatch(embeddings.map { it.toList() }, profile.toUniProfile())
}

/**
 * Kotlin-idiomatic wrapper for ELID profiles with better naming conventions.
 */
enum class ElidProfile {
    /**
     * Mini128: 128-bit SimHash encoding for semantic similarity.
     * Produces 26-character base32hex strings starting with '0'.
     * Best for: Semantic search, deduplication, nearest neighbor queries.
     */
    MINI_128,

    /**
     * Morton10x10: 10-dimensional Morton (Z-order) curve encoding.
     * Produces 27-character base32hex strings starting with 'g'.
     * Best for: Range queries, spatial indexing, multi-dimensional data.
     */
    MORTON_10X10,

    /**
     * Hilbert10x10: 10-dimensional Hilbert curve encoding.
     * Produces 27-character base32hex strings starting with 'o'.
     * Best for: Better locality preservation than Morton, optimal for spatial queries.
     */
    HILBERT_10X10;

    /**
     * Convert to the underlying UniFFI enum type.
     */
    fun toUniProfile(): UniProfile = when (this) {
        MINI_128 -> UniProfile.MINI128
        MORTON_10X10 -> UniProfile.MORTON10X10
        HILBERT_10X10 -> UniProfile.HILBERT10X10
    }

    companion object {
        /**
         * Create an ElidProfile from a string name (case-insensitive).
         */
        fun fromString(name: String): ElidProfile? = when (name.uppercase().replace("-", "_")) {
            "MINI128", "MINI_128" -> MINI_128
            "MORTON10X10", "MORTON_10X10" -> MORTON_10X10
            "HILBERT10X10", "HILBERT_10X10" -> HILBERT_10X10
            else -> null
        }
    }
}

/**
 * Data class representing an ELID with its profile and string representation.
 */
data class Elid(
    val value: String,
    val profile: ElidProfile
) {
    /**
     * Decode this ELID to raw bytes.
     */
    fun decode(): ByteArray = value.decodeElid()

    /**
     * Compute Hamming distance to another ELID.
     * Both ELIDs must use the Mini128 profile.
     */
    fun hammingDistanceTo(other: Elid): UInt {
        require(this.profile == ElidProfile.MINI_128 && other.profile == ElidProfile.MINI_128) {
            "Hamming distance is only supported for Mini128 profile"
        }
        return value.hammingDistanceTo(other.value)
    }

    /**
     * Check if this ELID is within a given Hamming distance threshold of another ELID.
     */
    fun isNearby(other: Elid, threshold: UInt): Boolean {
        return hammingDistanceTo(other) <= threshold
    }

    override fun toString(): String = value

    companion object {
        /**
         * Create an Elid from an embedding.
         */
        fun from(embedding: List<Float>, profile: ElidProfile): Elid {
            val value = embedding.toElid(profile)
            return Elid(value, profile)
        }

        /**
         * Create an Elid from a FloatArray.
         */
        fun from(embedding: FloatArray, profile: ElidProfile): Elid {
            val value = embedding.toElid(profile)
            return Elid(value, profile)
        }

        /**
         * Parse an ELID string, detecting the profile from the first character.
         */
        fun parse(elidString: String): Elid {
            val profile = when {
                elidString.startsWith('0') -> ElidProfile.MINI_128
                elidString.startsWith('g') -> ElidProfile.MORTON_10X10
                elidString.startsWith('o') -> ElidProfile.HILBERT_10X10
                else -> throw IllegalArgumentException("Unknown ELID profile for string: $elidString")
            }
            return Elid(elidString, profile)
        }
    }
}

/**
 * Result type for ELID operations that may fail.
 */
sealed class ElidResult<out T> {
    data class Success<T>(val value: T) : ElidResult<T>()
    data class Failure(val error: UniElidException) : ElidResult<Nothing>()

    fun getOrNull(): T? = when (this) {
        is Success -> value
        is Failure -> null
    }

    fun getOrThrow(): T = when (this) {
        is Success -> value
        is Failure -> throw error
    }

    inline fun <R> map(transform: (T) -> R): ElidResult<R> = when (this) {
        is Success -> Success(transform(value))
        is Failure -> this
    }

    inline fun onSuccess(action: (T) -> Unit): ElidResult<T> {
        if (this is Success) action(value)
        return this
    }

    inline fun onFailure(action: (UniElidException) -> Unit): ElidResult<T> {
        if (this is Failure) action(error)
        return this
    }
}

/**
 * Safe encoding that returns a Result instead of throwing.
 */
fun List<Float>.toElidOrNull(profile: ElidProfile): ElidResult<String> {
    return try {
        ElidResult.Success(this.toElid(profile))
    } catch (e: UniElidException) {
        ElidResult.Failure(e)
    }
}

/**
 * Safe decoding that returns a Result instead of throwing.
 */
fun String.decodeElidOrNull(): ElidResult<ByteArray> {
    return try {
        ElidResult.Success(this.decodeElid())
    } catch (e: UniElidException) {
        ElidResult.Failure(e)
    }
}

/**
 * Builder for batch encoding operations.
 */
class ElidBatchBuilder {
    private val embeddings = mutableListOf<List<Float>>()

    /**
     * Add an embedding to the batch.
     */
    fun add(embedding: List<Float>) {
        embeddings.add(embedding)
    }

    /**
     * Add an embedding from a FloatArray.
     */
    fun add(embedding: FloatArray) {
        embeddings.add(embedding.toList())
    }

    /**
     * Encode all embeddings in the batch.
     */
    fun encode(profile: ElidProfile): List<String> {
        return encodeBatch(embeddings, profile)
    }

    /**
     * Encode all embeddings and wrap them in Elid objects.
     */
    fun encodeToElids(profile: ElidProfile): List<Elid> {
        return encode(profile).map { Elid(it, profile) }
    }

    /**
     * Get the current size of the batch.
     */
    val size: Int get() = embeddings.size

    /**
     * Clear all embeddings from the batch.
     */
    fun clear() {
        embeddings.clear()
    }
}

/**
 * DSL function for building batches.
 */
inline fun buildElidBatch(profile: ElidProfile, block: ElidBatchBuilder.() -> Unit): List<String> {
    return ElidBatchBuilder().apply(block).encode(profile)
}

/**
 * DSL function for building batches that returns Elid objects.
 */
inline fun buildElidBatchToElids(profile: ElidProfile, block: ElidBatchBuilder.() -> Unit): List<Elid> {
    return ElidBatchBuilder().apply(block).encodeToElids(profile)
}
