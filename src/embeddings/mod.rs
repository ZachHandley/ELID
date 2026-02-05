//! Embedding encoding module for ELID
//!
//! This module provides compact, sortable identifiers for high-dimensional embeddings.
//! It includes support for four encoding profiles:
//!
//! - **Mini128**: 128-bit SimHash using signed random projections
//! - **Morton10x10**: Z-order curve encoding for database indexing
//! - **Hilbert10x10**: Hilbert curve encoding for maximum locality preservation
//! - **FullVector**: Reversible encoding with configurable precision and dimension reduction
//!
//! # Feature Gate
//!
//! This module is gated behind the `embeddings` feature flag:
//!
//! ```toml
//! [dependencies]
//! elid = { version = "0.1", features = ["embeddings"] }
//! ```

// Submodules
pub mod encoding;
pub mod error;
pub mod full_vector;
pub mod hilbert;
pub mod morton;
pub mod types;
pub mod vector_simhash;

// Re-exports for public API
pub use encoding::{decode_sortable, encode_sortable};
pub use error::ElidError;
pub use full_vector::{decode_full_vector, encode_full_vector, FullVectorMetadata};
pub use types::{
    DimensionMode, Elid, Embedding, Profile, ProfileInfo, QuantizedCoords, VectorPrecision,
};
pub use vector_simhash::{
    cosine_similarity_approx, elid_hamming_distance, embedding_to_bands, mini128_to_bands,
    simhash_128, simhash_from_bytes, simhash_to_bytes,
};

// Re-export curve functions
pub use hilbert::{hilbert_decode, hilbert_encode};
pub use morton::{morton_decode, morton_encode};

/// Encode an embedding into an ELID string
///
/// Converts a high-dimensional embedding vector into a compact, sortable identifier
/// using the specified profile. The resulting ELID preserves locality properties
/// for efficient similarity search.
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
/// # Examples
///
/// ```rust,ignore
/// use elid::embeddings::{encode, Profile};
///
/// let embedding = vec![0.1; 768];
/// let profile = Profile::default(); // Mini128
/// let elid = encode(&embedding, &profile)?;
/// println!("ELID: {}", elid);
/// ```
pub fn encode(embedding: &[f32], profile: &Profile) -> Result<Elid, ElidError> {
    // Step 1: Create and validate embedding
    let mut emb = Embedding::new(embedding.to_vec())?;

    // Step 2: Normalize to unit length (except for FullVector which may want unnormalized)
    if !matches!(profile, Profile::FullVector { .. }) {
        emb.normalize();
    }

    // Step 3: Apply profile-specific encoding
    let combined_bytes = match profile {
        Profile::Mini128 { seed } => {
            // Compute 128-bit SimHash
            let hash = simhash_128(emb.as_slice(), *seed);
            // Convert to big-endian bytes (16 bytes)
            let payload = simhash_to_bytes(hash).to_vec();

            // Create header
            let header = ProfileInfo {
                version: 0,
                profile_type: profile.type_id(),
                transform_id: None,
                model_id: None,
                original_dims: None,
                precision: None,
                dimension_mode: None,
                seed: None,
            };
            let mut combined = header.to_header();
            combined.extend_from_slice(&payload);
            combined
        }
        Profile::Morton10x10 {
            dims,
            bits_per_dim,
            transform_id,
        } => {
            // Morton curve encoding
            if transform_id.is_some() {
                return Err(ElidError::TransformNotFound(transform_id.unwrap()));
            }

            let quantized = QuantizedCoords::from_embedding(&emb, *dims, *bits_per_dim)?;
            let code = morton_encode(quantized.as_slice(), *bits_per_dim);
            let total_bits = (*dims as usize) * (*bits_per_dim as usize);
            let payload = code_to_bytes(code, total_bits);

            let header = ProfileInfo {
                version: 0,
                profile_type: profile.type_id(),
                transform_id: None,
                model_id: None,
                original_dims: None,
                precision: None,
                dimension_mode: None,
                seed: None,
            };
            let mut combined = header.to_header();
            combined.extend_from_slice(&payload);
            combined
        }
        Profile::Hilbert10x10 {
            dims,
            bits_per_dim,
            transform_id,
        } => {
            // Hilbert curve encoding
            if transform_id.is_some() {
                return Err(ElidError::TransformNotFound(transform_id.unwrap()));
            }

            let quantized = QuantizedCoords::from_embedding(&emb, *dims, *bits_per_dim)?;
            let code = hilbert_encode(quantized.as_slice(), *bits_per_dim);
            let total_bits = (*dims as usize) * (*bits_per_dim as usize);
            let payload = code_to_bytes(code, total_bits);

            let header = ProfileInfo {
                version: 0,
                profile_type: profile.type_id(),
                transform_id: None,
                model_id: None,
                original_dims: None,
                precision: None,
                dimension_mode: None,
                seed: None,
            };
            let mut combined = header.to_header();
            combined.extend_from_slice(&payload);
            combined
        }
        Profile::FullVector {
            precision,
            dimensions,
            seed,
        } => {
            // Full vector encoding (reversible)
            encode_full_vector(emb.as_slice(), *precision, *dimensions, *seed)?
        }
    };

    // Step 4: Encode to base32hex
    let encoded = encode_sortable(&combined_bytes);

    // Step 5: Create Elid
    Elid::from_string(encoded)
}

/// Decode an ELID string to raw bytes
///
/// Decodes a base32hex-encoded ELID string back to its raw byte representation.
///
/// # Parameters
///
/// - `elid`: The ELID string to decode
///
/// # Returns
///
/// - `Ok(Vec<u8>)`: Raw bytes (header + payload)
/// - `Err(ElidError::InvalidEncoding)`: Invalid base32hex string
pub fn decode(elid: &Elid) -> Result<Vec<u8>, ElidError> {
    decode_sortable(elid.as_str())
}

/// Decode an ELID back to an embedding vector
///
/// Only supported for FullVector profiles. Other profiles use lossy hashing
/// that cannot be reversed.
///
/// # Parameters
///
/// - `elid`: The ELID to decode
///
/// # Returns
///
/// - `Ok((Vec<f32>, FullVectorMetadata))`: Decoded embedding and metadata
/// - `Err(ElidError::DecodingNotSupported)`: Profile does not support decoding
///
/// # Note
///
/// If dimension reduction was used during encoding, the decoded embedding
/// will be in the reduced dimension space, not the original dimension space.
/// The metadata contains information about the original dimensions.
///
/// # Examples
///
/// ```rust,ignore
/// use elid::embeddings::{encode, decode_to_embedding, Profile};
///
/// let embedding = vec![0.1; 768];
/// let profile = Profile::lossless();
/// let elid = encode(&embedding, &profile)?;
///
/// let (decoded, metadata) = decode_to_embedding(&elid)?;
/// assert_eq!(embedding, decoded);
/// ```
pub fn decode_to_embedding(elid: &Elid) -> Result<(Vec<f32>, FullVectorMetadata), ElidError> {
    let bytes = decode(elid)?;

    // Check profile type
    if bytes.len() < 2 {
        return Err(ElidError::InvalidHeader);
    }

    let profile_type = bytes[0] & 0x0F;
    if profile_type != 0x04 {
        return Err(ElidError::DecodingNotSupported);
    }

    // Decode full vector
    decode_full_vector(&bytes)
}

/// Check if an ELID can be decoded back to an embedding
///
/// Returns `true` if the ELID was encoded with a FullVector profile,
/// which supports reversible encoding.
///
/// # Parameters
///
/// - `elid`: The ELID to check
///
/// # Returns
///
/// `true` if the embedding can be recovered, `false` otherwise
///
/// # Examples
///
/// ```rust,ignore
/// use elid::embeddings::{encode, is_reversible, Profile};
///
/// let embedding = vec![0.1; 768];
///
/// let mini128 = encode(&embedding, &Profile::default())?;
/// assert!(!is_reversible(&mini128));
///
/// let full_vector = encode(&embedding, &Profile::lossless())?;
/// assert!(is_reversible(&full_vector));
/// ```
pub fn is_reversible(elid: &Elid) -> bool {
    match elid.to_bytes() {
        Ok(bytes) if bytes.len() >= 2 => {
            let profile_type = bytes[0] & 0x0F;
            profile_type == 0x04 // FullVector
        }
        _ => false,
    }
}

/// Compute Hamming distance between two Mini128 ELIDs
///
/// Returns the number of differing bits in the SimHash payloads of two ELIDs.
/// This distance is proportional to the angular distance between the original
/// embeddings (Charikar 2002).
///
/// # Requirements
///
/// Both ELIDs must use the Mini128 profile.
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
    Ok(elid_hamming_distance(hash_a, hash_b))
}

/// Convert u128 code to big-endian bytes (only needed bytes)
///
/// For Morton/Hilbert codes, we only need enough bytes to hold `total_bits`.
fn code_to_bytes(code: u128, total_bits: usize) -> Vec<u8> {
    let needed_bytes = total_bits.div_ceil(8);
    let all_bytes = code.to_be_bytes();

    // Take only the needed bytes from the end (big-endian, so MSB first)
    let start_idx = 16 - needed_bytes;
    all_bytes[start_idx..].to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_hamming_distance_identical_elids() {
        let embedding = vec![0.3; 512];
        let profile = Profile::default();

        let elid = encode(&embedding, &profile).unwrap();
        let distance = hamming_distance(&elid, &elid).unwrap();

        assert_eq!(distance, 0, "Identical ELIDs should have distance 0");
    }

    #[test]
    fn test_encode_morton() {
        let embedding = vec![0.1; 128];
        let profile = Profile::Morton10x10 {
            dims: 10,
            bits_per_dim: 10,
            transform_id: None,
        };
        let result = encode(&embedding, &profile);
        assert!(result.is_ok(), "Morton encoding should work");
    }

    #[test]
    fn test_encode_hilbert() {
        let embedding = vec![0.1; 128];
        let profile = Profile::Hilbert10x10 {
            dims: 10,
            bits_per_dim: 10,
            transform_id: None,
        };
        let result = encode(&embedding, &profile);
        assert!(result.is_ok(), "Hilbert encoding should work");
    }

    // ========================================================================
    // FullVector Tests
    // ========================================================================

    #[test]
    fn test_encode_full_vector_lossless() {
        let embedding: Vec<f32> = (0..128).map(|i| (i as f32 / 64.0) - 1.0).collect();
        let profile = Profile::lossless();

        let elid = encode(&embedding, &profile).unwrap();
        assert!(is_reversible(&elid));

        let (decoded, metadata) = decode_to_embedding(&elid).unwrap();
        assert_eq!(embedding, decoded, "Lossless encoding should be exact");
        assert!(metadata.is_lossless());
    }

    #[test]
    fn test_encode_full_vector_half16() {
        let embedding: Vec<f32> = (0..768).map(|i| (i as f32 / 384.0) - 1.0).collect();
        let profile = Profile::FullVector {
            precision: VectorPrecision::Half16,
            dimensions: DimensionMode::Preserve,
            seed: 0,
        };

        let elid = encode(&embedding, &profile).unwrap();
        assert!(is_reversible(&elid));

        let (decoded, metadata) = decode_to_embedding(&elid).unwrap();
        assert_eq!(decoded.len(), embedding.len());
        assert!(!metadata.is_lossless());

        // Check error is reasonable (half precision)
        let max_error: f32 = embedding
            .iter()
            .zip(decoded.iter())
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f32, f32::max);
        assert!(max_error < 0.01, "Half16 max error: {}", max_error);
    }

    #[test]
    fn test_encode_full_vector_quant8() {
        let embedding: Vec<f32> = (0..256).map(|i| (i as f32 / 128.0) - 1.0).collect();
        let profile = Profile::FullVector {
            precision: VectorPrecision::Quant8,
            dimensions: DimensionMode::Preserve,
            seed: 0,
        };

        let elid = encode(&embedding, &profile).unwrap();
        let (decoded, _) = decode_to_embedding(&elid).unwrap();

        // Check error is reasonable (8-bit quantization)
        let max_error: f32 = embedding
            .iter()
            .zip(decoded.iter())
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f32, f32::max);
        assert!(max_error < 0.02, "Quant8 max error: {}", max_error);
    }

    #[test]
    fn test_encode_full_vector_dimension_reduction() {
        let embedding: Vec<f32> = (0..768).map(|i| (i as f32 / 384.0) - 1.0).collect();
        let profile = Profile::FullVector {
            precision: VectorPrecision::Full32,
            dimensions: DimensionMode::Reduce { target_dims: 256 },
            seed: 0x12345678,
        };

        let elid = encode(&embedding, &profile).unwrap();
        let (decoded, metadata) = decode_to_embedding(&elid).unwrap();

        assert_eq!(decoded.len(), 256);
        assert_eq!(metadata.original_dims, 768);
        assert_eq!(metadata.encoded_dims, 256);
        assert!(metadata.has_dimension_reduction());
    }

    #[test]
    fn test_encode_full_vector_cross_dimensional() {
        // Two embeddings of different dimensions
        let emb_256: Vec<f32> = (0..256).map(|i| (i as f32 / 128.0) - 1.0).collect();
        let emb_768: Vec<f32> = (0..768).map(|i| (i as f32 / 384.0) - 1.0).collect();

        let profile = Profile::cross_dimensional(128);

        // Encode both - cross_dimensional uses a fixed seed internally
        let elid_256 = encode(&emb_256, &profile).unwrap();
        let elid_768 = encode(&emb_768, &profile).unwrap();

        let (dec_256, meta_256) = decode_to_embedding(&elid_256).unwrap();
        let (dec_768, meta_768) = decode_to_embedding(&elid_768).unwrap();

        // Both should decode to 128 dimensions
        assert_eq!(dec_256.len(), 128);
        assert_eq!(dec_768.len(), 128);

        // Metadata should preserve original dimensions
        assert_eq!(meta_256.original_dims, 256);
        assert_eq!(meta_768.original_dims, 768);

        // Now they can be directly compared (same dimensionality)
        let similarity: f32 = dec_256.iter().zip(dec_768.iter()).map(|(a, b)| a * b).sum();
        // Just verify the comparison is possible, not the specific value
        assert!(similarity.is_finite());
    }

    #[test]
    fn test_is_reversible_mini128() {
        let embedding = vec![0.1; 128];
        let profile = Profile::default(); // Mini128

        let elid = encode(&embedding, &profile).unwrap();
        assert!(!is_reversible(&elid), "Mini128 should not be reversible");
    }

    #[test]
    fn test_is_reversible_full_vector() {
        let embedding = vec![0.1; 128];
        let profile = Profile::lossless();

        let elid = encode(&embedding, &profile).unwrap();
        assert!(is_reversible(&elid), "FullVector should be reversible");
    }

    #[test]
    fn test_decode_to_embedding_unsupported() {
        let embedding = vec![0.1; 128];
        let profile = Profile::default(); // Mini128

        let elid = encode(&embedding, &profile).unwrap();
        let result = decode_to_embedding(&elid);

        assert!(
            matches!(result, Err(ElidError::DecodingNotSupported)),
            "Mini128 should not support decode_to_embedding"
        );
    }

    #[test]
    fn test_profile_max_length_constraint() {
        let embedding: Vec<f32> = (0..768).map(|i| (i as f32 / 384.0) - 1.0).collect();
        let max_chars = 100;
        let profile = Profile::max_length(max_chars, 768);

        let elid = encode(&embedding, &profile).unwrap();

        assert!(
            elid.as_str().len() <= max_chars,
            "ELID length {} exceeds max {}",
            elid.as_str().len(),
            max_chars
        );
    }

    #[test]
    fn test_profile_compressed_retention() {
        let embedding: Vec<f32> = (0..768).map(|i| (i as f32 / 384.0) - 1.0).collect();

        // Test various retention levels
        for retention in [1.0, 0.5, 0.25, 0.1] {
            let profile = Profile::compressed(retention, 768);
            let elid = encode(&embedding, &profile).unwrap();

            // Should encode successfully
            assert!(is_reversible(&elid));

            // Verify output size is proportional to retention (roughly)
            let full_size = Profile::lossless().string_length_for_dims(768);
            let compressed_size = elid.as_str().len();

            // Allow some variance due to header overhead and rounding
            if retention < 0.9 {
                assert!(
                    compressed_size < full_size,
                    "Retention {} should reduce size (full: {}, compressed: {})",
                    retention,
                    full_size,
                    compressed_size
                );
            }
        }
    }
}
