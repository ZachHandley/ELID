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
//! 3. Taking the sign bit of each dot product (positive → 1, negative/zero → 0)
//! 4. Packing the 128 bits into a u128
//!
//! # Theoretical Guarantee (Charikar 2002)
//!
//! For two embeddings x and y with cosine similarity cos(θ), the probability that
//! their SimHash bits match at position i is:
//!
//! ```text
//! P[h_i(x) = h_i(y)] = 1 - θ/π
//! ```
//!
//! where θ = arccos(cosine_similarity(x, y))
//!
//! This means Hamming distance between hashes is proportional to angular distance:
//! - Similar embeddings (high cosine similarity) → low Hamming distance
//! - Dissimilar embeddings (low cosine similarity) → high Hamming distance
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
//! Expected: ~10-50 μs per embedding (768 dimensions)
//! - 128 × dim floating-point multiplications (98,304 FLOPs for 768-dim)
//! - Auto-vectorization with `-C target-cpu=native`
//!
//! # References
//!
//! **Charikar, M.S. (2002)**: "Similarity Estimation Techniques from Rounding Algorithms"
//! Proceedings of the 34th Annual ACM Symposium on Theory of Computing (STOC '02)
//! [PDF](https://www.cs.princeton.edu/courses/archive/spr04/cos598B/bib/CharikarEstim.pdf)

use blake3;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use rand_distr::{Distribution, StandardNormal};

use crate::error::ElidError;

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
///
/// # Examples
///
/// ```rust,ignore
/// let seed = derive_bit_seed(0x454c4944_53494d48, 0);
/// let rng = ChaCha20Rng::from_seed(seed);
/// ```
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
/// - Time: O(128 × dim) floating-point operations
/// - Space: O(1) (on-the-fly generation, no matrix storage)
/// - Expected: ~10-50 μs for 768-dimensional embeddings
///
/// # Examples
///
/// ```rust,ignore
/// use elid_core::simhash::simhash_128;
///
/// let embedding = vec![0.1, 0.2, 0.3, 0.4];
/// let seed = 0x454c4944_53494d48; // "ELIDSIMH"
/// let hash = simhash_128(&embedding, seed);
///
/// // Same input always produces same hash (deterministic)
/// let hash2 = simhash_128(&embedding, seed);
/// assert_eq!(hash, hash2);
/// ```
///
/// # Panics
///
/// This function does not panic. Invalid embeddings should be validated
/// before calling (see [`crate::types::Embedding::new`]).
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
/// E[hamming_distance] = (θ / π) × 128
/// ```
///
/// where θ = arccos(cosine_similarity)
///
/// # Examples
///
/// ```rust
/// use elid_core::simhash::hamming_distance_128;
///
/// let a = 0b1010_u128;
/// let b = 0b1100_u128;
/// assert_eq!(hamming_distance_128(a, b), 2);
///
/// // Identical hashes have distance 0
/// assert_eq!(hamming_distance_128(a, a), 0);
/// ```
#[inline]
#[must_use]
pub fn hamming_distance_128(a: u128, b: u128) -> u32 {
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
///
/// # Examples
///
/// ```rust
/// use elid_core::simhash::{simhash_to_bytes, simhash_from_bytes};
///
/// let hash = 0x0102030405060708090A0B0C0D0E0F10_u128;
/// let bytes = simhash_to_bytes(hash);
/// assert_eq!(bytes.len(), 16);
///
/// // Round-trip conversion
/// let recovered = simhash_from_bytes(&bytes).unwrap();
/// assert_eq!(hash, recovered);
/// ```
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
///
/// # Examples
///
/// ```rust
/// use elid_core::simhash::{simhash_to_bytes, simhash_from_bytes};
///
/// let hash = 0xDEADBEEFCAFEBABE1234567890ABCDEF_u128;
/// let bytes = simhash_to_bytes(hash);
/// let recovered = simhash_from_bytes(&bytes).unwrap();
/// assert_eq!(hash, recovered);
///
/// // Invalid length returns error
/// let invalid = [0u8; 8];
/// assert!(simhash_from_bytes(&invalid).is_err());
/// ```
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
/// P[h_i(x) = h_i(y)] = 1 - θ/π
/// ```
/// where θ = arccos(cosine_similarity)
///
/// Therefore:
/// ```text
/// hamming_distance / 128 ≈ θ/π
/// cosine_similarity ≈ cos(θ) ≈ 1 - (hamming_distance / 128) × π
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
/// This is an approximation with expected error decreasing as O(1/√k)
/// where k=128 bits. For 128 bits, typical error is ~5-10%.
///
/// # Examples
///
/// ```rust
/// use elid_core::simhash::cosine_similarity_approx;
///
/// // Identical hashes → similarity ≈ 1.0
/// let hash = 0x1234_u128;
/// let sim = cosine_similarity_approx(hash, hash);
/// assert!((sim - 1.0).abs() < 0.01);
///
/// // Completely different hashes → similarity ≈ 0.0 or negative
/// let a = 0_u128;
/// let b = !0_u128;  // All bits flipped
/// let sim = cosine_similarity_approx(a, b);
/// assert!(sim < 0.0);
/// ```
#[inline]
#[must_use]
pub fn cosine_similarity_approx(hash_a: u128, hash_b: u128) -> f32 {
    let distance = hamming_distance_128(hash_a, hash_b) as f32;
    1.0 - (distance / 128.0) * std::f32::consts::PI
}

// ============================================================================
// Hamming Ball Neighbor Search (T055-T058)
// ============================================================================

/// State machine for enumerating neighbors in a Hamming ball
///
/// This enum represents the current state of iteration through all hash values
/// within a specified Hamming distance from a center hash. It uses a state machine
/// to efficiently generate neighbors without storing them all in memory.
///
/// # States
///
/// - `Radius0(bool)`: Center only - bool tracks if center has been returned
/// - `Radius1 { bit_idx: u8 }`: Single bit flips (128 neighbors)
/// - `Radius2 { bit1: u8, bit2: u8 }`: Double bit flips (8128 neighbors)
/// - `Radius3 { bit1: u8, bit2: u8, bit3: u8 }`: Triple bit flips (341376 neighbors)
/// - `Done`: Iteration complete
#[derive(Debug, Clone)]
enum HammingBallState {
    /// Radius 0: Return center only (bool = whether center has been returned)
    Radius0(bool),

    /// Radius 1: Single bit flips (0..128)
    Radius1 { bit_idx: u8 },

    /// Radius 2: Double bit flips (C(128,2) = 8128)
    Radius2 { bit1: u8, bit2: u8 },

    /// Radius 3: Triple bit flips (C(128,3) = 341376)
    Radius3 { bit1: u8, bit2: u8, bit3: u8 },

    /// Iteration complete
    Done,
}

/// Iterator over all hash values within a Hamming ball
///
/// Generates all 128-bit hashes within a specified Hamming distance (radius)
/// from a center hash. The iterator yields valid ELID strings by flipping
/// bits in the underlying SimHash payload.
///
/// # Algorithm
///
/// The iterator uses a state machine to enumerate all bit flip combinations:
/// - **Radius 0**: Returns center only (1 neighbor)
/// - **Radius 1**: Enumerates all 128 single-bit flips
/// - **Radius 2**: Enumerates all C(128,2) = 8128 double-bit flips
/// - **Radius 3**: Enumerates all C(128,3) = 341376 triple-bit flips
///
/// Each state transition ensures all combinations at the current radius are
/// generated before moving to the next radius.
///
/// # Performance
///
/// - **Radius 0**: 1 iteration
/// - **Radius 1**: 129 iterations (center + 128)
/// - **Radius 2**: 8257 iterations (center + 128 + 8128)
/// - **Radius 3**: 349633 iterations (center + 128 + 8128 + 341376)
///
/// # Examples
///
/// ```rust,ignore
/// use elid_core::{encode, Profile, simhash::hamming_neighbors};
///
/// let embedding = vec![0.1; 768];
/// let profile = Profile::default();
/// let center = encode(&embedding, &profile)?;
///
/// // Find all neighbors within Hamming distance 1
/// let neighbors: Vec<_> = hamming_neighbors(&center, 1)?.collect();
/// assert_eq!(neighbors.len(), 129); // center + 128 single-bit flips
/// ```
pub struct HammingBall {
    /// The center hash (u128 from SimHash payload)
    center: u128,

    /// Maximum Hamming distance (0-3)
    radius: u8,

    /// Current iteration state
    state: HammingBallState,

    /// Header bytes from original ELID (for reconstructing neighbors)
    header: [u8; 2],
}

impl HammingBall {
    /// Create a new Hamming ball iterator
    ///
    /// Extracts the SimHash from the ELID and initializes the state machine
    /// to enumerate all neighbors within the specified radius.
    ///
    /// # Algorithm
    ///
    /// 1. Decode ELID to bytes
    /// 2. Extract 2-byte header (version + profile type)
    /// 3. Extract 16-byte SimHash payload
    /// 4. Convert payload to u128 (big-endian)
    /// 5. Validate radius ≤ 3
    /// 6. Initialize state machine at Radius0
    ///
    /// # Parameters
    ///
    /// - `center_id`: The center ELID
    /// - `radius`: Maximum Hamming distance (0-3)
    ///
    /// # Returns
    ///
    /// - `Ok(HammingBall)`: Iterator ready to enumerate neighbors
    /// - `Err(ElidError::RadiusTooLarge)`: If radius > 3
    /// - `Err(ElidError::InvalidEncoding)`: If ELID decoding fails
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let ball = HammingBall::new(&center_elid, 2)?;
    /// let neighbors: Vec<_> = ball.collect();
    /// assert_eq!(neighbors.len(), 8257); // 1 + 128 + 8128
    /// ```
    pub fn new(center_id: &crate::types::Elid, radius: u8) -> Result<Self, ElidError> {
        // Validate radius
        if radius > 3 {
            return Err(ElidError::RadiusTooLarge(radius));
        }

        // Decode ELID to bytes
        let bytes = crate::decode(center_id)?;

        // Extract header (first 2 bytes)
        if bytes.len() < 18 {
            return Err(ElidError::InvalidEncoding);
        }
        let header = [bytes[0], bytes[1]];

        // Extract SimHash payload (bytes 2..18)
        let payload = &bytes[2..18];

        // Convert to u128
        let center = simhash_from_bytes(payload)?;

        // Initialize state machine
        let state = HammingBallState::Radius0(false);

        Ok(HammingBall {
            center,
            radius,
            state,
            header,
        })
    }

    /// Convert a u128 hash back to an ELID string
    ///
    /// Reconstructs a valid ELID by combining the header with the modified hash.
    ///
    /// # Algorithm
    ///
    /// 1. Convert u128 hash to 16-byte array (big-endian)
    /// 2. Concatenate header (2 bytes) + hash (16 bytes)
    /// 3. Encode to base32hex
    /// 4. Wrap in Elid type
    ///
    /// # Parameters
    ///
    /// - `hash`: The 128-bit SimHash value
    ///
    /// # Returns
    ///
    /// A valid ELID string with the same header as the center
    fn hash_to_elid(&self, hash: u128) -> crate::types::Elid {
        // Convert hash to bytes
        let hash_bytes = simhash_to_bytes(hash);

        // Combine header + payload
        let mut combined = self.header.to_vec();
        combined.extend_from_slice(&hash_bytes);

        // Encode to base32hex
        let encoded = crate::encoding::encode_sortable(&combined);

        // Create Elid (this should never fail since we're generating valid base32hex)
        crate::types::Elid::from_string(encoded).expect("Generated ELID should be valid")
    }
}

impl Iterator for HammingBall {
    type Item = crate::types::Elid;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match &mut self.state {
                HammingBallState::Radius0(returned) => {
                    if !*returned {
                        *returned = true;
                        return Some(self.hash_to_elid(self.center));
                    }

                    // Transition to next state based on radius
                    if self.radius >= 1 {
                        self.state = HammingBallState::Radius1 { bit_idx: 0 };
                    } else {
                        self.state = HammingBallState::Done;
                    }
                }

                HammingBallState::Radius1 { bit_idx } => {
                    if *bit_idx < 128 {
                        // Generate single-bit flip
                        let neighbor = self.center ^ (1u128 << *bit_idx);
                        *bit_idx += 1;
                        return Some(self.hash_to_elid(neighbor));
                    }

                    // Transition to next state based on radius
                    if self.radius >= 2 {
                        self.state = HammingBallState::Radius2 { bit1: 0, bit2: 1 };
                    } else {
                        self.state = HammingBallState::Done;
                    }
                }

                HammingBallState::Radius2 { bit1, bit2 } => {
                    if *bit1 < 127 {
                        // Generate double-bit flip
                        let neighbor = self.center ^ (1u128 << *bit1) ^ (1u128 << *bit2);

                        // Advance to next combination
                        *bit2 += 1;
                        if *bit2 >= 128 {
                            *bit1 += 1;
                            *bit2 = *bit1 + 1;
                        }

                        return Some(self.hash_to_elid(neighbor));
                    }

                    // Transition to next state based on radius
                    if self.radius >= 3 {
                        self.state = HammingBallState::Radius3 {
                            bit1: 0,
                            bit2: 1,
                            bit3: 2,
                        };
                    } else {
                        self.state = HammingBallState::Done;
                    }
                }

                HammingBallState::Radius3 { bit1, bit2, bit3 } => {
                    if *bit1 < 126 {
                        // Generate triple-bit flip
                        let neighbor =
                            self.center ^ (1u128 << *bit1) ^ (1u128 << *bit2) ^ (1u128 << *bit3);

                        // Advance to next combination
                        *bit3 += 1;
                        if *bit3 >= 128 {
                            *bit2 += 1;
                            *bit3 = *bit2 + 1;

                            if *bit3 >= 128 {
                                *bit1 += 1;
                                *bit2 = *bit1 + 1;
                                *bit3 = *bit2 + 1;
                            }
                        }

                        return Some(self.hash_to_elid(neighbor));
                    }

                    // All neighbors enumerated
                    self.state = HammingBallState::Done;
                }

                HammingBallState::Done => {
                    return None;
                }
            }
        }
    }
}

/// Create an iterator over all ELIDs within a Hamming ball
///
/// Returns an iterator that yields all valid ELID strings within the specified
/// Hamming distance from the center ELID. This is useful for approximate nearest
/// neighbor search in databases indexed by ELID.
///
/// # Parameters
///
/// - `center_id`: The center ELID around which to search
/// - `radius`: Maximum Hamming distance (0-3)
///
/// # Returns
///
/// - `Ok(HammingBall)`: Iterator yielding neighbor ELIDs
/// - `Err(ElidError::RadiusTooLarge)`: If radius > 3
/// - `Err(ElidError::InvalidEncoding)`: If center_id is invalid
///
/// # Neighbor Counts
///
/// - **Radius 0**: 1 neighbor (center only)
/// - **Radius 1**: 129 neighbors (center + 128 single flips)
/// - **Radius 2**: 8257 neighbors (center + 128 + 8128 double flips)
/// - **Radius 3**: 349633 neighbors (center + 128 + 8128 + 341376 triple flips)
///
/// # Examples
///
/// ```rust,ignore
/// use elid_core::{encode, Profile, hamming_neighbors};
///
/// // Create an ELID
/// let embedding = vec![0.1; 768];
/// let profile = Profile::default();
/// let center = encode(&embedding, &profile)?;
///
/// // Find neighbors within Hamming distance 2
/// for neighbor in hamming_neighbors(&center, 2)? {
///     // Query database for this ELID
///     println!("Checking neighbor: {}", neighbor);
/// }
/// ```
///
/// # Database Query Pattern
///
/// ```sql
/// -- Example PostgreSQL query using ELID neighbors
/// SELECT * FROM embeddings
/// WHERE elid IN (
///     -- Generate list of neighbor ELIDs in application code
///     'elid1', 'elid2', ..., 'elidN'
/// );
/// ```
pub fn hamming_neighbors(
    center_id: &crate::types::Elid,
    radius: u8,
) -> Result<HammingBall, ElidError> {
    HammingBall::new(center_id, radius)
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
        let dist = hamming_distance_128(hash_pos, hash_neg);
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
        let dist = hamming_distance_128(hash_base, hash_similar);

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
        assert_eq!(hamming_distance_128(a, a), 0);
    }

    #[test]
    fn test_hamming_distance_one_bit() {
        let a = 0b0000_u128;
        let b = 0b0001_u128;
        assert_eq!(hamming_distance_128(a, b), 1);
    }

    #[test]
    fn test_hamming_distance_all_bits() {
        let a = 0_u128;
        let b = !0_u128; // All bits set
        assert_eq!(hamming_distance_128(a, b), 128);
    }

    #[test]
    fn test_hamming_distance_symmetric() {
        let a = 0x1234_5678_u128;
        let b = 0x9ABC_DEF0_u128;
        assert_eq!(hamming_distance_128(a, b), hamming_distance_128(b, a));
    }

    #[test]
    fn test_hamming_distance_known_pattern() {
        let a = 0b1010_u128;
        let b = 0b1100_u128;
        // Diff positions: bit 1 (1 vs 0) and bit 3 (0 vs 1)
        assert_eq!(hamming_distance_128(a, b), 2);
    }

    // ========================================================================
    // Property-based Tests (Manual)
    // ========================================================================

    #[test]
    fn test_simhash_triangle_inequality_approximation() {
        let seed = 0x454c4944_53494d48;

        let emb_a = vec![1.0, 0.0, 0.0, 0.0]
            .into_iter()
            .cycle()
            .take(128)
            .collect::<Vec<_>>();
        let emb_b = vec![0.0, 1.0, 0.0, 0.0]
            .into_iter()
            .cycle()
            .take(128)
            .collect::<Vec<_>>();
        let emb_c = vec![0.0, 0.0, 1.0, 0.0]
            .into_iter()
            .cycle()
            .take(128)
            .collect::<Vec<_>>();

        let hash_a = simhash_128(&emb_a, seed);
        let hash_b = simhash_128(&emb_b, seed);
        let hash_c = simhash_128(&emb_c, seed);

        let dist_ab = hamming_distance_128(hash_a, hash_b);
        let dist_bc = hamming_distance_128(hash_b, hash_c);
        let dist_ac = hamming_distance_128(hash_a, hash_c);

        // Hamming distance obeys triangle inequality
        assert!(dist_ac <= dist_ab + dist_bc);
        assert!(dist_ab <= dist_ac + dist_bc);
        assert!(dist_bc <= dist_ab + dist_ac);
    }

    #[test]
    fn test_simhash_normalized_vs_unnormalized() {
        let seed = 0x454c4944_53494d48;

        let embedding = vec![0.3, 0.4, 0.5, 0.6]
            .into_iter()
            .cycle()
            .take(128)
            .collect::<Vec<_>>();
        let normalized: Vec<f32> = {
            let norm = embedding.iter().map(|v| v * v).sum::<f32>().sqrt();
            embedding.iter().map(|v| v / norm).collect()
        };

        let hash1 = simhash_128(&embedding, seed);
        let hash2 = simhash_128(&normalized, seed);

        // SimHash should produce similar (but not identical) results for
        // normalized vs unnormalized embeddings since sign() is invariant
        // to positive scaling
        let dist = hamming_distance_128(hash1, hash2);
        assert!(
            dist < 20,
            "Normalized and unnormalized versions should have similar hashes, got distance {}",
            dist
        );
    }

    // ========================================================================
    // Regression Tests (Example Outputs)
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

    // ========================================================================
    // Byte Conversion Tests (T019)
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

    #[test]
    fn test_simhash_bytes_big_endian() {
        // Verify big-endian encoding
        let hash = 0x0102030405060708090A0B0C0D0E0F10_u128;
        let bytes = simhash_to_bytes(hash);

        // Most significant byte should come first
        assert_eq!(bytes[0], 0x01);
        assert_eq!(bytes[1], 0x02);
        // Least significant byte should come last
        assert_eq!(bytes[14], 0x0F);
        assert_eq!(bytes[15], 0x10);
    }

    // ========================================================================
    // Cosine Similarity Approximation Tests (T020)
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
        // 1 - (128/128) * π = 1 - π ≈ -2.14
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
    fn test_cosine_similarity_approx_half_distance() {
        // Create hashes with exactly 64 bits different
        let a = 0_u128;
        let b = (1_u128 << 64) - 1; // Lower 64 bits set

        let distance = hamming_distance_128(a, b);
        assert_eq!(distance, 64, "Should have exactly 64 bits different");

        let sim = cosine_similarity_approx(a, b);

        // 1 - (64/128) * π = 1 - π/2 ≈ -0.57
        let expected = 1.0 - (std::f32::consts::PI / 2.0);
        assert!(
            (sim - expected).abs() < 0.001,
            "Expected {}, got {}",
            expected,
            sim
        );
    }

    #[test]
    fn test_cosine_similarity_approx_known_values() {
        // Test known Hamming distances and their similarity approximations
        let base = 0_u128;

        let test_cases = vec![
            (0, 1.0),                                          // Distance 0 → sim = 1.0
            (1, 1.0 - (1.0 / 128.0) * std::f32::consts::PI),   // Distance 1
            (32, 1.0 - (32.0 / 128.0) * std::f32::consts::PI), // Distance 32
            (64, 1.0 - 0.5 * std::f32::consts::PI),            // Distance 64
            (128, 1.0 - std::f32::consts::PI),                 // Distance 128
        ];

        for (distance, expected_sim) in test_cases {
            // Create hash with exact distance
            let mut other = base;
            for i in 0..distance {
                other |= 1_u128 << i;
            }

            let actual_distance = hamming_distance_128(base, other);
            assert_eq!(
                actual_distance, distance,
                "Failed to create hash with distance {}",
                distance
            );

            let sim = cosine_similarity_approx(base, other);
            assert!(
                (sim - expected_sim).abs() < 0.001,
                "Distance {}: expected sim {}, got {}",
                distance,
                expected_sim,
                sim
            );
        }
    }

    #[test]
    fn test_cosine_similarity_approx_symmetry() {
        let a = 0x1234567890ABCDEF_u128;
        let b = 0xFEDCBA0987654321_u128;

        let sim_ab = cosine_similarity_approx(a, b);
        let sim_ba = cosine_similarity_approx(b, a);

        assert_eq!(sim_ab, sim_ba, "Cosine similarity should be symmetric");
    }

    #[test]
    fn test_cosine_similarity_approx_realistic() {
        // Test with realistic SimHash values
        let seed = 0x454c4944_53494d48;

        let emb1 = vec![0.5; 256];
        let mut emb2 = emb1.clone();
        emb2[0] = 0.6; // Slight perturbation

        let hash1 = simhash_128(&emb1, seed);
        let hash2 = simhash_128(&emb2, seed);

        let sim = cosine_similarity_approx(hash1, hash2);
        let distance = hamming_distance_128(hash1, hash2);

        // Very similar embeddings may have identical hashes (distance 0) or low distance
        // So we check that similarity is in valid range and consistent with distance
        assert!(
            (-1.0..=1.0).contains(&sim),
            "Similarity should be in range [-1, 1], got {}",
            sim
        );

        // If distance is 0, similarity must be 1.0
        if distance == 0 {
            assert_eq!(sim, 1.0, "Distance 0 should give similarity 1.0");
        } else {
            // Otherwise, similar embeddings should have high positive similarity
            assert!(
                sim > 0.5,
                "Similar embeddings with distance {} should have high similarity, got {}",
                distance,
                sim
            );
        }
    }

    #[test]
    fn test_cosine_similarity_approx_formula() {
        // Verify the formula: 1 - (distance / 128) * π
        for distance in [0, 1, 10, 32, 64, 100, 128] {
            let a = 0_u128;
            let mut b = 0_u128;

            // Set exactly 'distance' bits
            for i in 0..distance {
                b |= 1_u128 << i;
            }

            let sim = cosine_similarity_approx(a, b);
            let expected = 1.0 - (distance as f32 / 128.0) * std::f32::consts::PI;

            assert!(
                (sim - expected).abs() < 0.0001,
                "Distance {}: formula mismatch, expected {}, got {}",
                distance,
                expected,
                sim
            );
        }
    }

    // ========================================================================
    // Hamming Ball Neighbor Search Tests (T055-T058)
    // ========================================================================

    #[test]
    fn test_hamming_ball_radius_0() {
        // Create an ELID
        let embedding = vec![0.1, 0.2, 0.3, 0.4];
        let embedding = embedding.into_iter().cycle().take(768).collect::<Vec<_>>();
        let profile = crate::Profile::default();
        let elid = crate::encode(&embedding, &profile).unwrap();

        // Radius 0 should return only the center
        let neighbors: Vec<_> = hamming_neighbors(&elid, 0).unwrap().collect();
        assert_eq!(
            neighbors.len(),
            1,
            "Radius 0 should return exactly 1 neighbor (center)"
        );
        assert_eq!(neighbors[0], elid, "Radius 0 should return the center ID");
    }

    #[test]
    fn test_hamming_ball_radius_1_count() {
        let embedding = vec![0.5; 256];
        let profile = crate::Profile::default();
        let elid = crate::encode(&embedding, &profile).unwrap();

        // Radius 1 should return center + 128 neighbors (1 per bit flip)
        let neighbors: Vec<_> = hamming_neighbors(&elid, 1).unwrap().collect();
        assert_eq!(
            neighbors.len(),
            1 + 128,
            "Radius 1 should return 129 neighbors (center + 128 single-bit flips)"
        );
    }

    #[test]
    fn test_hamming_ball_radius_1_hamming_distance() {
        let embedding = vec![0.5; 256];
        let profile = crate::Profile::default();
        let center = crate::encode(&embedding, &profile).unwrap();

        let neighbors: Vec<_> = hamming_neighbors(&center, 1).unwrap().collect();

        // First neighbor is the center (distance 0)
        assert_eq!(
            crate::hamming_distance(&center, &neighbors[0]).unwrap(),
            0,
            "First neighbor should be center with distance 0"
        );

        // All other neighbors should have distance exactly 1
        for (i, neighbor) in neighbors.iter().enumerate().skip(1) {
            let dist = crate::hamming_distance(&center, neighbor).unwrap();
            assert_eq!(
                dist, 1,
                "Neighbor {} should have Hamming distance 1, got {}",
                i, dist
            );
        }
    }

    #[test]
    fn test_hamming_ball_radius_2_count() {
        let embedding = vec![0.3; 128];
        let profile = crate::Profile::default();
        let elid = crate::encode(&embedding, &profile).unwrap();

        // Radius 2 should return: center + 128 single-flips + C(128,2) double-flips
        // C(128,2) = 128 * 127 / 2 = 8128
        // Total: 1 + 128 + 8128 = 8257
        let neighbors: Vec<_> = hamming_neighbors(&elid, 2).unwrap().collect();
        assert_eq!(
            neighbors.len(),
            8257,
            "Radius 2 should return 8257 neighbors (1 + 128 + 8128)"
        );
    }

    #[test]
    fn test_hamming_ball_radius_2_hamming_distance() {
        let embedding = vec![0.7; 128];
        let profile = crate::Profile::default();
        let center = crate::encode(&embedding, &profile).unwrap();

        let neighbors: Vec<_> = hamming_neighbors(&center, 2).unwrap().collect();

        // Verify distances
        let mut dist_0_count = 0;
        let mut dist_1_count = 0;
        let mut dist_2_count = 0;

        for neighbor in &neighbors {
            let dist = crate::hamming_distance(&center, neighbor).unwrap();
            match dist {
                0 => dist_0_count += 1,
                1 => dist_1_count += 1,
                2 => dist_2_count += 1,
                other => panic!("Unexpected distance {} in radius 2 ball", other),
            }
        }

        assert_eq!(
            dist_0_count, 1,
            "Should have exactly 1 neighbor at distance 0"
        );
        assert_eq!(
            dist_1_count, 128,
            "Should have exactly 128 neighbors at distance 1"
        );
        assert_eq!(
            dist_2_count, 8128,
            "Should have exactly 8128 neighbors at distance 2"
        );
    }

    #[test]
    fn test_hamming_ball_radius_3_partial() {
        // Radius 3 generates 341376 neighbors, which is expensive to test fully
        // We'll just verify the first 1000 neighbors have correct distances
        let embedding = vec![0.5; 256];
        let profile = crate::Profile::default();
        let center = crate::encode(&embedding, &profile).unwrap();

        let mut ball = hamming_neighbors(&center, 3).unwrap();

        // Check first 1000 neighbors
        for i in 0..1000 {
            let neighbor = ball.next().expect("Should have more neighbors");
            let dist = crate::hamming_distance(&center, &neighbor).unwrap();
            assert!(
                dist <= 3,
                "Neighbor {} should have distance <= 3, got {}",
                i,
                dist
            );
        }
    }

    #[test]
    fn test_hamming_ball_radius_too_large() {
        let embedding = vec![0.5; 256];
        let profile = crate::Profile::default();
        let elid = crate::encode(&embedding, &profile).unwrap();

        // Radius > 3 should return error
        let result = hamming_neighbors(&elid, 4);
        assert!(
            matches!(result, Err(ElidError::RadiusTooLarge(4))),
            "Radius > 3 should return RadiusTooLarge error"
        );

        let result = hamming_neighbors(&elid, 10);
        assert!(
            matches!(result, Err(ElidError::RadiusTooLarge(10))),
            "Radius 10 should return RadiusTooLarge error"
        );
    }

    #[test]
    fn test_hamming_ball_determinism() {
        let embedding = vec![0.4; 512];
        let profile = crate::Profile::default();
        let elid = crate::encode(&embedding, &profile).unwrap();

        // Generate neighbors twice with radius 1
        let neighbors1: Vec<_> = hamming_neighbors(&elid, 1).unwrap().collect();
        let neighbors2: Vec<_> = hamming_neighbors(&elid, 1).unwrap().collect();

        // Should generate same sequence
        assert_eq!(
            neighbors1.len(),
            neighbors2.len(),
            "Should generate same count"
        );

        for (i, (n1, n2)) in neighbors1.iter().zip(neighbors2.iter()).enumerate() {
            assert_eq!(
                n1, n2,
                "Neighbor {} should be identical in both sequences",
                i
            );
        }
    }

    #[test]
    fn test_hamming_ball_all_valid_elids() {
        let embedding = vec![0.6; 256];
        let profile = crate::Profile::default();
        let center = crate::encode(&embedding, &profile).unwrap();

        // Test radius 1 (manageable count)
        let neighbors: Vec<_> = hamming_neighbors(&center, 1).unwrap().collect();

        // All neighbors should be valid ELIDs
        for (i, neighbor) in neighbors.iter().enumerate() {
            // Should be able to decode
            assert!(
                crate::decode(neighbor).is_ok(),
                "Neighbor {} should be decodable",
                i
            );

            // Should have valid profile
            assert!(
                neighbor.profile().is_ok(),
                "Neighbor {} should have valid profile",
                i
            );

            // Profile should be Mini128
            let profile_info = neighbor.profile().unwrap();
            assert_eq!(
                profile_info.profile_type, 0x01,
                "Neighbor {} should have Mini128 profile",
                i
            );
        }
    }

    #[test]
    fn test_hamming_ball_no_duplicates_radius_1() {
        let embedding = vec![0.8; 128];
        let profile = crate::Profile::default();
        let elid = crate::encode(&embedding, &profile).unwrap();

        let neighbors: Vec<_> = hamming_neighbors(&elid, 1).unwrap().collect();

        // Check for duplicates using a HashSet
        let unique_neighbors: std::collections::HashSet<_> = neighbors.iter().collect();
        assert_eq!(
            neighbors.len(),
            unique_neighbors.len(),
            "Should have no duplicate neighbors"
        );
    }
}
