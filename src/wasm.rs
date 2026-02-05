//! WebAssembly bindings for ELID
//!
//! This module provides JavaScript-friendly bindings for all ELID functions.
//! These bindings work in browsers, Node.js, Deno, and Bun.

use js_sys::{Object, Reflect};
use wasm_bindgen::prelude::*;

// Conditional imports for embeddings feature
#[cfg(feature = "embeddings")]
use crate::embeddings::{self, Profile};

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
