//! # ELID - Embedding Locality IDentifier
//!
//! ELID enables vector search without a vector store by encoding high-dimensional embeddings
//! into sortable string IDs that preserve locality. Similar vectors produce similar IDs,
//! allowing you to use standard database indexes for similarity search.
//!
//! ELID also includes a complete suite of fast, zero-dependency string similarity algorithms.
//!
//! ## Feature Sets
//!
//! ### Embedding Encoding (`embeddings` feature)
//!
//! Convert embeddings from any ML model into compact, sortable identifiers:
//!
//! - **Mini128**: 128-bit SimHash using signed random projections (fast, Hamming distance)
//! - **Morton10x10**: Z-order curve encoding (database range queries)
//! - **Hilbert10x10**: Hilbert curve encoding (maximum locality preservation)
//!
//! ### String Similarity (`strings` feature, default)
//!
//! - **Levenshtein Distance**: Classic edit distance algorithm
//! - **Normalized Levenshtein**: Returns similarity as a value between 0.0 and 1.0
//! - **Jaro-Winkler Similarity**: Better for short strings like names
//! - **Hamming Distance**: For equal-length strings
//! - **Optimal String Alignment (OSA)**: Levenshtein with transpositions
//! - **SimHash**: Locality-sensitive hashing for string similarity queries
//!
//! ## Feature Flags
//!
//! - `strings` (default): Zero-dependency string similarity algorithms
//! - `embeddings`: Vector encoding with Mini128, Morton, and Hilbert profiles
//! - `wasm` / `wasm-embeddings`: WebAssembly bindings
//! - `python` / `python-embeddings`: Python bindings via PyO3
//! - `ffi`: C FFI bindings
//!
//! ## Embedding Encoding Example
//!
//! ```rust,ignore
//! use elid::embeddings::{encode, Profile, hamming_distance};
//!
//! // Get embeddings from your ML model
//! let embedding1 = model.embed("Hello, world!")?;
//! let embedding2 = model.embed("Hello, universe!")?;
//!
//! // Encode to sortable ELIDs
//! let profile = Profile::default(); // Mini128
//! let elid1 = encode(&embedding1, &profile)?;
//! let elid2 = encode(&embedding2, &profile)?;
//!
//! // Compare via Hamming distance (lower = more similar)
//! let distance = hamming_distance(&elid1, &elid2)?;
//! ```
//!
//! ## String Similarity Example
//!
//! ```rust
//! use elid::{levenshtein, normalized_levenshtein, jaro_winkler, simhash, simhash_similarity};
//!
//! let distance = levenshtein("kitten", "sitting");
//! assert_eq!(distance, 3);
//!
//! let similarity = normalized_levenshtein("kitten", "sitting");
//! assert!(similarity > 0.5 && similarity < 0.7);
//!
//! let jw_similarity = jaro_winkler("martha", "marhta");
//! assert!(jw_similarity > 0.9);
//!
//! // SimHash for numeric database queries
//! let hash1 = simhash("iPhone 14");
//! let hash2 = simhash("iPhone 15");
//! let sim = simhash_similarity("iPhone 14", "iPhone 15");
//! assert!(sim > 0.8);
//! ```

#![deny(missing_docs)]
#![cfg_attr(not(feature = "ffi"), deny(unsafe_code))]

mod strings;

#[cfg(feature = "embeddings")]
pub mod embeddings;

#[cfg(feature = "wasm")]
pub mod wasm;

#[cfg(feature = "python")]
pub mod python;

#[cfg(feature = "ffi")]
pub mod ffi;

// Re-export everything from strings for backwards compatibility
pub use strings::{
    find_similar_hashes, hamming, jaro, jaro_winkler, jaro_winkler_with_prefix, levenshtein,
    levenshtein_with_opts, normalized_hamming, normalized_levenshtein, normalized_osa,
    osa_distance, simhash, simhash_distance, simhash_similarity, SimilarityOpts,
};

/// Compute the best matching similarity between two strings using multiple algorithms
/// and return the highest score.
///
/// This function runs multiple algorithms and returns the best result, useful when
/// you're not sure which algorithm will work best for your data.
///
/// # Example
///
/// ```rust
/// use elid::best_match;
///
/// let score = best_match("hello", "hallo");
/// assert!(score > 0.7);
/// ```
pub fn best_match(a: &str, b: &str) -> f64 {
    let lev = normalized_levenshtein(a, b);
    let jw = jaro_winkler(a, b);
    lev.max(jw)
}

/// Find the best match for a query string in a list of candidates.
///
/// Returns the index and similarity score of the best match.
///
/// # Example
///
/// ```rust
/// use elid::find_best_match;
///
/// let candidates = vec!["apple", "application", "apply"];
/// let (idx, score) = find_best_match("app", &candidates);
/// assert!(score > 0.5);
/// ```
pub fn find_best_match(query: &str, candidates: &[&str]) -> (usize, f64) {
    candidates
        .iter()
        .enumerate()
        .map(|(i, candidate)| (i, best_match(query, candidate)))
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or((0, 0.0))
}

/// Find all matches above a threshold score.
///
/// Returns a vector of (index, score) tuples for all candidates above the threshold.
///
/// # Example
///
/// ```rust
/// use elid::find_matches_above_threshold;
///
/// let candidates = vec!["apple", "application", "apply", "banana"];
/// let matches = find_matches_above_threshold("app", &candidates, 0.5);
/// assert!(matches.len() >= 2); // Should match at least "apple" and "apply"
/// ```
pub fn find_matches_above_threshold(
    query: &str,
    candidates: &[&str],
    threshold: f64,
) -> Vec<(usize, f64)> {
    candidates
        .iter()
        .enumerate()
        .filter_map(|(i, candidate)| {
            let score = best_match(query, candidate);
            if score >= threshold {
                Some((i, score))
            } else {
                None
            }
        })
        .collect()
}

// Python module is defined in python.rs and exported via #[pymodule]

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_best_match() {
        let score = best_match("hello", "hallo");
        assert!(score > 0.7);
    }

    #[test]
    fn test_find_best_match() {
        let candidates = vec!["apple", "application", "apply"];
        let (idx, score) = find_best_match("app", &candidates);
        assert!(score > 0.5);
        assert!(candidates[idx].starts_with("app"));
    }

    #[test]
    fn test_find_matches_above_threshold() {
        let candidates = vec!["apple", "application", "apply", "banana"];
        let matches = find_matches_above_threshold("app", &candidates, 0.5);
        assert!(matches.len() >= 2);
    }
}
