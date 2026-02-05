//! Embedding encoding module for ELID
//!
//! This module provides compact, sortable identifiers for high-dimensional embeddings.
//! It includes support for three encoding profiles:
//!
//! - **Mini128**: 128-bit SimHash using signed random projections
//! - **Morton10x10**: Z-order curve encoding for database indexing
//! - **Hilbert10x10**: Hilbert curve encoding for maximum locality preservation
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
pub mod hilbert;
pub mod morton;
pub mod types;
pub mod vector_simhash;

// Re-exports for public API
pub use encoding::{decode_sortable, encode_sortable};
pub use error::ElidError;
pub use types::{Elid, Embedding, Profile, ProfileInfo, QuantizedCoords};
pub use vector_simhash::{
    cosine_similarity_approx, elid_hamming_distance, simhash_128, simhash_from_bytes,
    simhash_to_bytes,
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

    // Step 2: Normalize to unit length
    emb.normalize();

    // Step 3: Apply profile-specific encoding
    let payload_bytes = match profile {
        Profile::Mini128 { seed } => {
            // Compute 128-bit SimHash
            let hash = simhash_128(emb.as_slice(), *seed);
            // Convert to big-endian bytes (16 bytes)
            simhash_to_bytes(hash).to_vec()
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
            code_to_bytes(code, total_bits)
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
}
