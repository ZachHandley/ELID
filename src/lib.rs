//! # ELID - Efficient Levenshtein and other string similarity metrics
//!
//! A fast, zero-dependency library for computing string similarity metrics.
//!
//! ## Features
//!
//! - **Levenshtein Distance**: Classic edit distance algorithm
//! - **Normalized Levenshtein**: Returns similarity as a value between 0.0 and 1.0
//! - **Jaro-Winkler Similarity**: Better for short strings like names
//! - **Hamming Distance**: For equal-length strings
//! - **Optimal String Alignment (OSA)**: Levenshtein with transpositions
//!
//! ## Example
//!
//! ```rust
//! use elid::{levenshtein, normalized_levenshtein, jaro_winkler};
//!
//! let distance = levenshtein("kitten", "sitting");
//! assert_eq!(distance, 3);
//!
//! let similarity = normalized_levenshtein("kitten", "sitting");
//! assert!(similarity > 0.5 && similarity < 0.7);
//!
//! let jw_similarity = jaro_winkler("martha", "marhta");
//! assert!(jw_similarity > 0.9);
//! ```

#![deny(missing_docs)]
#![deny(unsafe_code)]

mod levenshtein;
mod jaro_winkler;
mod hamming;
mod osa;
mod common;

#[cfg(feature = "wasm")]
pub mod wasm;

#[cfg(feature = "python")]
pub mod python;

pub use levenshtein::{levenshtein, normalized_levenshtein, levenshtein_with_opts};
pub use jaro_winkler::{jaro, jaro_winkler, jaro_winkler_with_prefix};
pub use hamming::{hamming, normalized_hamming};
pub use osa::{osa_distance, normalized_osa};

/// Options for configuring string similarity algorithms
#[derive(Debug, Clone, Copy)]
pub struct SimilarityOpts {
    /// Case-sensitive comparison (default: true)
    pub case_sensitive: bool,
    /// Trim whitespace before comparison (default: false)
    pub trim_whitespace: bool,
    /// Prefix scale for Jaro-Winkler (default: 0.1, max: 0.25)
    pub prefix_scale: f64,
}

impl Default for SimilarityOpts {
    fn default() -> Self {
        Self {
            case_sensitive: true,
            trim_whitespace: false,
            prefix_scale: 0.1,
        }
    }
}

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
pub fn find_matches_above_threshold(query: &str, candidates: &[&str], threshold: f64) -> Vec<(usize, f64)> {
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

// Re-export Python module when python feature is enabled
#[cfg(feature = "python")]
pub use python::elid as python_module;

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
