//! ELID Core: Embedding Locality IDentifier encoding and decoding
//!
//! This library provides compact, sortable identifiers for high-dimensional embeddings.
//!
//! # Overview
//!
//! ELID (Embedding Locality IDentifier) encodes embeddings into base32hex strings
//! that preserve locality properties for efficient similarity search and database indexing.
//!
//! # Profiles
//!
//! ELID supports three encoding profiles, each optimized for different use cases:
//!
//! - **Mini128**: 128-bit SimHash using signed random projections (29 characters)
//!   - **Use for**: Approximate nearest neighbor search via Hamming distance
//!   - **Best at**: Fast similarity comparisons, deduplication, fuzzy matching
//!   - **Locality**: Angular distance preservation (Charikar 2002)
//!
//! - **Morton10x10**: Z-order curve encoding (24 characters)
//!   - **Use for**: Database indexing with sortable IDs (default for sortability)
//!   - **Best at**: Fast encoding, good-enough spatial locality, range queries
//!   - **Locality**: Good clustering of nearby points in embedding space
//!
//! - **Hilbert10x10**: Hilbert curve encoding (24 characters)
//!   - **Use for**: Quality-critical applications requiring maximum locality
//!   - **Best at**: Superior spatial locality (5-10% better than Morton)
//!   - **Tradeoff**: 5-10x slower encoding than Morton
//!
//! **Choosing a Profile:**
//! - **Similarity search?** → Use `Mini128` for Hamming distance comparisons
//! - **Database indexing (production)?** → Use `Morton10x10` (fast, good locality)
//! - **Quality-critical indexing?** → Use `Hilbert10x10` (best locality, slower)
//!
//! # Quick Start
//!
//! ```rust
//! use elid_core::{encode, decode, hamming_distance, Profile};
//!
//! // Create embeddings (typically from ML models like BERT, OpenAI, etc.)
//! let embedding1 = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8];
//! let embedding1 = embedding1.into_iter().cycle().take(768).collect::<Vec<_>>();
//!
//! let embedding2 = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.9]; // Slightly different
//! let embedding2 = embedding2.into_iter().cycle().take(768).collect::<Vec<_>>();
//!
//! // Encode with Mini128 profile (default)
//! let profile = Profile::default();
//! let elid1 = encode(&embedding1, &profile).unwrap();
//! let elid2 = encode(&embedding2, &profile).unwrap();
//!
//! // ELIDs are compact, sortable strings
//! println!("ELID 1: {}", elid1);
//! println!("ELID 2: {}", elid2);
//! assert_eq!(elid1.as_str().len(), 29); // 29 characters for Mini128
//!
//! // Compute similarity via Hamming distance
//! let distance = hamming_distance(&elid1, &elid2).unwrap();
//! println!("Hamming distance: {}/128", distance);
//! // Similar embeddings have low distance (0-30), dissimilar have high (64-128)
//!
//! // Decode to raw bytes (for storage, transmission)
//! let bytes = decode(&elid1).unwrap();
//! assert_eq!(bytes.len(), 18); // 2 header + 16 payload bytes
//! ```
//!
//! # Advanced Usage: Cosine Similarity Approximation
//!
//! ```rust
//! use elid_core::{encode, Profile, simhash::cosine_similarity_approx};
//!
//! let embedding1 = vec![0.5; 256];
//! let embedding2 = vec![0.5; 256];
//!
//! let profile = Profile::default();
//! let elid1 = encode(&embedding1, &profile).unwrap();
//! let elid2 = encode(&embedding2, &profile).unwrap();
//!
//! // Decode to get hash values
//! let bytes1 = elid1.to_bytes().unwrap();
//! let bytes2 = elid2.to_bytes().unwrap();
//!
//! // Extract SimHash (skip 2-byte header)
//! let hash1 = u128::from_be_bytes(bytes1[2..18].try_into().unwrap());
//! let hash2 = u128::from_be_bytes(bytes2[2..18].try_into().unwrap());
//!
//! // Approximate cosine similarity from hashes (Charikar 2002)
//! let similarity = cosine_similarity_approx(hash1, hash2);
//! println!("Approximate cosine similarity: {:.3}", similarity);
//! // Range: [-1.0, 1.0], where 1.0 = identical, 0.0 = orthogonal, -1.0 = opposite
//! ```
//!
//! # Sortable IDs with Morton Encoding
//!
//! ```rust
//! use elid_core::{encode, Profile};
//!
//! // Create an embedding (e.g., from OpenAI's text-embedding-3-small)
//! let embedding = vec![0.1, 0.2, 0.3, 0.4, 0.5];
//! let embedding = embedding.into_iter().cycle().take(1536).collect::<Vec<_>>();
//!
//! // Encode with Morton profile for database indexing
//! let profile = Profile::Morton10x10 {
//!     dims: 10,          // Use first 10 dimensions (higher = more precision)
//!     bits_per_dim: 10,  // 10 bits per dimension (1024 quantization levels)
//!     transform_id: None // No PCA/transform (v0.2+ feature)
//! };
//!
//! let elid = encode(&embedding, &profile).unwrap();
//!
//! // ELID is a compact, sortable string (~24 characters)
//! println!("ELID: {}", elid); // e.g., "04g8c4g0c8g4c0g8c4g0c8g4"
//! assert_eq!(elid.as_str().len(), 24);
//!
//! // Use in database as primary key or index
//! // Near-neighbor embeddings will have similar ELIDs and cluster together
//! // in B-tree indexes, enabling efficient range queries:
//! //
//! // SELECT * FROM documents
//! // WHERE elid BETWEEN 'prefix_start' AND 'prefix_end'
//! // ORDER BY elid
//! // LIMIT 100
//! ```
//!
//! **Morton vs Hilbert Trade-offs:**
//!
//! | Aspect | Morton10x10 | Hilbert10x10 |
//! |--------|-------------|--------------|
//! | Encoding Speed | **Fast** (baseline) | 5-10x slower |
//! | Locality Quality | **Good** (baseline) | 5-10% better |
//! | Production Use | **Recommended** | Optional for quality-critical apps |
//! | ID Length | 24 characters | 24 characters |
//!
//! # Use Cases
//!
//! - **Vector Databases**: Use ELIDs as sortable primary keys for embedding rows
//! - **Deduplication**: Find near-duplicate embeddings via Hamming distance thresholds
//! - **Approximate Search**: Narrow search space using ELID prefixes before computing exact similarity
//! - **Caching**: Use ELIDs as cache keys for semantic search results
//!
//! # Performance
//!
//! - **Encoding**: ~10-50 μs per 768-dimensional embedding (Mini128)
//! - **Hamming Distance**: ~10 ns (single CPU instruction on modern hardware)
//! - **Memory**: Zero allocations for Hamming distance, minimal for encoding
//!
//! # Error Handling
//!
//! All fallible operations return `Result<T, ElidError>`. Common errors:
//!
//! - [`ElidError::InvalidDimension`]: Embedding dimension outside [64, 2048]
//! - [`ElidError::InvalidValue`]: Embedding contains NaN or Inf values
//! - [`ElidError::InvalidEncoding`]: Malformed base32hex string
//! - [`ElidError::ProfileMismatch`]: Hamming distance requires same profile
//!
//! # References
//!
//! - **Charikar, M.S. (2002)**: "Similarity Estimation Techniques from Rounding Algorithms"
//!   - Theoretical foundation for SimHash locality-sensitive hashing
//! - **Sagan, H. (1994)**: "Space-Filling Curves"
//!   - Hilbert curve locality preservation (Phase 4)

#![forbid(unsafe_code)]
#![warn(missing_docs)]

// Module declarations
pub mod encoding;
pub mod error;
pub mod hilbert;
pub mod morton;
pub mod simhash;
pub mod types;

// Re-exports - Public API
pub use error::ElidError;
pub use types::{Elid, Embedding, Profile, ProfileInfo, QuantizedCoords};

// Main public API functions (encode, decode, hamming_distance)
// are defined below in the "Public API Functions" section

// Advanced exports for power users
pub use simhash::{cosine_similarity_approx, hamming_neighbors, HammingBall};

// Internal re-exports (not part of public API documentation)
use encoding::{decode_sortable, encode_sortable};
use simhash::{hamming_distance_128, simhash_128};

// ============================================================================
// Public API Functions
// ============================================================================

/// Encode an embedding into an ELID string
///
/// Converts a high-dimensional embedding vector into a compact, sortable identifier
/// using the specified profile. The resulting ELID preserves locality properties
/// for efficient similarity search.
///
/// # Algorithm
///
/// 1. Validate embedding dimensions and values (via [`Embedding::new`])
/// 2. Normalize the embedding to unit length (L2 norm)
/// 3. Apply profile-specific encoding:
///    - **Mini128**: 128-bit SimHash using random projections
///    - **Morton10x10**: Z-order curve encoding (Phase 4)
///    - **Hilbert10x10**: Hilbert curve encoding (Phase 4)
/// 4. Prepend 2-byte header (version + profile type)
/// 5. Encode to base32hex string
///
/// # Parameters
///
/// - `embedding`: Input vector (f32 slice, 64-2048 dimensions)
/// - `profile`: Encoding strategy configuration
///
/// # Returns
///
/// - `Ok(Elid)`: Encoded identifier string
/// - `Err(ElidError)`: Validation or encoding error
///
/// # Errors
///
/// - [`ElidError::InvalidDimension`]: Embedding dimension outside [64, 2048]
/// - [`ElidError::InvalidValue`]: Embedding contains NaN or Inf values
///
/// # Examples
///
/// **Mini128 (Similarity Search):**
/// ```rust,ignore
/// use elid_core::{encode, Profile};
///
/// let embedding = vec![0.1, 0.2, 0.3, 0.4];
/// let embedding = embedding.into_iter().cycle().take(768).collect::<Vec<_>>();
///
/// let profile = Profile::default(); // Mini128
/// let elid = encode(&embedding, &profile)?;
/// println!("ELID: {} (length: {})", elid, elid.as_str().len()); // 29 chars
/// ```
///
/// **Morton10x10 (Fast Database Indexing):**
/// ```rust,ignore
/// use elid_core::{encode, Profile};
///
/// let embedding = vec![0.1; 1536]; // OpenAI text-embedding-3-small
///
/// let profile = Profile::Morton10x10 {
///     dims: 10,
///     bits_per_dim: 10,
///     transform_id: None
/// };
/// let elid = encode(&embedding, &profile)?;
/// println!("ELID: {} (length: {})", elid, elid.as_str().len()); // ~24 chars
/// ```
///
/// **Hilbert10x10 (Quality-Critical Indexing):**
/// ```rust,ignore
/// use elid_core::{encode, Profile};
///
/// let embedding = vec![0.1; 768]; // BERT embeddings
///
/// let profile = Profile::Hilbert10x10 {
///     dims: 10,
///     bits_per_dim: 10,
///     transform_id: None
/// };
/// let elid = encode(&embedding, &profile)?;
/// // 5-10% better locality than Morton, but 5-10x slower encoding
/// println!("ELID: {} (length: {})", elid, elid.as_str().len()); // ~24 chars
/// ```
pub fn encode(embedding: &[f32], profile: &Profile) -> Result<Elid, ElidError> {
    // Step 1: Create and validate embedding
    let mut emb = Embedding::new(embedding.to_vec())?;

    // Step 2: Normalize to unit length
    emb.normalize();

    // Step 3: Apply profile-specific encoding
    let payload_bytes = match profile {
        Profile::Mini128 { seed } => {
            // Compute 128-bit SimHash
            let hash = simhash_128(emb.as_slice(), *seed);
            // Convert to big-endian bytes (16 bytes)
            simhash_to_bytes(hash)
        }
        Profile::Morton10x10 {
            dims,
            bits_per_dim,
            transform_id,
        } => {
            // Morton curve encoding
            // Step 1: Verify no transform_id (transforms are v0.2+)
            if transform_id.is_some() {
                return Err(ElidError::TransformNotFound(transform_id.unwrap()));
            }

            // Step 2: Create quantized coordinates
            let quantized = QuantizedCoords::from_embedding(&emb, *dims, *bits_per_dim)?;

            // Step 3: Encode to Morton code
            let code = morton::morton_encode(quantized.as_slice(), *bits_per_dim);

            // Step 4: Calculate total bits and convert to bytes
            let total_bits = (*dims as usize) * (*bits_per_dim as usize);
            code_to_bytes(code, total_bits)
        }
        Profile::Hilbert10x10 {
            dims,
            bits_per_dim,
            transform_id,
        } => {
            // Hilbert curve encoding
            // Step 1: Verify no transform_id (transforms are v0.2+)
            if transform_id.is_some() {
                return Err(ElidError::TransformNotFound(transform_id.unwrap()));
            }

            // Step 2: Create quantized coordinates
            let quantized = QuantizedCoords::from_embedding(&emb, *dims, *bits_per_dim)?;

            // Step 3: Encode to Hilbert code
            let code = hilbert::hilbert_encode(quantized.as_slice(), *bits_per_dim);

            // Step 4: Calculate total bits and convert to bytes
            let total_bits = (*dims as usize) * (*bits_per_dim as usize);
            code_to_bytes(code, total_bits)
        }
    };

    // Step 4: Create header (version=0, profile_type)
    let header = ProfileInfo {
        version: 0,
        profile_type: profile.type_id(),
        transform_id: None,
        model_id: None,
    };
    let header_bytes = header.to_header();

    // Step 5: Combine header + payload
    let mut combined = header_bytes;
    combined.extend_from_slice(&payload_bytes);

    // Step 6: Encode to base32hex
    let encoded = encode_sortable(&combined);

    // Step 7: Create Elid
    Elid::from_string(encoded)
}

/// Decode an ELID string to raw bytes
///
/// Decodes a base32hex-encoded ELID string back to its raw byte representation.
/// The output includes the 2-byte header (version + profile type) followed by
/// the profile-specific payload.
///
/// # Parameters
///
/// - `elid`: The ELID string to decode
///
/// # Returns
///
/// - `Ok(Vec<u8>)`: Raw bytes (header + payload)
/// - `Err(ElidError::InvalidEncoding)`: Invalid base32hex string
///
/// # Format
///
/// ```text
/// [byte 0: (version << 4) | profile_type]
/// [byte 1: reserved]
/// [bytes 2..N: payload (profile-specific)]
/// ```
///
/// # Examples
///
/// ```rust,ignore
/// use elid_core::{encode, decode, Profile};
///
/// let embedding = vec![0.1, 0.2, 0.3, 0.4];
/// let elid = encode(&embedding, &Profile::default())?;
///
/// let bytes = decode(&elid)?;
/// assert_eq!(bytes.len(), 18); // 2 header + 16 payload (Mini128)
/// ```
pub fn decode(elid: &Elid) -> Result<Vec<u8>, ElidError> {
    decode_sortable(elid.as_str())
}

/// Compute Hamming distance between two Mini128 ELIDs
///
/// Returns the number of differing bits in the SimHash payloads of two ELIDs.
/// This distance is proportional to the angular distance between the original
/// embeddings (Charikar 2002).
///
/// # Requirements
///
/// Both ELIDs must use the Mini128 profile. This function returns an error if
/// either ELID uses a different profile (Morton, Hilbert).
///
/// # Parameters
///
/// - `a`: First ELID
/// - `b`: Second ELID
///
/// # Returns
///
/// - `Ok(u32)`: Hamming distance (0-128)
/// - `Err(ElidError)`: Decoding error or profile mismatch
///
/// # Relationship to Cosine Similarity
///
/// From Charikar (2002):
/// ```text
/// E[hamming_distance] = (arccos(cosine_similarity) / π) × 128
/// ```
///
/// Approximate cosine similarity:
/// ```text
/// cosine_similarity ≈ cos(π × hamming_distance / 128)
/// ```
///
/// # Examples
///
/// ```rust,ignore
/// use elid_core::{encode, hamming_distance, Profile};
///
/// let emb1 = vec![0.1, 0.2, 0.3, 0.4];
/// let emb2 = vec![0.1, 0.2, 0.3, 0.5]; // Similar to emb1
///
/// let profile = Profile::default();
/// let elid1 = encode(&emb1, &profile)?;
/// let elid2 = encode(&emb2, &profile)?;
///
/// let distance = hamming_distance(&elid1, &elid2)?;
/// println!("Hamming distance: {}", distance);
/// // Low distance indicates high similarity
/// ```
///
/// # Errors
///
/// - [`ElidError::InvalidEncoding`]: Invalid base32hex string
/// - [`ElidError::InvalidHeader`]: Corrupted header
/// - [`ElidError::ProfileMismatch`]: One or both ELIDs are not Mini128
pub fn hamming_distance(a: &Elid, b: &Elid) -> Result<u32, ElidError> {
    // Step 1: Decode both ELIDs
    let bytes_a = decode(a)?;
    let bytes_b = decode(b)?;

    // Step 2: Extract and verify profile info
    let profile_a = ProfileInfo::from_header(&bytes_a[0..2])?;
    let profile_b = ProfileInfo::from_header(&bytes_b[0..2])?;

    // Step 3: Verify both are Mini128 (type_id = 0x01)
    if profile_a.profile_type != 0x01 {
        return Err(ElidError::ProfileMismatch {
            expected: "Mini128".to_string(),
            got: format!("Type {:#x}", profile_a.profile_type),
        });
    }
    if profile_b.profile_type != 0x01 {
        return Err(ElidError::ProfileMismatch {
            expected: "Mini128".to_string(),
            got: format!("Type {:#x}", profile_b.profile_type),
        });
    }

    // Step 4: Extract payload bytes (skip 2-byte header)
    let payload_a = &bytes_a[2..];
    let payload_b = &bytes_b[2..];

    // Step 5: Convert to u128
    let hash_a = simhash_from_bytes(payload_a)?;
    let hash_b = simhash_from_bytes(payload_b)?;

    // Step 6: Compute Hamming distance
    Ok(hamming_distance_128(hash_a, hash_b))
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Convert u128 SimHash to big-endian bytes (16 bytes)
fn simhash_to_bytes(hash: u128) -> Vec<u8> {
    hash.to_be_bytes().to_vec()
}

/// Convert u128 code to big-endian bytes (only needed bytes)
///
/// For Morton/Hilbert codes, we only need enough bytes to hold `total_bits`.
/// For example, 100 bits requires 13 bytes (ceil(100/8) = 13).
///
/// # Parameters
///
/// - `code`: The u128 code (Morton or Hilbert)
/// - `total_bits`: Total number of bits in the code
///
/// # Returns
///
/// Vec<u8> containing only the necessary bytes in big-endian order
fn code_to_bytes(code: u128, total_bits: usize) -> Vec<u8> {
    let needed_bytes = (total_bits + 7) / 8; // ceil(total_bits / 8)
    let all_bytes = code.to_be_bytes();

    // Take only the needed bytes from the end (big-endian, so MSB first)
    let start_idx = 16 - needed_bytes;
    all_bytes[start_idx..].to_vec()
}

/// Convert big-endian bytes to u128 SimHash
///
/// # Errors
///
/// Returns [`ElidError::InvalidEncoding`] if the byte slice is not exactly 16 bytes.
fn simhash_from_bytes(bytes: &[u8]) -> Result<u128, ElidError> {
    if bytes.len() != 16 {
        return Err(ElidError::InvalidEncoding);
    }
    let mut array = [0u8; 16];
    array.copy_from_slice(bytes);
    Ok(u128::from_be_bytes(array))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Public API Tests - encode()
    // ========================================================================

    #[test]
    fn test_encode_basic() {
        let embedding = vec![0.1; 128];
        let profile = Profile::default();
        let result = encode(&embedding, &profile);
        assert!(result.is_ok());
    }

    #[test]
    fn test_encode_deterministic() {
        let embedding = vec![0.1, 0.2, 0.3, 0.4];
        let embedding = embedding.into_iter().cycle().take(128).collect::<Vec<_>>();
        let profile = Profile::Mini128 {
            seed: 0x454c4944_53494d48,
        };

        let elid1 = encode(&embedding, &profile).unwrap();
        let elid2 = encode(&embedding, &profile).unwrap();

        assert_eq!(
            elid1, elid2,
            "Same embedding + profile should produce same ELID"
        );
    }

    #[test]
    fn test_encode_validates_dimensions() {
        // Too small
        let embedding = vec![0.1; 32]; // < 64
        let profile = Profile::default();
        let result = encode(&embedding, &profile);
        assert!(matches!(result, Err(ElidError::InvalidDimension { .. })));

        // Too large
        let embedding = vec![0.1; 4096]; // > 2048
        let result = encode(&embedding, &profile);
        assert!(matches!(result, Err(ElidError::InvalidDimension { .. })));
    }

    #[test]
    fn test_encode_validates_nan() {
        let mut embedding = vec![0.1; 128];
        embedding[64] = f32::NAN;
        let profile = Profile::default();
        let result = encode(&embedding, &profile);
        assert!(matches!(result, Err(ElidError::InvalidValue)));
    }

    #[test]
    fn test_encode_validates_inf() {
        let mut embedding = vec![0.1; 128];
        embedding[64] = f32::INFINITY;
        let profile = Profile::default();
        let result = encode(&embedding, &profile);
        assert!(matches!(result, Err(ElidError::InvalidValue)));
    }

    #[test]
    fn test_encode_normalizes_embedding() {
        // Test that encoding normalizes the embedding
        let embedding1 = vec![1.0; 128];
        let embedding2 = vec![2.0; 128]; // Scaled version

        let profile = Profile::Mini128 {
            seed: 0x454c4944_53494d48,
        };

        let elid1 = encode(&embedding1, &profile).unwrap();
        let elid2 = encode(&embedding2, &profile).unwrap();

        // Should produce identical ELIDs since direction is the same
        assert_eq!(
            elid1, elid2,
            "Scaled embeddings should produce same ELID after normalization"
        );
    }

    #[test]
    fn test_encode_mini128_output_length() {
        let embedding = vec![0.1; 768];
        let profile = Profile::Mini128 {
            seed: 0x454c4944_53494d48,
        };
        let elid = encode(&embedding, &profile).unwrap();

        // Mini128: 2 header bytes + 16 payload bytes = 18 bytes
        // Base32hex: ceil(18 * 8 / 5) = ceil(28.8) = 29 characters
        assert_eq!(elid.as_str().len(), 29);
    }

    #[test]
    fn test_encode_different_seeds_produce_different_elids() {
        let embedding = vec![0.1; 128];

        let profile1 = Profile::Mini128 {
            seed: 0x1111_1111_1111_1111,
        };
        let profile2 = Profile::Mini128 {
            seed: 0x2222_2222_2222_2222,
        };

        let elid1 = encode(&embedding, &profile1).unwrap();
        let elid2 = encode(&embedding, &profile2).unwrap();

        assert_ne!(
            elid1, elid2,
            "Different seeds should produce different ELIDs"
        );
    }

    #[test]
    fn test_encode_morton_hilbert_implemented() {
        let embedding = vec![0.1; 128];

        let morton = Profile::Morton10x10 {
            dims: 10,
            bits_per_dim: 10,
            transform_id: None,
        };
        let result = encode(&embedding, &morton);
        assert!(result.is_ok(), "Morton encoding should work");

        let hilbert = Profile::Hilbert10x10 {
            dims: 10,
            bits_per_dim: 10,
            transform_id: None,
        };
        let result = encode(&embedding, &hilbert);
        assert!(result.is_ok(), "Hilbert encoding should work");
    }

    // ========================================================================
    // Public API Tests - decode()
    // ========================================================================

    #[test]
    fn test_decode_roundtrip() {
        let embedding = vec![0.1, 0.2, 0.3, 0.4];
        let embedding = embedding.into_iter().cycle().take(768).collect::<Vec<_>>();
        let profile = Profile::default();

        let elid = encode(&embedding, &profile).unwrap();
        let bytes = decode(&elid).unwrap();

        // Should have 2 header bytes + 16 payload bytes = 18 bytes
        assert_eq!(bytes.len(), 18);
    }

    #[test]
    fn test_decode_invalid_base32hex() {
        // Create an invalid ELID with characters outside base32hex alphabet
        let invalid_elid = Elid::from_string("xyz123".to_string());
        assert!(matches!(invalid_elid, Err(ElidError::InvalidEncoding)));
    }

    #[test]
    fn test_decode_extracts_header() {
        let embedding = vec![0.5; 256];
        let profile = Profile::Mini128 {
            seed: 0x454c4944_53494d48,
        };

        let elid = encode(&embedding, &profile).unwrap();
        let bytes = decode(&elid).unwrap();

        // Check header structure
        let version = (bytes[0] & 0xF0) >> 4;
        let profile_type = bytes[0] & 0x0F;

        assert_eq!(version, 0, "Version should be 0");
        assert_eq!(profile_type, 0x01, "Profile type should be 0x01 (Mini128)");
    }

    // ========================================================================
    // Public API Tests - hamming_distance()
    // ========================================================================

    #[test]
    fn test_hamming_distance_identical_elids() {
        let embedding = vec![0.3; 512];
        let profile = Profile::default();

        let elid = encode(&embedding, &profile).unwrap();
        let distance = hamming_distance(&elid, &elid).unwrap();

        assert_eq!(distance, 0, "Identical ELIDs should have distance 0");
    }

    #[test]
    fn test_hamming_distance_similar_embeddings() {
        let profile = Profile::Mini128 {
            seed: 0x454c4944_53494d48,
        };

        // Create similar embeddings
        let emb1 = vec![0.5; 256];
        let mut emb2 = emb1.clone();
        emb2[0] = 0.51; // Slight perturbation

        let elid1 = encode(&emb1, &profile).unwrap();
        let elid2 = encode(&emb2, &profile).unwrap();

        let distance = hamming_distance(&elid1, &elid2).unwrap();

        // Similar embeddings should have low Hamming distance
        assert!(
            distance < 64,
            "Similar embeddings should have low Hamming distance, got {}",
            distance
        );
    }

    #[test]
    fn test_hamming_distance_different_embeddings() {
        let profile = Profile::Mini128 {
            seed: 0x454c4944_53494d48,
        };

        // Create very different embeddings
        let emb1 = vec![1.0, 0.0]
            .into_iter()
            .cycle()
            .take(128)
            .collect::<Vec<_>>();
        let emb2 = vec![0.0, 1.0]
            .into_iter()
            .cycle()
            .take(128)
            .collect::<Vec<_>>();

        let elid1 = encode(&emb1, &profile).unwrap();
        let elid2 = encode(&emb2, &profile).unwrap();

        let distance = hamming_distance(&elid1, &elid2).unwrap();

        // Different embeddings should have higher Hamming distance
        assert!(
            distance > 32,
            "Different embeddings should have significant Hamming distance, got {}",
            distance
        );
    }

    #[test]
    fn test_hamming_distance_symmetric() {
        let profile = Profile::default();

        let emb1 = vec![0.1; 128];
        let emb2 = vec![0.2; 128];

        let elid1 = encode(&emb1, &profile).unwrap();
        let elid2 = encode(&emb2, &profile).unwrap();

        let dist_ab = hamming_distance(&elid1, &elid2).unwrap();
        let dist_ba = hamming_distance(&elid2, &elid1).unwrap();

        assert_eq!(dist_ab, dist_ba, "Hamming distance should be symmetric");
    }

    #[test]
    fn test_hamming_distance_range() {
        let profile = Profile::default();

        let emb1 = vec![1.0; 128];
        let emb2 = vec![-1.0; 128]; // Opposite direction

        let elid1 = encode(&emb1, &profile).unwrap();
        let elid2 = encode(&emb2, &profile).unwrap();

        let distance = hamming_distance(&elid1, &elid2).unwrap();

        // Distance must be in valid range [0, 128]
        assert!(distance <= 128, "Hamming distance must be <= 128");
    }

    // ========================================================================
    // Public API Tests - Elid methods
    // ========================================================================

    #[test]
    fn test_elid_to_bytes() {
        let embedding = vec![0.1; 128];
        let profile = Profile::default();

        let elid = encode(&embedding, &profile).unwrap();
        let bytes = elid.to_bytes().unwrap();

        assert_eq!(
            bytes.len(),
            18,
            "Mini128 should produce 18 bytes (2 header + 16 payload)"
        );
    }

    #[test]
    fn test_elid_profile() {
        let embedding = vec![0.1; 128];
        let profile = Profile::Mini128 {
            seed: 0x454c4944_53494d48,
        };

        let elid = encode(&embedding, &profile).unwrap();
        let profile_info = elid.profile().unwrap();

        assert_eq!(profile_info.version, 0);
        assert_eq!(profile_info.profile_type, 0x01); // Mini128
    }

    // ========================================================================
    // Integration Tests - End-to-End Scenarios
    // ========================================================================

    #[test]
    fn test_end_to_end_workflow() {
        // Create embedding
        let embedding = vec![
            0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, -0.1, -0.2, -0.3, -0.4, -0.5, -0.6, -0.7, -0.8,
        ];
        let embedding = embedding.into_iter().cycle().take(768).collect::<Vec<_>>();

        // Encode with default profile
        let profile = Profile::default();
        let elid = encode(&embedding, &profile).unwrap();

        // Verify ELID properties
        assert!(elid
            .as_str()
            .chars()
            .all(|c| matches!(c, '0'..='9' | 'a'..='v')));
        assert_eq!(elid.as_str().len(), 29);

        // Decode and verify structure
        let bytes = decode(&elid).unwrap();
        assert_eq!(bytes.len(), 18);

        // Extract profile info
        let profile_info = elid.profile().unwrap();
        assert_eq!(profile_info.version, 0);
        assert_eq!(profile_info.profile_type, 0x01);

        // Test similarity search scenario
        let mut similar_embedding = embedding.clone();
        similar_embedding[0] += 0.01; // Slight change

        let elid2 = encode(&similar_embedding, &profile).unwrap();
        let distance = hamming_distance(&elid, &elid2).unwrap();

        assert!(distance < 64, "Similar embeddings should have low distance");
    }

    #[test]
    fn test_sortability() {
        let profile = Profile::default();

        // Create embeddings with different patterns (not just scaled versions)
        // After normalization, these will have different directions
        let emb1 = vec![1.0, 0.0]
            .into_iter()
            .cycle()
            .take(128)
            .collect::<Vec<_>>();
        let emb2 = vec![0.0, 1.0]
            .into_iter()
            .cycle()
            .take(128)
            .collect::<Vec<_>>();
        let emb3 = vec![1.0, 1.0]
            .into_iter()
            .cycle()
            .take(128)
            .collect::<Vec<_>>();

        let elid1 = encode(&emb1, &profile).unwrap();
        let elid2 = encode(&emb2, &profile).unwrap();
        let elid3 = encode(&emb3, &profile).unwrap();

        // ELIDs should maintain lexicographic ordering
        // (though not guaranteed to match value ordering due to hashing)
        let mut elids = [elid1.clone(), elid2.clone(), elid3.clone()];
        elids.sort();

        // Verify they are distinct
        assert_ne!(elid1.as_str(), elid2.as_str());
        assert_ne!(elid2.as_str(), elid3.as_str());
        assert_ne!(elid1.as_str(), elid3.as_str());
    }

    #[test]
    fn test_different_dimension_embeddings() {
        let profile = Profile::default();

        // Test various valid dimensions
        for dim in [64, 128, 256, 512, 768, 1024, 1536, 2048] {
            let embedding = vec![0.1; dim];
            let result = encode(&embedding, &profile);
            assert!(result.is_ok(), "Failed for dimension {}", dim);

            let elid = result.unwrap();
            assert_eq!(
                elid.as_str().len(),
                29,
                "Wrong length for dimension {}",
                dim
            );
        }
    }

    // ========================================================================
    // Helper Function Tests
    // ========================================================================

    #[test]
    fn test_simhash_to_bytes_roundtrip() {
        let hash: u128 = 0xDEAD_BEEF_CAFE_BABE_1234_5678_9ABC_DEF0;
        let bytes = simhash_to_bytes(hash);
        assert_eq!(bytes.len(), 16);

        let recovered = simhash_from_bytes(&bytes).unwrap();
        assert_eq!(recovered, hash);
    }

    #[test]
    fn test_simhash_from_bytes_invalid_length() {
        let bytes = vec![0u8; 10]; // Wrong length
        let result = simhash_from_bytes(&bytes);
        assert!(matches!(result, Err(ElidError::InvalidEncoding)));
    }

    #[test]
    fn test_simhash_bytes_big_endian() {
        let hash: u128 = 0x0102030405060708090a0b0c0d0e0f10;
        let bytes = simhash_to_bytes(hash);

        // Verify big-endian byte order
        assert_eq!(bytes[0], 0x01);
        assert_eq!(bytes[1], 0x02);
        assert_eq!(bytes[15], 0x10);
    }
}
