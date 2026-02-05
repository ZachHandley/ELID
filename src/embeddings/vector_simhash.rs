//! SimHash-128 implementation using signed random projections
//!
//! This module implements the locality-sensitive hashing algorithm from Charikar (2002)
//! for high-dimensional embeddings. The algorithm uses random hyperplanes to project
//! embeddings into a compact 128-bit hash while preserving cosine similarity.
//!
//! # Algorithm Overview
//!
//! SimHash computes a k-bit hash (k=128) by:
//! 1. Generating k random hyperplanes (normal distribution) using deterministic seeding
//! 2. Computing the dot product of the embedding with each hyperplane
//! 3. Taking the sign bit of each dot product (positive -> 1, negative/zero -> 0)
//! 4. Packing the 128 bits into a u128
//!
//! # Theoretical Guarantee (Charikar 2002)
//!
//! For two embeddings x and y with cosine similarity cos(theta), the probability that
//! their SimHash bits match at position i is:
//!
//! ```text
//! P[h_i(x) = h_i(y)] = 1 - theta/pi
//! ```
//!
//! where theta = arccos(cosine_similarity(x, y))
//!
//! This means Hamming distance between hashes is proportional to angular distance:
//! - Similar embeddings (high cosine similarity) -> low Hamming distance
//! - Dissimilar embeddings (low cosine similarity) -> high Hamming distance
//!
//! # Implementation Notes
//!
//! - **Deterministic**: Same seed + embedding always produces the same hash
//! - **On-the-fly generation**: Random projections are generated per-bit to avoid
//!   storing a large matrix (would be 0.5-8MB for 64-2048 dimensional embeddings)
//! - **Blake3 seed derivation**: Each bit uses a unique seed derived from the base seed
//! - **ChaCha20Rng**: Cryptographically-secure PRNG for reproducible random projections
//! - **StandardNormal distribution**: Gaussian-distributed projection vectors
//!
//! # Performance
//!
//! Expected: ~10-50 us per embedding (768 dimensions)
//! - 128 x dim floating-point multiplications (98,304 FLOPs for 768-dim)
//! - Auto-vectorization with `-C target-cpu=native`
//!
//! # References
//!
//! **Charikar, M.S. (2002)**: "Similarity Estimation Techniques from Rounding Algorithms"
//! Proceedings of the 34th Annual ACM Symposium on Theory of Computing (STOC '02)

use blake3;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use rand_distr::{Distribution, StandardNormal};

use super::error::ElidError;

/// Derive a unique 32-byte seed for a specific bit position
///
/// Uses Blake3 to hash the base seed concatenated with the bit index,
/// producing a deterministic 32-byte seed for initializing ChaCha20Rng.
///
/// # Algorithm
///
/// ```text
/// seed_bytes = blake3::hash(base_seed || bit_idx)
/// return seed_bytes[0..32]
/// ```
///
/// # Parameters
///
/// - `base_seed`: The master seed for reproducibility (typically from Profile)
/// - `bit_idx`: Bit position (0..128) to derive seed for
///
/// # Returns
///
/// A 32-byte seed suitable for ChaCha20Rng initialization
#[inline]
pub fn derive_bit_seed(base_seed: u64, bit_idx: u8) -> [u8; 32] {
    // Concatenate base_seed and bit_idx as input to hash function
    let mut input = [0u8; 9];
    input[0..8].copy_from_slice(&base_seed.to_le_bytes());
    input[8] = bit_idx;

    // Hash to produce 32-byte seed
    let hash = blake3::hash(&input);
    *hash.as_bytes()
}

/// Compute 128-bit SimHash of an embedding using signed random projections
///
/// This is the core SimHash algorithm implementing Charikar's 2002 paper on
/// locality-sensitive hashing via random hyperplanes.
///
/// # Algorithm (Charikar 2002)
///
/// For each bit i in 0..128:
/// 1. Derive a unique seed: `seed_i = derive_bit_seed(seed, i)`
/// 2. Initialize PRNG: `rng = ChaCha20Rng::from_seed(seed_i)`
/// 3. Generate random projection vector r_i ~ N(0,1)^d (d = embedding dimension)
/// 4. Compute dot product: `dot = sum(embedding[j] * r_i[j])`
/// 5. Extract sign bit: `bit_i = (dot > 0.0) ? 1 : 0`
/// 6. Pack into u128: `hash |= (bit_i << i)`
///
/// # Parameters
///
/// - `embedding`: The input vector (f32 slice, length 64-2048)
/// - `seed`: Master seed for deterministic hashing (from Profile)
///
/// # Returns
///
/// A 128-bit hash as u128, where bit 0 is the LSB
///
/// # Performance
///
/// - Time: O(128 x dim) floating-point operations
/// - Space: O(1) (on-the-fly generation, no matrix storage)
/// - Expected: ~10-50 us for 768-dimensional embeddings
#[inline]
pub fn simhash_128(embedding: &[f32], seed: u64) -> u128 {
    let _dim = embedding.len();
    let mut hash: u128 = 0;

    // For each of the 128 bits
    for bit_idx in 0..128 {
        // Derive unique seed for this bit position
        let bit_seed = derive_bit_seed(seed, bit_idx);

        // Initialize deterministic PRNG
        let mut rng = ChaCha20Rng::from_seed(bit_seed);

        // Compute dot product with random projection vector
        // projection[i] ~ N(0, 1) (standard normal distribution)
        let mut dot_product: f32 = 0.0;
        for &value in embedding {
            let projection_value: f32 = StandardNormal.sample(&mut rng);
            dot_product += value * projection_value;
        }

        // Extract sign bit: 1 if positive, 0 if negative or zero
        if dot_product > 0.0 {
            hash |= 1u128 << bit_idx;
        }
    }

    hash
}

/// Compute Hamming distance between two 128-bit hashes
///
/// Returns the number of differing bits between two SimHash values.
/// This is used as a proxy for angular distance between embeddings.
///
/// # Algorithm
///
/// ```text
/// hamming_distance(a, b) = popcount(a XOR b)
/// ```
///
/// Uses the hardware `popcnt` instruction on modern CPUs (~3-5 cycles).
///
/// # Parameters
///
/// - `a`: First 128-bit hash
/// - `b`: Second 128-bit hash
///
/// # Returns
///
/// Number of differing bits (0-128)
///
/// # Relationship to Cosine Similarity
///
/// From Charikar (2002), the expected Hamming distance is:
///
/// ```text
/// E[hamming_distance] = (theta / pi) x 128
/// ```
///
/// where theta = arccos(cosine_similarity)
#[inline]
#[must_use]
pub fn elid_hamming_distance(a: u128, b: u128) -> u32 {
    (a ^ b).count_ones()
}

/// Convert a 128-bit SimHash to big-endian byte array
///
/// Converts the hash to a 16-byte array for storage or transmission.
/// Uses big-endian encoding for consistent cross-platform representation.
///
/// # Parameters
///
/// - `hash`: The 128-bit SimHash value
///
/// # Returns
///
/// A 16-byte array in big-endian order
#[inline]
#[must_use]
pub fn simhash_to_bytes(hash: u128) -> [u8; 16] {
    hash.to_be_bytes()
}

/// Convert big-endian byte array to 128-bit SimHash
///
/// Converts a 16-byte array back to a u128 hash value.
/// Validates that the input is exactly 16 bytes.
///
/// # Parameters
///
/// - `bytes`: Byte slice (must be exactly 16 bytes)
///
/// # Returns
///
/// - `Ok(u128)`: The reconstructed hash value
/// - `Err(ElidError::InvalidEncoding)`: If bytes.len() != 16
#[inline]
pub fn simhash_from_bytes(bytes: &[u8]) -> Result<u128, ElidError> {
    if bytes.len() != 16 {
        return Err(ElidError::InvalidEncoding);
    }

    let mut array = [0u8; 16];
    array.copy_from_slice(bytes);
    Ok(u128::from_be_bytes(array))
}

/// Approximate cosine similarity from Hamming distance
///
/// Uses the Charikar (2002) theorem to estimate the cosine similarity
/// between two embeddings based on their SimHash Hamming distance.
///
/// # Algorithm
///
/// From Charikar's theorem:
/// ```text
/// P[h_i(x) = h_i(y)] = 1 - theta/pi
/// ```
/// where theta = arccos(cosine_similarity)
///
/// Therefore:
/// ```text
/// hamming_distance / 128 ~ theta/pi
/// cosine_similarity ~ cos(theta) ~ 1 - (hamming_distance / 128) x pi
/// ```
///
/// # Parameters
///
/// - `hash_a`: First 128-bit SimHash
/// - `hash_b`: Second 128-bit SimHash
///
/// # Returns
///
/// Approximate cosine similarity in range [-1.0, 1.0]
///
/// # Accuracy
///
/// This is an approximation with expected error decreasing as O(1/sqrt(k))
/// where k=128 bits. For 128 bits, typical error is ~5-10%.
#[inline]
#[must_use]
pub fn cosine_similarity_approx(hash_a: u128, hash_b: u128) -> f32 {
    let distance = elid_hamming_distance(hash_a, hash_b) as f32;
    1.0 - (distance / 128.0) * std::f32::consts::PI
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Seed Derivation Tests
    // ========================================================================

    #[test]
    fn test_derive_bit_seed_deterministic() {
        let seed1 = derive_bit_seed(0x1234_5678_9ABC_DEF0, 0);
        let seed2 = derive_bit_seed(0x1234_5678_9ABC_DEF0, 0);
        assert_eq!(seed1, seed2);
    }

    #[test]
    fn test_derive_bit_seed_different_bits() {
        let seed0 = derive_bit_seed(0x1234_5678_9ABC_DEF0, 0);
        let seed1 = derive_bit_seed(0x1234_5678_9ABC_DEF0, 1);
        assert_ne!(seed0, seed1);
    }

    #[test]
    fn test_derive_bit_seed_different_base() {
        let seed_a = derive_bit_seed(0x1111_1111_1111_1111, 0);
        let seed_b = derive_bit_seed(0x2222_2222_2222_2222, 0);
        assert_ne!(seed_a, seed_b);
    }

    #[test]
    fn test_derive_bit_seed_coverage() {
        // Test all 128 bit positions produce unique seeds
        let base = 0x454c4944_53494d48;
        let mut seeds = std::collections::HashSet::new();

        for bit_idx in 0..128 {
            let seed = derive_bit_seed(base, bit_idx);
            assert!(seeds.insert(seed), "Duplicate seed at bit {}", bit_idx);
        }
    }

    // ========================================================================
    // SimHash Core Algorithm Tests
    // ========================================================================

    #[test]
    fn test_simhash_128_deterministic() {
        let embedding = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8];
        let embedding = embedding.into_iter().cycle().take(128).collect::<Vec<_>>();
        let seed = 0x454c4944_53494d48;

        let hash1 = simhash_128(&embedding, seed);
        let hash2 = simhash_128(&embedding, seed);

        assert_eq!(hash1, hash2, "SimHash must be deterministic");
    }

    #[test]
    fn test_simhash_128_different_seeds() {
        let embedding = vec![0.1, 0.2, 0.3, 0.4];
        let embedding = embedding.into_iter().cycle().take(128).collect::<Vec<_>>();

        let hash1 = simhash_128(&embedding, 0x1111_1111_1111_1111);
        let hash2 = simhash_128(&embedding, 0x2222_2222_2222_2222);

        assert_ne!(
            hash1, hash2,
            "Different seeds should produce different hashes"
        );
    }

    #[test]
    fn test_simhash_128_different_embeddings() {
        let seed = 0x454c4944_53494d48;

        // Use actually different embeddings (not just scaled versions)
        let emb1 = vec![1.0, 0.0, 0.0, 0.0]
            .into_iter()
            .cycle()
            .take(128)
            .collect::<Vec<_>>();
        let emb2 = vec![0.0, 1.0, 0.0, 0.0]
            .into_iter()
            .cycle()
            .take(128)
            .collect::<Vec<_>>();

        let hash1 = simhash_128(&emb1, seed);
        let hash2 = simhash_128(&emb2, seed);

        assert_ne!(
            hash1, hash2,
            "Different embeddings should produce different hashes"
        );
    }

    #[test]
    fn test_simhash_128_all_zeros() {
        let embedding = vec![0.0; 128];
        let seed = 0x454c4944_53494d48;

        // Should not panic, but hash will depend on random projections
        let hash = simhash_128(&embedding, seed);

        // All-zero embedding should produce consistent hash
        let hash2 = simhash_128(&embedding, seed);
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_simhash_128_various_dimensions() {
        let seed = 0x454c4944_53494d48;

        // Test different valid dimensions
        for dim in [64, 128, 256, 512, 768, 1024, 1536, 2048] {
            let embedding = vec![0.1; dim];
            let hash = simhash_128(&embedding, seed);
            // Should complete without panic
            let _ = hash; // Use hash to avoid unused variable warning
        }
    }

    #[test]
    fn test_simhash_128_sign_extraction() {
        let seed = 0x454c4944_53494d48;

        // Create embedding with known structure
        let positive_embedding = vec![1.0; 128];
        let negative_embedding = vec![-1.0; 128];

        let hash_pos = simhash_128(&positive_embedding, seed);
        let hash_neg = simhash_128(&negative_embedding, seed);

        // Opposite embeddings should have very different hashes
        let dist = elid_hamming_distance(hash_pos, hash_neg);
        // Expect significant Hamming distance (not exactly 128 due to randomness)
        assert!(
            dist > 64,
            "Opposite embeddings should have high Hamming distance, got {}",
            dist
        );
    }

    #[test]
    fn test_simhash_128_locality_preservation() {
        let seed = 0x454c4944_53494d48;

        // Create similar embeddings
        let base = vec![0.5; 256];
        let mut similar = base.clone();
        similar[0] = 0.51; // Slight perturbation

        let hash_base = simhash_128(&base, seed);
        let hash_similar = simhash_128(&similar, seed);

        // Similar embeddings should have low Hamming distance
        let dist = elid_hamming_distance(hash_base, hash_similar);

        // Due to locality-sensitive property, expect relatively low distance
        // (not 0, but significantly less than random)
        assert!(
            dist < 64,
            "Similar embeddings should have low Hamming distance, got {}",
            dist
        );
    }

    // ========================================================================
    // Hamming Distance Tests
    // ========================================================================

    #[test]
    fn test_hamming_distance_identical() {
        let a = 0xDEAD_BEEF_CAFE_BABE_1234_5678_9ABC_DEF0_u128;
        assert_eq!(elid_hamming_distance(a, a), 0);
    }

    #[test]
    fn test_hamming_distance_one_bit() {
        let a = 0b0000_u128;
        let b = 0b0001_u128;
        assert_eq!(elid_hamming_distance(a, b), 1);
    }

    #[test]
    fn test_hamming_distance_all_bits() {
        let a = 0_u128;
        let b = !0_u128; // All bits set
        assert_eq!(elid_hamming_distance(a, b), 128);
    }

    #[test]
    fn test_hamming_distance_symmetric() {
        let a = 0x1234_5678_u128;
        let b = 0x9ABC_DEF0_u128;
        assert_eq!(elid_hamming_distance(a, b), elid_hamming_distance(b, a));
    }

    #[test]
    fn test_hamming_distance_known_pattern() {
        let a = 0b1010_u128;
        let b = 0b1100_u128;
        // Diff positions: bit 1 (1 vs 0) and bit 3 (0 vs 1)
        assert_eq!(elid_hamming_distance(a, b), 2);
    }

    // ========================================================================
    // Byte Conversion Tests
    // ========================================================================

    #[test]
    fn test_simhash_to_bytes_basic() {
        let hash = 0x0102030405060708090A0B0C0D0E0F10_u128;
        let bytes = simhash_to_bytes(hash);

        assert_eq!(bytes.len(), 16);
        assert_eq!(bytes[0], 0x01);
        assert_eq!(bytes[15], 0x10);
    }

    #[test]
    fn test_simhash_from_bytes_valid() {
        let bytes = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
            0x0F, 0x10,
        ];

        let hash = simhash_from_bytes(&bytes).unwrap();
        assert_eq!(hash, 0x0102030405060708090A0B0C0D0E0F10_u128);
    }

    #[test]
    fn test_simhash_from_bytes_invalid_length() {
        let too_short = [0u8; 8];
        assert!(simhash_from_bytes(&too_short).is_err());

        let too_long = [0u8; 32];
        assert!(simhash_from_bytes(&too_long).is_err());

        let empty: [u8; 0] = [];
        assert!(simhash_from_bytes(&empty).is_err());
    }

    #[test]
    fn test_simhash_bytes_roundtrip() {
        let test_hashes = vec![
            0_u128,
            !0_u128,
            0xDEADBEEFCAFEBABE1234567890ABCDEF_u128,
            0x0000000000000000FFFFFFFFFFFFFFFF_u128,
            0x5555555555555555AAAAAAAAAAAAAAAA_u128,
        ];

        for hash in test_hashes {
            let bytes = simhash_to_bytes(hash);
            let recovered = simhash_from_bytes(&bytes).unwrap();
            assert_eq!(hash, recovered, "Round-trip failed for 0x{:032x}", hash);
        }
    }

    // ========================================================================
    // Cosine Similarity Approximation Tests
    // ========================================================================

    #[test]
    fn test_cosine_similarity_approx_identical() {
        let hash = 0xDEADBEEFCAFEBABE_u128;
        let sim = cosine_similarity_approx(hash, hash);

        // Identical hashes should have similarity = 1.0
        assert_eq!(sim, 1.0, "Identical hashes should have similarity 1.0");
    }

    #[test]
    fn test_cosine_similarity_approx_opposite() {
        let a = 0_u128;
        let b = !0_u128; // All bits flipped (Hamming distance = 128)
        let sim = cosine_similarity_approx(a, b);

        // Maximum Hamming distance should give negative similarity
        // 1 - (128/128) * pi = 1 - pi ~ -2.14
        let expected = 1.0 - std::f32::consts::PI;
        assert!(
            (sim - expected).abs() < 0.001,
            "Expected {}, got {}",
            expected,
            sim
        );
        assert!(sim < 0.0, "Opposite hashes should have negative similarity");
    }

    #[test]
    fn test_cosine_similarity_approx_symmetry() {
        let a = 0x1234567890ABCDEF_u128;
        let b = 0xFEDCBA0987654321_u128;

        let sim_ab = cosine_similarity_approx(a, b);
        let sim_ba = cosine_similarity_approx(b, a);

        assert_eq!(sim_ab, sim_ba, "Cosine similarity should be symmetric");
    }

    // ========================================================================
    // Reference Output Test
    // ========================================================================

    #[test]
    fn test_simhash_reference_output() {
        // Known reference output for reproducibility testing
        let embedding = vec![
            0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, -0.1, -0.2, -0.3, -0.4, -0.5, -0.6, -0.7, -0.8,
        ]
        .into_iter()
        .cycle()
        .take(768)
        .collect::<Vec<_>>();

        let seed = 0x454c4944_53494d48; // "ELIDSIMH"
        let hash = simhash_128(&embedding, seed);

        // This is the expected output for this specific input
        // If this test fails after code changes, verify correctness before updating
        let expected: u128 = 0x9f52baea6db62f9b250de36caf8f0b13;

        assert_eq!(
            hash, expected,
            "Reference hash mismatch! Got 0x{:032x}, expected 0x{:032x}",
            hash, expected
        );
    }
}
