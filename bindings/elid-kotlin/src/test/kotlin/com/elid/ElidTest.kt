package com.elid

import org.junit.jupiter.api.*
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertTrue
import kotlin.test.assertFailsWith
import com.google.gson.Gson
import com.google.gson.JsonObject
import java.io.File

@TestInstance(TestInstance.Lifecycle.PER_CLASS)
class ElidTest {

    companion object {
        @JvmStatic
        @BeforeAll
        fun loadLibrary() {
            // Load the native library
            System.loadLibrary("elid_ffi")
        }
    }

    @Test
    fun `test basic encode with Mini128 profile`() {
        // Create a simple 384-dimensional embedding
        val embedding = List(384) { it.toFloat() / 384.0f }

        val elid = uniEncode(embedding, UniProfile.MINI128)

        assertNotNull(elid, "ELID should not be null")
        assertTrue(elid.isNotEmpty(), "ELID should not be empty")
        assertTrue(elid.startsWith("0"), "Mini128 ELID should start with '0'")
    }

    @Test
    fun `test basic encode with Morton10x10 profile`() {
        val embedding = List(384) { it.toFloat() / 384.0f }

        val elid = uniEncode(embedding, UniProfile.MORTON10X10)

        assertNotNull(elid)
        assertTrue(elid.isNotEmpty())
        // Validate base32hex and expected length (~24 chars including 2-byte header)
        assertTrue(elid.all { it in "0123456789abcdefghijklmnopqrstuv" }, "ELID must be base32hex")
        assertEquals(24, elid.length, "Morton10x10 ELID should be 24 characters")
    }

    @Test
    fun `test basic encode with Hilbert10x10 profile`() {
        val embedding = List(384) { it.toFloat() / 384.0f }

        val elid = uniEncode(embedding, UniProfile.HILBERT10X10)

        assertNotNull(elid)
        assertTrue(elid.isNotEmpty())
        // Validate base32hex and expected length (~24 chars including 2-byte header)
        assertTrue(elid.all { it in "0123456789abcdefghijklmnopqrstuv" }, "ELID must be base32hex")
        assertEquals(24, elid.length, "Hilbert10x10 ELID should be 24 characters")
    }

    @Test
    fun `test decode returns correct byte array`() {
        val embedding = List(384) { it.toFloat() / 384.0f }
        val elid = uniEncode(embedding, UniProfile.MINI128)

        val decoded = uniDecode(elid)

        assertNotNull(decoded)
        assertTrue(decoded.isNotEmpty(), "Decoded bytes should not be empty")

        // Mini128 produces 18 bytes (2-byte header + 16-byte payload)
        assertEquals(18, decoded.size, "Mini128 should produce 18 bytes (2 header + 16 payload)")
    }

    @Test
    fun `test hamming distance between same ELID is zero`() {
        val embedding = List(384) { it.toFloat() / 384.0f }
        val elid = uniEncode(embedding, UniProfile.MINI128)

        val distance = uniHammingDistance(elid, elid)

        assertEquals(0u, distance, "Hamming distance of ELID with itself should be 0")
    }

    @Test
    fun `test hamming distance between different ELIDs`() {
        val embedding1 = List(384) { it.toFloat() / 384.0f }
        val embedding2 = List(384) { (it + 100).toFloat() / 384.0f }

        val elid1 = uniEncode(embedding1, UniProfile.MINI128)
        val elid2 = uniEncode(embedding2, UniProfile.MINI128)

        val distance = uniHammingDistance(elid1, elid2)

        assertTrue(distance > 0u, "Different embeddings should have non-zero Hamming distance")
        assertTrue(distance <= 128u, "Hamming distance should be <= 128 for Mini128")
    }

    @Test
    fun `test encode batch with multiple embeddings`() {
        val embeddings = listOf(
            List(384) { it.toFloat() / 384.0f },
            List(384) { (it + 50).toFloat() / 384.0f },
            List(384) { (it + 100).toFloat() / 384.0f }
        )

        val elids = uniEncodeBatch(embeddings, UniProfile.MINI128)

        assertEquals(3, elids.size, "Should encode all 3 embeddings")
        assertTrue(elids.all { it.startsWith("0") }, "All Mini128 ELIDs should start with '0'")

        // Check that different embeddings produce different ELIDs
        assertEquals(3, elids.toSet().size, "All ELIDs should be unique")
    }

    @Test
    fun `test invalid dimension throws error`() {
        // Too few dimensions (minimum is 64)
        val tooSmall = List(32) { it.toFloat() }

        assertFailsWith<UniElidException.InvalidDimension> {
            uniEncode(tooSmall, UniProfile.MINI128)
        }
    }

    @Test
    fun `test invalid value throws error`() {
        // Create embedding with NaN values
        val invalidEmbedding = List(384) { if (it == 0) Float.NaN else it.toFloat() }

        assertFailsWith<UniElidException.InvalidValue> {
            uniEncode(invalidEmbedding, UniProfile.MINI128)
        }
    }

    @Test
    fun `test invalid encoding throws error`() {
        assertFailsWith<UniElidException.InvalidEncoding> {
            uniDecode("invalid-elid-string")
        }
    }

    @Test
    fun `test profile mismatch in hamming distance`() {
        val embedding = List(384) { it.toFloat() / 384.0f }

        val elidMini = uniEncode(embedding, UniProfile.MINI128)
        val elidMorton = uniEncode(embedding, UniProfile.MORTON10X10)

        // Should throw ProfileMismatch when comparing different profiles
        assertFailsWith<UniElidException.ProfileMismatch> {
            uniHammingDistance(elidMini, elidMorton)
        }
    }

    @Nested
    @DisplayName("Cross-language validation tests")
    inner class CrossLanguageTests {

        private val testVectors: JsonObject by lazy {
            val vectorsFile = File("../test-vectors.json")
            require(vectorsFile.exists()) { "test-vectors.json not found at ${vectorsFile.absolutePath}" }
            Gson().fromJson(vectorsFile.readText(), JsonObject::class.java)
        }

        @Test
        fun `test against reference test vectors for Mini128`() {
            val vectors = testVectors.getAsJsonArray("vectors")

            // Test the first vector
            val vector = vectors[0].asJsonObject
            val embedding = vector.getAsJsonArray("embedding").map { it.asFloat }
            val expectedElid = vector.getAsJsonObject("expected_elids").get("Mini128").asString

            val actualElid = uniEncode(embedding, UniProfile.MINI128)

            assertEquals(expectedElid, actualElid, "Kotlin encoding should match reference test vector")
        }

        @Test
        fun `test against reference test vectors for Morton10x10`() {
            val vectors = testVectors.getAsJsonArray("vectors")

            val vector = vectors[0].asJsonObject
            val embedding = vector.getAsJsonArray("embedding").map { it.asFloat }
            val expectedElid = vector.getAsJsonObject("expected_elids").get("Morton10x10").asString

            val actualElid = uniEncode(embedding, UniProfile.MORTON10X10)

            assertEquals(expectedElid, actualElid, "Kotlin Morton encoding should match reference")
        }

        @Test
        fun `test against reference test vectors for Hilbert10x10`() {
            val vectors = testVectors.getAsJsonArray("vectors")

            val vector = vectors[0].asJsonObject
            val embedding = vector.getAsJsonArray("embedding").map { it.asFloat }
            val expectedElid = vector.getAsJsonObject("expected_elids").get("Hilbert10x10").asString

            val actualElid = uniEncode(embedding, UniProfile.HILBERT10X10)

            assertEquals(expectedElid, actualElid, "Kotlin Hilbert encoding should match reference")
        }

        @Test
        fun `test decode matches reference bytes`() {
            val vectors = testVectors.getAsJsonArray("vectors")

            val vector = vectors[0].asJsonObject
            val expectedBytes = vector.getAsJsonArray("expected_bytes")
                .get("Mini128")
                .asJsonArray
                .map { it.asByte.toUByte() }

            val embedding = vector.getAsJsonArray("embedding").map { it.asFloat }
            val elid = uniEncode(embedding, UniProfile.MINI128)
            val actualBytes = uniDecode(elid)

            assertEquals(expectedBytes, actualBytes, "Decoded bytes should match reference")
        }

        @Test
        fun `test all reference vectors encode correctly`() {
            val vectors = testVectors.getAsJsonArray("vectors")

            vectors.forEach { vectorElement ->
                val vector = vectorElement.asJsonObject
                val embedding = vector.getAsJsonArray("embedding").map { it.asFloat }

                // Test each profile
                UniProfile.values().forEach { profile ->
                    val profileName = when (profile) {
                        UniProfile.MINI128 -> "Mini128"
                        UniProfile.MORTON10X10 -> "Morton10x10"
                        UniProfile.HILBERT10X10 -> "Hilbert10x10"
                    }

                    val expected = vector.getAsJsonObject("expected_elids").get(profileName).asString
                    val actual = uniEncode(embedding, profile)

                    assertEquals(expected, actual,
                        "Vector encoding should match for profile $profileName")
                }
            }
        }
    }

    @Nested
    @DisplayName("Batch processing tests")
    inner class BatchTests {

        @Test
        fun `test batch encode is consistent with single encode`() {
            val embeddings = listOf(
                List(384) { it.toFloat() / 384.0f },
                List(384) { (it + 50).toFloat() / 384.0f },
                List(384) { (it + 100).toFloat() / 384.0f }
            )

            val batchElids = uniEncodeBatch(embeddings, UniProfile.MINI128)

            // Encode individually
            val individualElids = embeddings.map { uniEncode(it, UniProfile.MINI128) }

            assertEquals(individualElids, batchElids,
                "Batch encoding should produce same results as individual encoding")
        }

        @Test
        fun `test empty batch`() {
            val emptyBatch = emptyList<List<Float>>()
            val elids = uniEncodeBatch(emptyBatch, UniProfile.MINI128)

            assertTrue(elids.isEmpty(), "Empty batch should produce empty result")
        }

        @Test
        fun `test large batch performance`() {
            // Create 100 random embeddings
            val largeBatch = List(100) {
                List(384) { (it + kotlin.random.Random.nextInt(1000)).toFloat() / 384.0f }
            }

            val startTime = System.nanoTime()
            val elids = uniEncodeBatch(largeBatch, UniProfile.MINI128)
            val duration = (System.nanoTime() - startTime) / 1_000_000 // Convert to ms

            assertEquals(100, elids.size, "Should encode all 100 embeddings")
            println("Batch encoded 100 embeddings in ${duration}ms")
        }
    }

    @Nested
    @DisplayName("Edge cases and error handling")
    inner class EdgeCaseTests {

        @Test
        fun `test minimum valid dimension (64)`() {
            val minEmbedding = List(64) { it.toFloat() / 64.0f }
            val elid = uniEncode(minEmbedding, UniProfile.MINI128)

            assertNotNull(elid)
            assertTrue(elid.startsWith("0"))
        }

        @Test
        fun `test maximum valid dimension (2048)`() {
            val maxEmbedding = List(2048) { it.toFloat() / 2048.0f }
            val elid = uniEncode(maxEmbedding, UniProfile.MINI128)

            assertNotNull(elid)
            assertTrue(elid.startsWith("0"))
        }

        @Test
        fun `test all zeros embedding`() {
            val zeros = List(384) { 0.0f }
            val elid = uniEncode(zeros, UniProfile.MINI128)

            assertNotNull(elid)
            assertTrue(elid.isNotEmpty())
        }

        @Test
        fun `test all ones embedding`() {
            val ones = List(384) { 1.0f }
            val elid = uniEncode(ones, UniProfile.MINI128)

            assertNotNull(elid)
            assertTrue(elid.isNotEmpty())
        }

        @Test
        fun `test normalized embedding (unit vector)`() {
            // Create a unit vector
            val dim = 384
            val value = 1.0f / kotlin.math.sqrt(dim.toFloat())
            val normalized = List(dim) { value }

            val elid = uniEncode(normalized, UniProfile.MINI128)

            assertNotNull(elid)
            assertTrue(elid.startsWith("0"))
        }

        @Test
        fun `test decode then encode is not necessarily identity`() {
            // Note: decode returns bytes, not the original embedding
            // This test demonstrates that encode/decode is NOT a round trip
            val embedding = List(384) { it.toFloat() / 384.0f }
            val elid1 = uniEncode(embedding, UniProfile.MINI128)
            val bytes = uniDecode(elid1)

            // We can only re-encode if we interpret the bytes somehow,
            // but ELID doesn't support encoding from bytes back to ELID
            // This test just verifies the bytes are stable
            val elid2 = uniEncode(embedding, UniProfile.MINI128)
            assertEquals(elid1, elid2, "Same embedding should always produce same ELID")
        }
    }
}
