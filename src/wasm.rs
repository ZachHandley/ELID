//! WebAssembly bindings for ELID
//!
//! This module provides JavaScript-friendly bindings for all ELID functions.
//! These bindings work in browsers, Node.js, Deno, and Bun.

use wasm_bindgen::prelude::*;

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
    crate::levenshtein::levenshtein(a, b)
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
    crate::levenshtein::normalized_levenshtein(a, b)
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
    crate::jaro_winkler::jaro(a, b)
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
    crate::jaro_winkler::jaro_winkler(a, b)
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
    crate::hamming::hamming(a, b)
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
    crate::osa::osa_distance(a, b)
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
pub fn find_best_match(query: &str, candidates: Vec<String>) -> JsValue {
    let candidate_refs: Vec<&str> = candidates.iter().map(|s| s.as_str()).collect();
    let (idx, score) = crate::find_best_match(query, &candidate_refs);

    serde_wasm_bindgen::to_value(&serde_json::json!({
        "index": idx,
        "score": score
    })).unwrap()
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
pub fn find_matches_above_threshold(query: &str, candidates: Vec<String>, threshold: f64) -> JsValue {
    let candidate_refs: Vec<&str> = candidates.iter().map(|s| s.as_str()).collect();
    let matches = crate::find_matches_above_threshold(query, &candidate_refs, threshold);

    let results: Vec<_> = matches
        .into_iter()
        .map(|(idx, score)| serde_json::json!({
            "index": idx,
            "score": score
        }))
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
    crate::levenshtein::levenshtein_with_opts(a, b, &rust_opts)
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
