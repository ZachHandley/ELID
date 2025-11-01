#![deny(clippy::all)]

//! Node.js bindings for ELID using napi-rs
//!
//! This crate provides TypeScript/JavaScript bindings with automatic type generation.

#[cfg(not(target_arch = "wasm32"))]
use elid_core::{decode, encode, hamming_distance, Elid, Profile};

#[cfg(not(target_arch = "wasm32"))]
use napi::bindgen_prelude::*;

#[cfg(not(target_arch = "wasm32"))]
use napi_derive::napi;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

/// ELID encoding profile
#[cfg(not(target_arch = "wasm32"))]
#[napi]
pub enum ElidProfile {
  /// 128-bit SimHash (29 characters) - Best for similarity search
  Mini128 = 0,
  /// 10D Morton curve (16-24 characters) - Fast indexing
  Morton10x10 = 1,
  /// 10D Hilbert curve (16-24 characters) - Maximum locality
  Hilbert10x10 = 2,
}

#[cfg(not(target_arch = "wasm32"))]
impl From<ElidProfile> for Profile {
  fn from(profile: ElidProfile) -> Self {
    match profile {
      ElidProfile::Mini128 => Profile::Mini128 { seed: 0 },
      ElidProfile::Morton10x10 => Profile::Morton10x10 {
        dims: 10,
        bits_per_dim: 10,
        transform_id: None,
      },
      ElidProfile::Hilbert10x10 => Profile::Hilbert10x10 {
        dims: 10,
        bits_per_dim: 10,
        transform_id: None,
      },
    }
  }
}

/// Encode a high-dimensional embedding to a sortable string identifier.
///
/// # Arguments
///
/// * `embedding` - Float64Array of length 64-2048
/// * `profile` - Encoding profile (Mini128, Morton10x10, or Hilbert10x10)
///
/// # Returns
///
/// Sortable string identifier (29 chars for Mini128, 16-24 for others)
///
/// # Errors
///
/// Throws if embedding dimensions are invalid or encoding fails
///
/// # Example
///
/// ```javascript
/// const { encodeElid, ElidProfile } = require('elid');
///
/// const embedding = new Float64Array(768).fill(0.5);
/// const elidId = encodeElid(embedding, ElidProfile.Mini128);
/// console.log(elidId); // 29-character string
/// ```
#[cfg(not(target_arch = "wasm32"))]
#[napi]
pub fn encode_elid(embedding: Float64Array, profile: ElidProfile) -> Result<String> {
  let embedding_slice: &[f64] = embedding.as_ref();

  // Convert f64 to f32 as elid-core expects f32
  let embedding_f32: Vec<f32> = embedding_slice.iter().map(|&x| x as f32).collect();

  let rust_profile: Profile = profile.into();

  encode(&embedding_f32, &rust_profile)
    .map(|elid| elid.to_string())
    .map_err(|e| Error::from_reason(format!("Encoding failed: {}", e)))
}

/// Decode an ELID string back to raw bytes.
///
/// # Arguments
///
/// * `elid` - ELID string identifier (base32hex encoded)
///
/// # Returns
///
/// Buffer containing raw bytes including 2-byte header (18 bytes for Mini128; 15 bytes for Morton/Hilbert10x10)
///
/// # Errors
///
/// Throws if ELID string is malformed or invalid encoding
///
/// # Example
///
/// ```javascript
/// const { decodeElid } = require('elid');
///
/// const bytes = decodeElid('0123456789ABCDEFGHIJKLMNOPQ');
/// console.log(bytes.length); // 16
/// ```
#[cfg(not(target_arch = "wasm32"))]
#[napi]
pub fn decode_elid(elid: String) -> Result<Buffer> {
  let elid_obj = Elid::from_string(elid)
    .map_err(|e| Error::from_reason(format!("Invalid ELID: {}", e)))?;

  decode(&elid_obj)
    .map(|bytes| bytes.into())
    .map_err(|e| Error::from_reason(format!("Decoding failed: {}", e)))
}

/// Calculate Hamming distance between two ELIDs.
///
/// # Arguments
///
/// * `elid1` - First ELID string
/// * `elid2` - Second ELID string (must use same profile as elid1)
///
/// # Returns
///
/// Hamming distance (0-128 for Mini128, 0-80 for Morton/Hilbert10x10).
/// Lower distance indicates higher similarity.
///
/// # Errors
///
/// Throws if ELIDs use different profiles or are malformed
///
/// # Example
///
/// ```javascript
/// const { encodeElid, hammingDistanceElid, ElidProfile } = require('elid');
///
/// const embedding = new Float64Array(768).fill(1.0);
/// const elid1 = encodeElid(embedding, ElidProfile.Mini128);
/// const elid2 = encodeElid(embedding, ElidProfile.Mini128);
/// const distance = hammingDistanceElid(elid1, elid2);
/// console.log(distance); // 0 (identical embeddings)
/// ```
#[cfg(not(target_arch = "wasm32"))]
#[napi]
pub fn hamming_distance_elid(elid1: String, elid2: String) -> Result<u32> {
  let elid_obj1 = Elid::from_string(elid1)
    .map_err(|e| Error::from_reason(format!("Invalid ELID 1: {}", e)))?;
  let elid_obj2 = Elid::from_string(elid2)
    .map_err(|e| Error::from_reason(format!("Invalid ELID 2: {}", e)))?;

  hamming_distance(&elid_obj1, &elid_obj2)
    .map_err(|e| Error::from_reason(format!("Hamming distance calculation failed: {}", e)))
}

/// Encode multiple embeddings asynchronously using Tokio for non-blocking execution.
///
/// # Arguments
///
/// * `embeddings` - Array of Float64Array, each length 64-2048
/// * `profile` - Encoding profile for all embeddings
///
/// # Returns
///
/// Promise resolving to array of ELID strings, same order as input
///
/// # Errors
///
/// Throws if any embedding has invalid dimensions
///
/// # Example
///
/// ```javascript
/// const { encodeBatch, ElidProfile } = require('elid');
///
/// const embeddings = Array.from({ length: 1000 }, () =>
///   new Float64Array(768).fill(Math.random())
/// );
/// const elids = await encodeBatch(embeddings, ElidProfile.Mini128);
/// console.log(elids.length); // 1000
/// ```
#[cfg(not(target_arch = "wasm32"))]
#[napi]
pub async fn encode_batch(embeddings: Vec<Float64Array>, profile: ElidProfile) -> Result<Vec<String>> {
  let rust_profile: Profile = profile.into();

  // Use tokio::task::spawn_blocking for CPU-intensive work
  tokio::task::spawn_blocking(move || {
    embeddings
      .iter()
      .map(|embedding| {
        let embedding_slice: &[f64] = embedding.as_ref();
        let embedding_f32: Vec<f32> = embedding_slice.iter().map(|&x| x as f32).collect();

        encode(&embedding_f32, &rust_profile)
          .map(|elid| elid.to_string())
          .map_err(|e| Error::from_reason(format!("Batch encoding failed: {}", e)))
      })
      .collect::<Result<Vec<String>>>()
  })
  .await
  .map_err(|e| Error::from_reason(format!("Task execution failed: {}", e)))?
}

#[cfg(test)]
mod tests {
  #[cfg(not(target_arch = "wasm32"))]
  use super::*;

  #[cfg(not(target_arch = "wasm32"))]
  #[test]
  fn test_profile_conversion() {
    let profile = ElidProfile::Mini128;
    let rust_profile: Profile = profile.into();
    match rust_profile {
      Profile::Mini128 { seed } => assert_eq!(seed, 0),
      _ => panic!("Wrong profile type"),
    }
  }
}
