//! WebAssembly bindings for ELID
//!
//! This module provides JavaScript-friendly bindings for all ELID functions.
//! These bindings work in browsers, Node.js, Deno, and Bun.

use js_sys::{Float64Array, Object, Reflect};
use wasm_bindgen::prelude::*;

// Conditional imports for embeddings feature
#[cfg(feature = "embeddings")]
use crate::embeddings::{self, DimensionMode, Profile, VectorPrecision};

/// Compute the Levenshtein distance between two strings.
///
/// Returns the minimum number of single-character edits needed to transform one string into another.
///
/// # JavaScript Example
///
/// ```javascript
/// import { levenshtein } from 'elid';
///
/// const distance = levenshtein("kitten", "sitting");
/// console.log(distance); // 3
/// ```
#[wasm_bindgen]
pub fn levenshtein(a: &str, b: &str) -> usize {
    crate::levenshtein(a, b)
}

/// Compute the normalized Levenshtein similarity between two strings.
///
/// Returns a value between 0.0 (completely different) and 1.0 (identical).
///
/// # JavaScript Example
///
/// ```javascript
/// import { normalizedLevenshtein } from 'elid';
///
/// const similarity = normalizedLevenshtein("hello", "hallo");
/// console.log(similarity); // ~0.8
/// ```
#[wasm_bindgen(js_name = normalizedLevenshtein)]
pub fn normalized_levenshtein(a: &str, b: &str) -> f64 {
    crate::normalized_levenshtein(a, b)
}

/// Compute the Jaro similarity between two strings.
///
/// Returns a value between 0.0 (completely different) and 1.0 (identical).
/// Particularly effective for short strings like names.
///
/// # JavaScript Example
///
/// ```javascript
/// import { jaro } from 'elid';
///
/// const similarity = jaro("martha", "marhta");
/// console.log(similarity); // ~0.944
/// ```
#[wasm_bindgen]
pub fn jaro(a: &str, b: &str) -> f64 {
    crate::jaro(a, b)
}

/// Compute the Jaro-Winkler similarity between two strings.
///
/// Returns a value between 0.0 (completely different) and 1.0 (identical).
/// Gives more favorable ratings to strings with common prefixes.
///
/// # JavaScript Example
///
/// ```javascript
/// import { jaroWinkler } from 'elid';
///
/// const similarity = jaroWinkler("martha", "marhta");
/// console.log(similarity); // ~0.961
/// ```
#[wasm_bindgen(js_name = jaroWinkler)]
pub fn jaro_winkler(a: &str, b: &str) -> f64 {
    crate::jaro_winkler(a, b)
}

/// Compute the Hamming distance between two strings.
///
/// Returns the number of positions at which the characters differ.
/// Returns null if strings have different lengths.
///
/// # JavaScript Example
///
/// ```javascript
/// import { hamming } from 'elid';
///
/// const distance = hamming("karolin", "kathrin");
/// console.log(distance); // 3
///
/// const invalid = hamming("hello", "world!");
/// console.log(invalid); // null
/// ```
#[wasm_bindgen]
pub fn hamming(a: &str, b: &str) -> Option<usize> {
    crate::hamming(a, b)
}

/// Compute the OSA (Optimal String Alignment) distance between two strings.
///
/// Similar to Levenshtein but also considers transpositions as a single operation.
///
/// # JavaScript Example
///
/// ```javascript
/// import { osaDistance } from 'elid';
///
/// const distance = osaDistance("ca", "ac");
/// console.log(distance); // 1 (transposition)
/// ```
#[wasm_bindgen(js_name = osaDistance)]
pub fn osa_distance(a: &str, b: &str) -> usize {
    crate::osa_distance(a, b)
}

/// Compute the best matching similarity between two strings.
///
/// Runs multiple algorithms and returns the highest score.
///
/// # JavaScript Example
///
/// ```javascript
/// import { bestMatch } from 'elid';
///
/// const score = bestMatch("hello", "hallo");
/// console.log(score); // ~0.8
/// ```
#[wasm_bindgen(js_name = bestMatch)]
pub fn best_match(a: &str, b: &str) -> f64 {
    crate::best_match(a, b)
}

/// Find the best match for a query string in an array of candidates.
///
/// Returns an object with the index and similarity score of the best match.
///
/// # JavaScript Example
///
/// ```javascript
/// import { findBestMatch } from 'elid';
///
/// const candidates = ["apple", "application", "apply"];
/// const result = findBestMatch("app", candidates);
/// console.log(result); // { index: 0, score: 0.907 }
/// ```
#[wasm_bindgen(js_name = findBestMatch)]
pub fn find_best_match(query: &str, candidates: Vec<String>) -> Object {
    let candidate_refs: Vec<&str> = candidates.iter().map(|s| s.as_str()).collect();
    let (idx, score) = crate::find_best_match(query, &candidate_refs);

    let result = Object::new();
    Reflect::set(&result, &"index".into(), &JsValue::from(idx)).unwrap();
    Reflect::set(&result, &"score".into(), &JsValue::from(score)).unwrap();
    result
}

/// Find all matches above a threshold score.
///
/// Returns an array of objects with index and score for all candidates above the threshold.
///
/// # JavaScript Example
///
/// ```javascript
/// import { findMatchesAboveThreshold } from 'elid';
///
/// const candidates = ["apple", "application", "apply", "banana"];
/// const matches = findMatchesAboveThreshold("app", candidates, 0.5);
/// console.log(matches); // [{ index: 0, score: 0.907 }, { index: 1, score: 0.830 }, ...]
/// ```
#[wasm_bindgen(js_name = findMatchesAboveThreshold)]
pub fn find_matches_above_threshold(
    query: &str,
    candidates: Vec<String>,
    threshold: f64,
) -> JsValue {
    let candidate_refs: Vec<&str> = candidates.iter().map(|s| s.as_str()).collect();
    let matches = crate::find_matches_above_threshold(query, &candidate_refs, threshold);

    let results: Vec<_> = matches
        .into_iter()
        .map(|(idx, score)| {
            serde_json::json!({
                "index": idx,
                "score": score
            })
        })
        .collect();

    serde_wasm_bindgen::to_value(&results).unwrap()
}

/// Options for configuring string similarity algorithms
#[wasm_bindgen]
pub struct SimilarityOptions {
    /// Case-sensitive comparison (default: true)
    #[wasm_bindgen(getter_with_clone)]
    pub case_sensitive: bool,
    /// Trim whitespace before comparison (default: false)
    #[wasm_bindgen(getter_with_clone)]
    pub trim_whitespace: bool,
    /// Prefix scale for Jaro-Winkler (default: 0.1, max: 0.25)
    #[wasm_bindgen(getter_with_clone)]
    pub prefix_scale: f64,
}

impl Default for SimilarityOptions {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl SimilarityOptions {
    /// Create a new SimilarityOptions with default values
    #[wasm_bindgen(constructor)]
    pub fn new() -> SimilarityOptions {
        SimilarityOptions {
            case_sensitive: true,
            trim_whitespace: false,
            prefix_scale: 0.1,
        }
    }

    /// Set case sensitivity
    #[wasm_bindgen(js_name = setCaseSensitive)]
    pub fn set_case_sensitive(&mut self, value: bool) {
        self.case_sensitive = value;
    }

    /// Set whitespace trimming
    #[wasm_bindgen(js_name = setTrimWhitespace)]
    pub fn set_trim_whitespace(&mut self, value: bool) {
        self.trim_whitespace = value;
    }

    /// Set prefix scale
    #[wasm_bindgen(js_name = setPrefixScale)]
    pub fn set_prefix_scale(&mut self, value: f64) {
        self.prefix_scale = value;
    }
}

impl From<SimilarityOptions> for crate::SimilarityOpts {
    fn from(opts: SimilarityOptions) -> Self {
        crate::SimilarityOpts {
            case_sensitive: opts.case_sensitive,
            trim_whitespace: opts.trim_whitespace,
            prefix_scale: opts.prefix_scale,
        }
    }
}

/// Compute Levenshtein distance with custom options.
///
/// # JavaScript Example
///
/// ```javascript
/// import { levenshteinWithOpts, SimilarityOptions } from 'elid';
///
/// const opts = new SimilarityOptions();
/// opts.setCaseSensitive(false);
/// opts.setTrimWhitespace(true);
///
/// const distance = levenshteinWithOpts("  HELLO  ", "hello", opts);
/// console.log(distance); // 0
/// ```
#[wasm_bindgen(js_name = levenshteinWithOpts)]
pub fn levenshtein_with_opts(a: &str, b: &str, opts: SimilarityOptions) -> usize {
    let rust_opts = crate::SimilarityOpts::from(opts);
    crate::levenshtein_with_opts(a, b, &rust_opts)
}

/// Compute the SimHash fingerprint of a string.
///
/// Returns a 64-bit hash where similar strings produce similar numbers.
/// Use this for database queries by storing the hash and querying by numeric range.
///
/// # JavaScript Example
///
/// ```javascript
/// import { simhash } from 'elid';
///
/// const hash1 = simhash("iPhone 14");
/// const hash2 = simhash("iPhone 15");
/// const hash3 = simhash("Galaxy S23");
///
/// // hash1 and hash2 will be numerically close
/// // hash3 will be numerically distant
///
/// // Store in database as bigint:
/// // { name: "iPhone 14", simhash: hash1 }
/// ```
#[wasm_bindgen]
pub fn simhash(text: &str) -> f64 {
    // JavaScript doesn't have native u64, so we return as f64 (safe for 53 bits)
    crate::simhash(text) as f64
}

/// Compute the Hamming distance between two SimHash values.
///
/// Returns the number of differing bits. Lower values = higher similarity.
///
/// # JavaScript Example
///
/// ```javascript
/// import { simhash, simhashDistance } from 'elid';
///
/// const hash1 = simhash("iPhone 14");
/// const hash2 = simhash("iPhone 15");
/// const distance = simhashDistance(hash1, hash2);
///
/// console.log(distance); // Low number = similar
/// ```
#[wasm_bindgen(js_name = simhashDistance)]
pub fn simhash_distance(hash1: f64, hash2: f64) -> u32 {
    crate::simhash_distance(hash1 as u64, hash2 as u64)
}

/// Compute the normalized SimHash similarity between two strings.
///
/// Returns a value between 0.0 (completely different) and 1.0 (identical).
///
/// # JavaScript Example
///
/// ```javascript
/// import { simhashSimilarity } from 'elid';
///
/// const similarity = simhashSimilarity("iPhone 14", "iPhone 15");
/// console.log(similarity); // ~0.9 (very similar)
///
/// const similarity2 = simhashSimilarity("iPhone", "Galaxy");
/// console.log(similarity2); // ~0.4 (different)
/// ```
#[wasm_bindgen(js_name = simhashSimilarity)]
pub fn simhash_similarity(a: &str, b: &str) -> f64 {
    crate::simhash_similarity(a, b)
}

/// Find all hashes within a given distance threshold.
///
/// Useful for database queries - pre-compute hashes, then find similar ones.
///
/// # JavaScript Example
///
/// ```javascript
/// import { simhash, findSimilarHashes } from 'elid';
///
/// const candidates = ["iPhone 14 Pro", "iPhone 13", "Galaxy S23"];
/// const hashes = candidates.map(s => simhash(s));
///
/// const queryHash = simhash("iPhone 14");
/// const matches = findSimilarHashes(queryHash, hashes, 10);
///
/// console.log(matches); // [0, 1] - indices of similar items
/// ```
#[wasm_bindgen(js_name = findSimilarHashes)]
pub fn find_similar_hashes(
    query_hash: f64,
    candidate_hashes: Vec<f64>,
    max_distance: u32,
) -> Vec<usize> {
    let u64_hashes: Vec<u64> = candidate_hashes.iter().map(|&h| h as u64).collect();
    crate::find_similar_hashes(query_hash as u64, &u64_hashes, max_distance)
}

// ============================================================================
// Embedding Functions (feature-gated)
// ============================================================================

/// ELID encoding profile for vector embeddings.
///
/// Choose a profile based on your use case:
/// - `Mini128`: Fast 128-bit SimHash, good for similarity via Hamming distance
/// - `Morton10x10`: Z-order curve encoding, good for range queries
/// - `Hilbert10x10`: Hilbert curve encoding, best locality preservation
///
/// # JavaScript Example
///
/// ```javascript
/// import { ElidProfile, encodeElid } from 'elid';
///
/// const embedding = new Float64Array(768).fill(0.1);
/// const elid = encodeElid(embedding, ElidProfile.Mini128);
/// ```
#[cfg(feature = "embeddings")]
#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub enum ElidProfile {
    /// 128-bit SimHash (cosine similarity via Hamming distance)
    Mini128 = 0,
    /// Morton/Z-order curve encoding (10 dims, 10 bits each)
    Morton10x10 = 1,
    /// Hilbert curve encoding (10 dims, 10 bits each)
    Hilbert10x10 = 2,
}

#[cfg(feature = "embeddings")]
impl From<ElidProfile> for Profile {
    fn from(p: ElidProfile) -> Self {
        match p {
            ElidProfile::Mini128 => Profile::Mini128 {
                seed: 0x454c4944_53494d48, // "ELIDSIMH" standard seed
            },
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

/// Encode an embedding vector to an ELID string.
///
/// Converts a high-dimensional embedding (64-2048 dimensions) into a compact,
/// sortable identifier. The ELID preserves locality properties for efficient
/// similarity search.
///
/// # Parameters
///
/// - `embedding`: Float64 array of embedding values (64-2048 dimensions)
/// - `profile`: Encoding profile (Mini128, Morton10x10, or Hilbert10x10)
///
/// # Returns
///
/// A base32hex-encoded ELID string suitable for storage and comparison.
///
/// # JavaScript Example
///
/// ```javascript
/// import { encodeElid, ElidProfile } from 'elid';
///
/// // OpenAI embeddings are 1536 dimensions
/// const embedding = await getEmbedding("Hello world");
/// const elid = encodeElid(embedding, ElidProfile.Mini128);
/// console.log(elid); // "012345abcdef..."
/// ```
#[cfg(feature = "embeddings")]
#[wasm_bindgen(js_name = encodeElid)]
pub fn encode_elid(embedding: &[f64], profile: ElidProfile) -> Result<String, JsValue> {
    // Convert f64 to f32 (JS uses f64 for all numbers)
    let embedding_f32: Vec<f32> = embedding.iter().map(|&x| x as f32).collect();

    embeddings::encode(&embedding_f32, &Profile::from(profile))
        .map(|elid| elid.to_string())
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Decode an ELID string to raw bytes.
///
/// Returns the raw byte representation of an ELID, including the header
/// and payload bytes. Useful for custom processing or debugging.
///
/// # Parameters
///
/// - `elid_str`: A valid ELID string (base32hex encoded)
///
/// # Returns
///
/// A Uint8Array containing the raw bytes (header + payload).
///
/// # JavaScript Example
///
/// ```javascript
/// import { decodeElid } from 'elid';
///
/// const bytes = decodeElid("012345abcdef...");
/// console.log(bytes); // Uint8Array [...]
/// ```
#[cfg(feature = "embeddings")]
#[wasm_bindgen(js_name = decodeElid)]
pub fn decode_elid(elid_str: String) -> Result<Vec<u8>, JsValue> {
    // First validate and create the Elid type
    let elid =
        embeddings::Elid::from_string(elid_str).map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Then decode to bytes
    embeddings::decode(&elid).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Compute the Hamming distance between two ELID strings.
///
/// Returns the number of differing bits between two Mini128 ELIDs.
/// This distance is proportional to the angular distance between the
/// original embeddings (lower = more similar).
///
/// # Requirements
///
/// Both ELIDs must use the Mini128 profile.
///
/// # Parameters
///
/// - `elid1`: First ELID string
/// - `elid2`: Second ELID string
///
/// # Returns
///
/// Hamming distance (0-128). 0 means identical, 128 means completely different.
///
/// # JavaScript Example
///
/// ```javascript
/// import { encodeElid, elidHammingDistance, ElidProfile } from 'elid';
///
/// const elid1 = encodeElid(embedding1, ElidProfile.Mini128);
/// const elid2 = encodeElid(embedding2, ElidProfile.Mini128);
///
/// const distance = elidHammingDistance(elid1, elid2);
/// if (distance < 20) {
///     console.log("Very similar embeddings!");
/// }
/// ```
#[cfg(feature = "embeddings")]
#[wasm_bindgen(js_name = elidHammingDistance)]
pub fn elid_hamming_distance_wasm(elid1: String, elid2: String) -> Result<u32, JsValue> {
    // Parse ELID strings
    let elid_a =
        embeddings::Elid::from_string(elid1).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let elid_b =
        embeddings::Elid::from_string(elid2).map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Compute Hamming distance
    embeddings::hamming_distance(&elid_a, &elid_b).map_err(|e| JsValue::from_str(&e.to_string()))
}

// ============================================================================
// FullVector Encoding Functions (feature-gated)
// ============================================================================

/// Precision options for full vector encoding.
///
/// Controls how many bits are used to represent each dimension value.
/// Higher precision means more accurate reconstruction but larger output.
///
/// # JavaScript Example
///
/// ```javascript
/// import { ElidVectorPrecision, encodeElidWithPrecision } from 'elid';
///
/// const embedding = new Float64Array(768).fill(0.1);
/// // Full32 = lossless, Half16 = smaller with minimal error
/// ```
#[cfg(feature = "embeddings")]
#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub enum ElidVectorPrecision {
    /// Full 32-bit float (lossless, 4 bytes per dimension)
    Full32 = 0,
    /// 16-bit half-precision float (2 bytes per dimension)
    Half16 = 1,
    /// 8-bit quantized (1 byte per dimension, ~1% error)
    Quant8 = 2,
}

#[cfg(feature = "embeddings")]
impl From<ElidVectorPrecision> for VectorPrecision {
    fn from(p: ElidVectorPrecision) -> Self {
        match p {
            ElidVectorPrecision::Full32 => VectorPrecision::Full32,
            ElidVectorPrecision::Half16 => VectorPrecision::Half16,
            ElidVectorPrecision::Quant8 => VectorPrecision::Quant8,
        }
    }
}

/// Dimension handling mode for full vector encoding.
///
/// Controls whether to preserve original dimensions, reduce them,
/// or project to a common space for cross-dimensional comparison.
///
/// # JavaScript Example
///
/// ```javascript
/// import { ElidDimensionMode, encodeElidFullVector } from 'elid';
///
/// // Preserve all dimensions
/// // Reduce to fewer dimensions for smaller output
/// // Common space for comparing different-sized embeddings
/// ```
#[cfg(feature = "embeddings")]
#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub enum ElidDimensionMode {
    /// Preserve all original dimensions (no projection)
    Preserve = 0,
    /// Reduce dimensions using random projection
    Reduce = 1,
    /// Project to common space for cross-dimensional comparison
    Common = 2,
}

/// Encode an embedding using lossless full vector encoding.
///
/// Preserves the exact embedding values (32-bit float precision) and all dimensions.
/// This produces the largest output but allows exact reconstruction.
///
/// # Parameters
///
/// - `embedding`: Float64 array of embedding values (64-2048 dimensions)
///
/// # Returns
///
/// A base32hex-encoded ELID string that can be decoded back to the original embedding.
///
/// # JavaScript Example
///
/// ```javascript
/// import { encodeElidLossless, decodeElidToEmbedding } from 'elid';
///
/// const embedding = new Float64Array(768).fill(0.1);
/// const elid = encodeElidLossless(embedding);
///
/// // Later, recover the exact embedding
/// const recovered = decodeElidToEmbedding(elid);
/// // recovered is identical to embedding
/// ```
#[cfg(feature = "embeddings")]
#[wasm_bindgen(js_name = encodeElidLossless)]
pub fn encode_elid_lossless(embedding: &[f64]) -> Result<String, JsValue> {
    let embedding_f32: Vec<f32> = embedding.iter().map(|&x| x as f32).collect();
    let profile = Profile::lossless();

    embeddings::encode(&embedding_f32, &profile)
        .map(|elid| elid.to_string())
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Encode an embedding with percentage-based compression.
///
/// The retention percentage (0.0-1.0) controls how much information is preserved:
/// - 1.0 = lossless (Full32 precision, all dimensions)
/// - 0.5 = half precision and/or half dimensions
/// - 0.25 = quarter precision and/or quarter dimensions
///
/// The algorithm optimizes for dimension reduction first (which preserves
/// more geometric relationships) before reducing precision.
///
/// # Parameters
///
/// - `embedding`: Float64 array of embedding values (64-2048 dimensions)
/// - `retention_pct`: Information retention percentage (0.0-1.0)
///
/// # Returns
///
/// A base32hex-encoded ELID string.
///
/// # JavaScript Example
///
/// ```javascript
/// import { encodeElidCompressed } from 'elid';
///
/// const embedding = new Float64Array(768).fill(0.1);
///
/// // 50% retention - good balance of size and fidelity
/// const elid = encodeElidCompressed(embedding, 0.5);
///
/// // 25% retention - smaller but less accurate
/// const smallElid = encodeElidCompressed(embedding, 0.25);
/// ```
#[cfg(feature = "embeddings")]
#[wasm_bindgen(js_name = encodeElidCompressed)]
pub fn encode_elid_compressed(embedding: &[f64], retention_pct: f64) -> Result<String, JsValue> {
    let embedding_f32: Vec<f32> = embedding.iter().map(|&x| x as f32).collect();
    let original_dims = embedding_f32.len() as u16;
    let profile = Profile::compressed(retention_pct as f32, original_dims);

    embeddings::encode(&embedding_f32, &profile)
        .map(|elid| elid.to_string())
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Encode an embedding with a maximum output string length constraint.
///
/// Calculates the optimal precision and dimension settings to fit within
/// the specified character limit while maximizing fidelity.
///
/// # Parameters
///
/// - `embedding`: Float64 array of embedding values (64-2048 dimensions)
/// - `max_chars`: Maximum output string length in characters
///
/// # Returns
///
/// A base32hex-encoded ELID string guaranteed to be <= max_chars in length.
///
/// # JavaScript Example
///
/// ```javascript
/// import { encodeElidMaxLength } from 'elid';
///
/// const embedding = new Float64Array(768).fill(0.1);
///
/// // Fit in 100 characters (e.g., for database column constraints)
/// const elid = encodeElidMaxLength(embedding, 100);
/// console.log(elid.length <= 100); // true
///
/// // Fit in 50 characters (more compression)
/// const shortElid = encodeElidMaxLength(embedding, 50);
/// ```
#[cfg(feature = "embeddings")]
#[wasm_bindgen(js_name = encodeElidMaxLength)]
pub fn encode_elid_max_length(embedding: &[f64], max_chars: usize) -> Result<String, JsValue> {
    let embedding_f32: Vec<f32> = embedding.iter().map(|&x| x as f32).collect();
    let original_dims = embedding_f32.len() as u16;
    let profile = Profile::max_length(max_chars, original_dims);

    embeddings::encode(&embedding_f32, &profile)
        .map(|elid| elid.to_string())
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Decode an ELID string back to an embedding vector.
///
/// Only works for ELIDs encoded with a FullVector profile (lossless,
/// compressed, or max_length). Returns null for non-reversible profiles
/// like Mini128, Morton, or Hilbert.
///
/// # Parameters
///
/// - `elid_str`: A valid ELID string (base32hex encoded)
///
/// # Returns
///
/// A Float64Array containing the decoded embedding, or null if the ELID
/// is not reversible.
///
/// Note: If dimension reduction was used during encoding, the decoded
/// embedding will be in the reduced dimension space, not the original.
///
/// # JavaScript Example
///
/// ```javascript
/// import { encodeElidLossless, decodeElidToEmbedding, isElidReversible } from 'elid';
///
/// const embedding = new Float64Array(768).fill(0.1);
/// const elid = encodeElidLossless(embedding);
///
/// if (isElidReversible(elid)) {
///     const recovered = decodeElidToEmbedding(elid);
///     console.log(recovered.length); // 768
/// }
/// ```
#[cfg(feature = "embeddings")]
#[wasm_bindgen(js_name = decodeElidToEmbedding)]
pub fn decode_elid_to_embedding(elid_str: String) -> Result<JsValue, JsValue> {
    let elid =
        embeddings::Elid::from_string(elid_str).map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Check if reversible first
    if !embeddings::is_reversible(&elid) {
        return Ok(JsValue::NULL);
    }

    // Decode to embedding
    let (values, _metadata) =
        embeddings::decode_to_embedding(&elid).map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Convert to Float64Array for JavaScript
    let f64_values: Vec<f64> = values.iter().map(|&x| x as f64).collect();
    let array = Float64Array::new_with_length(f64_values.len() as u32);
    for (i, &val) in f64_values.iter().enumerate() {
        array.set_index(i as u32, val);
    }

    Ok(array.into())
}

/// Check if an ELID can be decoded back to an embedding.
///
/// Returns true if the ELID was encoded with a FullVector profile
/// (lossless, compressed, or max_length), false otherwise.
///
/// # Parameters
///
/// - `elid_str`: A valid ELID string (base32hex encoded)
///
/// # Returns
///
/// `true` if decodeElidToEmbedding will return an embedding, `false` otherwise.
///
/// # JavaScript Example
///
/// ```javascript
/// import { encodeElid, encodeElidLossless, isElidReversible, ElidProfile } from 'elid';
///
/// const embedding = new Float64Array(768).fill(0.1);
///
/// // Mini128 is NOT reversible
/// const mini128Elid = encodeElid(embedding, ElidProfile.Mini128);
/// console.log(isElidReversible(mini128Elid)); // false
///
/// // Lossless IS reversible
/// const losslessElid = encodeElidLossless(embedding);
/// console.log(isElidReversible(losslessElid)); // true
/// ```
#[cfg(feature = "embeddings")]
#[wasm_bindgen(js_name = isElidReversible)]
pub fn is_elid_reversible(elid_str: String) -> Result<bool, JsValue> {
    let elid =
        embeddings::Elid::from_string(elid_str).map_err(|e| JsValue::from_str(&e.to_string()))?;

    Ok(embeddings::is_reversible(&elid))
}

/// Encode an embedding for cross-dimensional comparison.
///
/// Projects the embedding to a common dimension space, allowing comparison
/// between embeddings of different original dimensions (e.g., 256d vs 768d).
///
/// # Parameters
///
/// - `embedding`: Float64 array of embedding values (64-2048 dimensions)
/// - `common_dims`: Target dimension space (all vectors projected here)
///
/// # Returns
///
/// A base32hex-encoded ELID string.
///
/// # JavaScript Example
///
/// ```javascript
/// import { encodeElidCrossDimensional, decodeElidToEmbedding } from 'elid';
///
/// // Different sized embeddings from different models
/// const embedding256 = new Float64Array(256).fill(0.1);
/// const embedding768 = new Float64Array(768).fill(0.1);
///
/// // Project both to 128-dim common space
/// const elid1 = encodeElidCrossDimensional(embedding256, 128);
/// const elid2 = encodeElidCrossDimensional(embedding768, 128);
///
/// // Now they can be compared directly (both decode to 128 dims)
/// const dec1 = decodeElidToEmbedding(elid1);
/// const dec2 = decodeElidToEmbedding(elid2);
/// // Both have length 128
/// ```
#[cfg(feature = "embeddings")]
#[wasm_bindgen(js_name = encodeElidCrossDimensional)]
pub fn encode_elid_cross_dimensional(
    embedding: &[f64],
    common_dims: u16,
) -> Result<String, JsValue> {
    let embedding_f32: Vec<f32> = embedding.iter().map(|&x| x as f32).collect();
    let profile = Profile::cross_dimensional(common_dims);

    embeddings::encode(&embedding_f32, &profile)
        .map(|elid| elid.to_string())
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Convert an embedding vector directly to LSH bands.
///
/// Computes the 128-bit SimHash of the embedding and splits it into bands
/// for Locality-Sensitive Hashing (LSH) indexing in databases.
///
/// # Parameters
///
/// - `embedding`: Float64 array of embedding values (64-2048 dimensions)
/// - `num_bands`: Number of bands to split into (must be 1, 2, 4, 8, or 16)
/// - `seed`: Optional seed for deterministic hashing (defaults to standard ELID seed)
///
/// # Returns
///
/// An array of base32hex-encoded band strings. Returns an empty array if
/// `num_bands` is invalid.
///
/// # JavaScript Example
///
/// ```javascript
/// import { embeddingToBands } from 'elid';
///
/// const embedding = new Float64Array(768).fill(0.1);
///
/// // Split into 4 bands (32 bits each) - good balance for most use cases
/// const bands = embeddingToBands(embedding, 4);
/// console.log(bands.length); // 4
///
/// // Store bands in database for efficient OR queries:
/// // SELECT * FROM embeddings WHERE band0 = ? OR band1 = ? OR band2 = ? OR band3 = ?
///
/// // Use custom seed for different hash family
/// const bandsWithSeed = embeddingToBands(embedding, 4, 12345n);
/// ```
#[cfg(feature = "embeddings")]
#[wasm_bindgen(js_name = embeddingToBands)]
pub fn embedding_to_bands_wasm(embedding: &[f64], num_bands: u8, seed: Option<u64>) -> Vec<String> {
    // Convert f64 to f32 (JS uses f64 for all numbers)
    let embedding_f32: Vec<f32> = embedding.iter().map(|&x| x as f32).collect();

    // Use default seed if not provided (same as Mini128 default: "ELIDSIMH")
    let seed_value = seed.unwrap_or(0x454c4944_53494d48);

    embeddings::embedding_to_bands(&embedding_f32, num_bands, seed_value)
}

/// Split an existing Mini128 hash into LSH bands.
///
/// Takes a 128-bit hash (16 bytes) and splits it into bands for
/// Locality-Sensitive Hashing (LSH) indexing.
///
/// # Parameters
///
/// - `hash`: Uint8Array containing exactly 16 bytes (128-bit hash)
/// - `num_bands`: Number of bands to split into (must be 1, 2, 4, 8, or 16)
///
/// # Returns
///
/// An array of base32hex-encoded band strings.
///
/// # Throws
///
/// Throws an error if the hash is not exactly 16 bytes.
///
/// # JavaScript Example
///
/// ```javascript
/// import { mini128ToBands, encodeElid, decodeElid, ElidProfile } from 'elid';
///
/// // Get hash bytes from an existing Mini128 ELID
/// const embedding = new Float64Array(768).fill(0.1);
/// const elid = encodeElid(embedding, ElidProfile.Mini128);
/// const bytes = decodeElid(elid);
///
/// // Extract the 16-byte hash (skip header byte)
/// const hashBytes = bytes.slice(1, 17);
///
/// // Split into bands
/// const bands = mini128ToBands(hashBytes, 4);
/// console.log(bands.length); // 4
/// ```
#[cfg(feature = "embeddings")]
#[wasm_bindgen(js_name = mini128ToBands)]
pub fn mini128_to_bands_wasm(hash: &[u8], num_bands: u8) -> Result<Vec<String>, JsValue> {
    // Validate hash is exactly 16 bytes
    if hash.len() != 16 {
        return Err(JsValue::from_str(&format!(
            "Hash must be exactly 16 bytes, got {} bytes",
            hash.len()
        )));
    }

    // Convert slice to fixed-size array
    let hash_array: [u8; 16] = hash
        .try_into()
        .map_err(|_| JsValue::from_str("Failed to convert hash to 16-byte array"))?;

    Ok(embeddings::mini128_to_bands(&hash_array, num_bands))
}

/// Get metadata about a FullVector ELID.
///
/// Returns an object containing information about how the ELID was encoded,
/// including original dimensions, precision, and dimension mode.
///
/// # Parameters
///
/// - `elid_str`: A valid ELID string (base32hex encoded)
///
/// # Returns
///
/// An object with metadata fields, or null if not a FullVector ELID.
///
/// # JavaScript Example
///
/// ```javascript
/// import { encodeElidCompressed, getElidMetadata } from 'elid';
///
/// const embedding = new Float64Array(768).fill(0.1);
/// const elid = encodeElidCompressed(embedding, 0.5);
///
/// const meta = getElidMetadata(elid);
/// if (meta) {
///     console.log(meta.originalDims);  // 768
///     console.log(meta.encodedDims);   // depends on compression
///     console.log(meta.isLossless);    // false
/// }
/// ```
#[cfg(feature = "embeddings")]
#[wasm_bindgen(js_name = getElidMetadata)]
pub fn get_elid_metadata(elid_str: String) -> Result<JsValue, JsValue> {
    let elid =
        embeddings::Elid::from_string(elid_str).map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Check if reversible (FullVector)
    if !embeddings::is_reversible(&elid) {
        return Ok(JsValue::NULL);
    }

    // Decode to get metadata
    let (_values, metadata) =
        embeddings::decode_to_embedding(&elid).map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Build result object
    let result = Object::new();
    Reflect::set(
        &result,
        &"originalDims".into(),
        &JsValue::from(metadata.original_dims),
    )
    .unwrap();
    Reflect::set(
        &result,
        &"encodedDims".into(),
        &JsValue::from(metadata.encoded_dims),
    )
    .unwrap();
    Reflect::set(
        &result,
        &"isLossless".into(),
        &JsValue::from(metadata.is_lossless()),
    )
    .unwrap();
    Reflect::set(
        &result,
        &"hasDimensionReduction".into(),
        &JsValue::from(metadata.has_dimension_reduction()),
    )
    .unwrap();

    // Precision type as string
    let precision_str = match metadata.precision {
        VectorPrecision::Full32 => "Full32",
        VectorPrecision::Half16 => "Half16",
        VectorPrecision::Quant8 => "Quant8",
        VectorPrecision::Bits { bits } => {
            // Return with bit count
            Reflect::set(&result, &"precisionBits".into(), &JsValue::from(bits)).unwrap();
            "Bits"
        }
    };
    Reflect::set(
        &result,
        &"precision".into(),
        &JsValue::from_str(precision_str),
    )
    .unwrap();

    // Dimension mode as string
    let mode_str = match metadata.dimension_mode {
        DimensionMode::Preserve => "Preserve",
        DimensionMode::Reduce { .. } => "Reduce",
        DimensionMode::Common { .. } => "Common",
    };
    Reflect::set(
        &result,
        &"dimensionMode".into(),
        &JsValue::from_str(mode_str),
    )
    .unwrap();

    Ok(result.into())
}

// ===== Model Inference =====

// --- Remote model initialization (fetches from GitHub Releases) ---

/// Initialize the text embedding model by downloading from GitHub Releases.
///
/// Returns a Promise. Call this once before using `embedText()`.
/// Subsequent calls return immediately (idempotent).
///
/// # JavaScript Example
///
/// ```javascript
/// await initTextModel();
/// const embedding = embedText("Hello, world!");
/// console.log(embedding.length); // 256
/// ```
#[cfg(all(feature = "models-text", target_arch = "wasm32"))]
#[wasm_bindgen(js_name = initTextModel)]
pub async fn init_text_model() -> Result<(), JsError> {
    crate::models::text::init_text_model()
        .await
        .map_err(|e| JsError::new(&e.to_string()))
}

/// Embed text into a 256-dimensional vector using Model2Vec.
///
/// Call `initTextModel()` first to download the model, or use
/// `embedTextFromBytes()` to supply model data manually.
///
/// # JavaScript Example
///
/// ```javascript
/// await initTextModel();
/// const embedding = embedText("Hello, world!");
/// console.log(embedding.length); // 256
/// ```
#[cfg(feature = "models-text")]
#[wasm_bindgen(js_name = embedText)]
pub fn embed_text_simple(text: &str) -> Result<Vec<f32>, JsError> {
    crate::models::text::embed_text_cached(text).map_err(|e| JsError::new(&e.to_string()))
}

/// Initialize the image embedding model by downloading from GitHub Releases.
///
/// Returns a Promise. Call this once before using `embedImage()`.
///
/// # JavaScript Example
///
/// ```javascript
/// await initImageModel();
/// const embedding = embedImage(imageBytes);
/// console.log(embedding.length); // 1000
/// ```
#[cfg(all(feature = "models-image", target_arch = "wasm32"))]
#[wasm_bindgen(js_name = initImageModel)]
pub async fn init_image_model() -> Result<(), JsError> {
    crate::models::image::init_image_model()
        .await
        .map_err(|e| JsError::new(&e.to_string()))
}

/// Embed an image into a 1000-dimensional vector using MobileNetV3-Small.
///
/// Call `initImageModel()` first to download the model, or use
/// `embedImageFromBytes()` to supply model data manually.
///
/// # JavaScript Example
///
/// ```javascript
/// await initImageModel();
/// const embedding = embedImage(imageBytes);
/// console.log(embedding.length); // 1000
/// ```
#[cfg(feature = "models-image")]
#[wasm_bindgen(js_name = embedImage)]
pub fn embed_image_simple(image_bytes: &[u8]) -> Result<Vec<f32>, JsError> {
    crate::models::image::embed_image_cached(image_bytes).map_err(|e| JsError::new(&e.to_string()))
}

// --- Bytes-based API (manual model loading) ---

/// Embed text by passing model files as byte arrays.
///
/// Use this if you want to host models yourself instead of using `initTextModel()`.
#[cfg(feature = "models-text")]
#[wasm_bindgen(js_name = embedTextFromBytes)]
pub fn embed_text_from_bytes(
    text: &str,
    safetensors_bytes: &[u8],
    tokenizer_json: &[u8],
    config_json: &[u8],
) -> Result<Vec<f32>, JsError> {
    crate::models::embed_text_from_bytes(text, safetensors_bytes, tokenizer_json, config_json)
        .map_err(|e| JsError::new(&e.to_string()))
}

/// Embed an image by passing the ONNX model as a byte array.
///
/// Use this if you want to host models yourself instead of using `initImageModel()`.
#[cfg(feature = "models-image")]
#[wasm_bindgen(js_name = embedImageFromBytes)]
pub fn embed_image_from_bytes(image_bytes: &[u8], model_onnx: &[u8]) -> Result<Vec<f32>, JsError> {
    crate::models::embed_image_from_bytes(image_bytes, model_onnx)
        .map_err(|e| JsError::new(&e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_levenshtein() {
        assert_eq!(levenshtein("kitten", "sitting"), 3);
    }

    #[wasm_bindgen_test]
    fn test_normalized_levenshtein() {
        let sim = normalized_levenshtein("hello", "hello");
        assert_eq!(sim, 1.0);
    }

    #[wasm_bindgen_test]
    fn test_jaro_winkler() {
        let sim = jaro_winkler("martha", "marhta");
        assert!(sim > 0.9);
    }

    #[wasm_bindgen_test]
    fn test_best_match() {
        let score = best_match("hello", "hallo");
        assert!(score > 0.7);
    }
}
