//! Python bindings for ELID using PyO3
//!
//! This module provides Python bindings for all ELID functions.

// PyO3 0.22 proc macros trigger false positive useless_conversion lints
// See: https://github.com/rust-lang/rust-clippy/issues/12039
#![allow(clippy::useless_conversion)]

use pyo3::prelude::*;

// Conditional imports for embeddings feature
#[cfg(feature = "embeddings")]
use crate::embeddings::{self, Profile as EmbedProfile};
#[cfg(feature = "embeddings")]
use numpy::PyReadonlyArray1;
#[cfg(feature = "embeddings")]
use pyo3::types::PyBytes;

/// Compute the Levenshtein distance between two strings.
///
/// Returns the minimum number of single-character edits needed to transform one string into another.
///
/// Args:
///     a (str): First string
///     b (str): Second string
///
/// Returns:
///     int: The Levenshtein distance
///
/// Example:
///     >>> import elid
///     >>> elid.levenshtein("kitten", "sitting")
///     3
#[pyfunction]
fn levenshtein(a: &str, b: &str) -> usize {
    crate::levenshtein(a, b)
}

/// Compute the normalized Levenshtein similarity between two strings.
///
/// Returns a value between 0.0 (completely different) and 1.0 (identical).
///
/// Args:
///     a (str): First string
///     b (str): Second string
///
/// Returns:
///     float: Similarity score between 0.0 and 1.0
///
/// Example:
///     >>> import elid
///     >>> elid.normalized_levenshtein("hello", "hallo")
///     0.8
#[pyfunction]
fn normalized_levenshtein(a: &str, b: &str) -> f64 {
    crate::normalized_levenshtein(a, b)
}

/// Compute the Jaro similarity between two strings.
///
/// Returns a value between 0.0 (completely different) and 1.0 (identical).
/// Particularly effective for short strings like names.
///
/// Args:
///     a (str): First string
///     b (str): Second string
///
/// Returns:
///     float: Similarity score between 0.0 and 1.0
///
/// Example:
///     >>> import elid
///     >>> elid.jaro("martha", "marhta")
///     0.944
#[pyfunction]
fn jaro(a: &str, b: &str) -> f64 {
    crate::jaro(a, b)
}

/// Compute the Jaro-Winkler similarity between two strings.
///
/// Returns a value between 0.0 (completely different) and 1.0 (identical).
/// Gives more favorable ratings to strings with common prefixes.
///
/// Args:
///     a (str): First string
///     b (str): Second string
///
/// Returns:
///     float: Similarity score between 0.0 and 1.0
///
/// Example:
///     >>> import elid
///     >>> elid.jaro_winkler("martha", "marhta")
///     0.961
#[pyfunction]
fn jaro_winkler(a: &str, b: &str) -> f64 {
    crate::jaro_winkler(a, b)
}

/// Compute the Hamming distance between two strings.
///
/// Returns the number of positions at which the characters differ.
/// Returns None if strings have different lengths.
///
/// Args:
///     a (str): First string
///     b (str): Second string
///
/// Returns:
///     Optional[int]: Hamming distance or None if lengths differ
///
/// Example:
///     >>> import elid
///     >>> elid.hamming("karolin", "kathrin")
///     3
///     >>> elid.hamming("hello", "world!")  # Returns None
#[pyfunction]
fn hamming(a: &str, b: &str) -> Option<usize> {
    crate::hamming(a, b)
}

/// Compute the OSA (Optimal String Alignment) distance between two strings.
///
/// Similar to Levenshtein but also considers transpositions as a single operation.
///
/// Args:
///     a (str): First string
///     b (str): Second string
///
/// Returns:
///     int: OSA distance
///
/// Example:
///     >>> import elid
///     >>> elid.osa_distance("ca", "ac")
///     1
#[pyfunction]
fn osa_distance(a: &str, b: &str) -> usize {
    crate::osa_distance(a, b)
}

/// Compute the best matching similarity between two strings.
///
/// Runs multiple algorithms and returns the highest score.
///
/// Args:
///     a (str): First string
///     b (str): Second string
///
/// Returns:
///     float: Best similarity score between 0.0 and 1.0
///
/// Example:
///     >>> import elid
///     >>> elid.best_match("hello", "hallo")
///     0.8
#[pyfunction]
fn best_match(a: &str, b: &str) -> f64 {
    crate::best_match(a, b)
}

/// Find the best match for a query string in a list of candidates.
///
/// Args:
///     query (str): Query string
///     candidates (List[str]): List of candidate strings
///
/// Returns:
///     dict: Dictionary with 'index' and 'score' keys
///
/// Example:
///     >>> import elid
///     >>> candidates = ["apple", "application", "apply"]
///     >>> result = elid.find_best_match("app", candidates)
///     >>> result
///     {'index': 0, 'score': 0.907}
#[pyfunction]
fn find_best_match(query: &str, candidates: Vec<String>, py: Python<'_>) -> PyResult<Py<pyo3::PyAny>> {
    let candidate_refs: Vec<&str> = candidates.iter().map(|s| s.as_str()).collect();
    let (idx, score) = crate::find_best_match(query, &candidate_refs);

    let dict = pyo3::types::PyDict::new(py);
    dict.set_item("index", idx)?;
    dict.set_item("score", score)?;
    Ok(dict.unbind().into())
}

/// Find all matches above a threshold score.
///
/// Args:
///     query (str): Query string
///     candidates (List[str]): List of candidate strings
///     threshold (float): Minimum similarity score (0.0 to 1.0)
///
/// Returns:
///     List[dict]: List of dictionaries with 'index' and 'score' keys
///
/// Example:
///     >>> import elid
///     >>> candidates = ["apple", "application", "apply", "banana"]
///     >>> matches = elid.find_matches_above_threshold("app", candidates, 0.5)
///     >>> matches
///     [{'index': 0, 'score': 0.907}, {'index': 1, 'score': 0.830}, ...]
#[pyfunction]
fn find_matches_above_threshold(
    query: &str,
    candidates: Vec<String>,
    threshold: f64,
    py: Python<'_>,
) -> PyResult<Py<pyo3::PyAny>> {
    let candidate_refs: Vec<&str> = candidates.iter().map(|s| s.as_str()).collect();
    let matches = crate::find_matches_above_threshold(query, &candidate_refs, threshold);

    let list = pyo3::types::PyList::empty(py);
    for (idx, score) in matches {
        let dict = pyo3::types::PyDict::new(py);
        dict.set_item("index", idx)?;
        dict.set_item("score", score)?;
        list.append(dict)?;
    }
    Ok(list.unbind().into())
}

/// Options for configuring string similarity algorithms.
///
/// Attributes:
///     case_sensitive (bool): Case-sensitive comparison (default: True)
///     trim_whitespace (bool): Trim whitespace before comparison (default: False)
///     prefix_scale (float): Prefix scale for Jaro-Winkler (default: 0.1, max: 0.25)
///
/// Example:
///     >>> import elid
///     >>> opts = elid.SimilarityOpts(case_sensitive=False, trim_whitespace=True)
///     >>> elid.levenshtein_with_opts("  HELLO  ", "hello", opts)
///     0
#[pyclass]
struct SimilarityOpts {
    #[pyo3(get, set)]
    case_sensitive: bool,
    #[pyo3(get, set)]
    trim_whitespace: bool,
    #[pyo3(get, set)]
    prefix_scale: f64,
}

#[pymethods]
impl SimilarityOpts {
    #[new]
    #[pyo3(signature = (case_sensitive=true, trim_whitespace=false, prefix_scale=0.1))]
    fn new(case_sensitive: bool, trim_whitespace: bool, prefix_scale: f64) -> Self {
        SimilarityOpts {
            case_sensitive,
            trim_whitespace,
            prefix_scale,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "SimilarityOpts(case_sensitive={}, trim_whitespace={}, prefix_scale={})",
            self.case_sensitive, self.trim_whitespace, self.prefix_scale
        )
    }
}

impl From<&SimilarityOpts> for crate::SimilarityOpts {
    fn from(opts: &SimilarityOpts) -> Self {
        crate::SimilarityOpts {
            case_sensitive: opts.case_sensitive,
            trim_whitespace: opts.trim_whitespace,
            prefix_scale: opts.prefix_scale,
        }
    }
}

/// Compute Levenshtein distance with custom options.
///
/// Args:
///     a (str): First string
///     b (str): Second string
///     opts (SimilarityOpts): Configuration options
///
/// Returns:
///     int: Levenshtein distance
///
/// Example:
///     >>> import elid
///     >>> opts = elid.SimilarityOpts(case_sensitive=False, trim_whitespace=True)
///     >>> elid.levenshtein_with_opts("  HELLO  ", "hello", opts)
///     0
#[pyfunction]
fn levenshtein_with_opts(a: &str, b: &str, opts: &SimilarityOpts) -> usize {
    let rust_opts = crate::SimilarityOpts::from(opts);
    crate::levenshtein_with_opts(a, b, &rust_opts)
}

/// Compute the SimHash fingerprint of a string.
///
/// Returns a 64-bit integer hash where similar strings produce similar numbers.
/// Use this for database queries by storing the hash.
///
/// Args:
///     text (str): Input string
///
/// Returns:
///     int: 64-bit hash value
///
/// Example:
///     >>> import elid
///     >>> hash1 = elid.simhash("iPhone 14")
///     >>> hash2 = elid.simhash("iPhone 15")
///     >>> hash3 = elid.simhash("Galaxy S23")
///     >>> # hash1 and hash2 will be numerically close
///     >>> # hash3 will be different
#[pyfunction]
fn simhash(text: &str) -> u64 {
    crate::simhash(text)
}

/// Compute the Hamming distance between two SimHash values.
///
/// Returns the number of differing bits. Lower values indicate higher similarity.
///
/// Args:
///     hash1 (int): First SimHash value
///     hash2 (int): Second SimHash value
///
/// Returns:
///     int: Hamming distance (0-64)
///
/// Example:
///     >>> import elid
///     >>> hash1 = elid.simhash("iPhone 14")
///     >>> hash2 = elid.simhash("iPhone 15")
///     >>> distance = elid.simhash_distance(hash1, hash2)
///     >>> distance  # Low number = similar
#[pyfunction]
fn simhash_distance(hash1: u64, hash2: u64) -> u32 {
    crate::simhash_distance(hash1, hash2)
}

/// Compute the normalized SimHash similarity between two strings.
///
/// Returns a value between 0.0 (completely different) and 1.0 (identical).
///
/// Args:
///     a (str): First string
///     b (str): Second string
///
/// Returns:
///     float: Similarity score between 0.0 and 1.0
///
/// Example:
///     >>> import elid
///     >>> similarity = elid.simhash_similarity("iPhone 14", "iPhone 15")
///     >>> similarity  # ~0.9 (very similar)
///     >>> similarity2 = elid.simhash_similarity("iPhone", "Galaxy")
///     >>> similarity2  # ~0.4 (different)
#[pyfunction]
fn simhash_similarity(a: &str, b: &str) -> f64 {
    crate::simhash_similarity(a, b)
}

/// Find all hashes within a given distance threshold.
///
/// Args:
///     query_hash (int): The query SimHash value
///     candidate_hashes (List[int]): List of candidate SimHash values
///     max_distance (int): Maximum Hamming distance threshold
///
/// Returns:
///     List[int]: Indices of candidates within the distance threshold
///
/// Example:
///     >>> import elid
///     >>> candidates = ["iPhone 14 Pro", "iPhone 13", "Galaxy S23"]
///     >>> hashes = [elid.simhash(s) for s in candidates]
///     >>> query_hash = elid.simhash("iPhone 14")
///     >>> matches = elid.find_similar_hashes(query_hash, hashes, 10)
///     >>> matches  # [0, 1] - indices of iPhone variants
#[pyfunction]
fn find_similar_hashes(
    query_hash: u64,
    candidate_hashes: Vec<u64>,
    max_distance: u32,
) -> Vec<usize> {
    crate::find_similar_hashes(query_hash, &candidate_hashes, max_distance)
}

// ============================================================================
// Embedding functions (feature-gated)
// ============================================================================

/// Encoding profile for embedding vectors.
///
/// Profiles determine how embeddings are transformed into compact identifiers.
///
/// Variants:
///     Mini128: 128-bit SimHash (default, fast cosine similarity via Hamming distance)
///     Morton10x10: Z-order curve encoding for database indexing
///     Hilbert10x10: Hilbert curve encoding for maximum locality preservation
///
/// Example:
///     >>> import elid
///     >>> profile = elid.Profile.Mini128
///     >>> elid_str = elid.encode(embedding, profile)
#[cfg(feature = "embeddings")]
#[pyclass]
#[derive(Clone, Copy, Debug)]
pub enum Profile {
    /// 128-bit SimHash encoding
    Mini128,
    /// Morton (Z-order) curve encoding with 10 dimensions x 10 bits
    Morton10x10,
    /// Hilbert curve encoding with 10 dimensions x 10 bits
    Hilbert10x10,
}

#[cfg(feature = "embeddings")]
impl From<Profile> for EmbedProfile {
    fn from(p: Profile) -> Self {
        match p {
            Profile::Mini128 => EmbedProfile::Mini128 {
                seed: 0x454c4944_53494d48, // Default "ELIDSIMH" seed
            },
            Profile::Morton10x10 => EmbedProfile::Morton10x10 {
                dims: 10,
                bits_per_dim: 10,
                transform_id: None,
            },
            Profile::Hilbert10x10 => EmbedProfile::Hilbert10x10 {
                dims: 10,
                bits_per_dim: 10,
                transform_id: None,
            },
        }
    }
}

/// Encode an embedding vector to an ELID string.
///
/// Converts a high-dimensional embedding vector into a compact, sortable identifier
/// using the specified profile. The resulting ELID preserves locality properties
/// for efficient similarity search.
///
/// Args:
///     embedding (numpy.ndarray): Input vector (f32, 64-2048 dimensions)
///     profile (Profile): Encoding strategy (Mini128, Morton10x10, or Hilbert10x10)
///
/// Returns:
///     str: Encoded ELID string
///
/// Raises:
///     ValueError: If embedding dimensions are invalid or values contain NaN/Inf
///
/// Example:
///     >>> import elid
///     >>> import numpy as np
///     >>> embedding = np.random.randn(768).astype(np.float32)
///     >>> elid_str = elid.encode(embedding, elid.Profile.Mini128)
///     >>> print(elid_str)  # e.g., "01a2b3c4d5e6f7g8h9i0..."
#[cfg(feature = "embeddings")]
#[pyfunction]
#[pyo3(name = "encode")]
fn encode_embedding(embedding: PyReadonlyArray1<f32>, profile: Profile) -> PyResult<String> {
    let slice = embedding.as_slice()?;
    embeddings::encode(slice, &EmbedProfile::from(profile))
        .map(|elid| elid.to_string())
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
}

/// Decode an ELID string to raw bytes.
///
/// Decodes a base32hex-encoded ELID string back to its raw byte representation.
/// This returns the header bytes + payload bytes.
///
/// Args:
///     elid_str (str): The ELID string to decode
///
/// Returns:
///     bytes: Raw bytes (header + payload)
///
/// Raises:
///     ValueError: If the ELID string contains invalid characters
///
/// Example:
///     >>> import elid
///     >>> raw_bytes = elid.decode("01a2b3c4d5e6f7...")
///     >>> print(len(raw_bytes))  # 18 for Mini128 (2 header + 16 payload)
#[cfg(feature = "embeddings")]
#[pyfunction]
#[pyo3(name = "decode")]
fn decode_elid<'py>(py: Python<'py>, elid_str: &str) -> PyResult<Bound<'py, PyBytes>> {
    // First create an Elid from the string
    let elid = embeddings::types::Elid::from_string(elid_str.to_string())
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

    // Then decode to bytes
    let bytes = embeddings::decode(&elid)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

    Ok(PyBytes::new(py, &bytes))
}

/// Compute Hamming distance between two ELID strings.
///
/// Returns the number of differing bits in the SimHash payloads of two ELIDs.
/// This distance is proportional to the angular distance between the original
/// embeddings. Both ELIDs must use the Mini128 profile.
///
/// Args:
///     elid1 (str): First ELID string
///     elid2 (str): Second ELID string
///
/// Returns:
///     int: Hamming distance (0-128)
///
/// Raises:
///     ValueError: If either ELID is invalid or uses a non-Mini128 profile
///
/// Example:
///     >>> import elid
///     >>> import numpy as np
///     >>> emb1 = np.random.randn(768).astype(np.float32)
///     >>> emb2 = emb1 + np.random.randn(768).astype(np.float32) * 0.1  # Similar
///     >>> elid1 = elid.encode(emb1, elid.Profile.Mini128)
///     >>> elid2 = elid.encode(emb2, elid.Profile.Mini128)
///     >>> distance = elid.elid_hamming_distance(elid1, elid2)
///     >>> print(f"Distance: {distance}")  # Low number = similar embeddings
#[cfg(feature = "embeddings")]
#[pyfunction]
fn elid_hamming_distance(elid1: &str, elid2: &str) -> PyResult<u32> {
    // Create Elid objects from strings
    let a = embeddings::types::Elid::from_string(elid1.to_string())
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
    let b = embeddings::types::Elid::from_string(elid2.to_string())
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

    embeddings::hamming_distance(&a, &b)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
}

/// ELID - Efficient Levenshtein and String Similarity Library
///
/// A fast library for computing various string similarity metrics.
#[pymodule]
fn elid(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // String similarity functions
    m.add_function(wrap_pyfunction!(levenshtein, m)?)?;
    m.add_function(wrap_pyfunction!(normalized_levenshtein, m)?)?;
    m.add_function(wrap_pyfunction!(jaro, m)?)?;
    m.add_function(wrap_pyfunction!(jaro_winkler, m)?)?;
    m.add_function(wrap_pyfunction!(hamming, m)?)?;
    m.add_function(wrap_pyfunction!(osa_distance, m)?)?;
    m.add_function(wrap_pyfunction!(best_match, m)?)?;
    m.add_function(wrap_pyfunction!(find_best_match, m)?)?;
    m.add_function(wrap_pyfunction!(find_matches_above_threshold, m)?)?;
    m.add_function(wrap_pyfunction!(levenshtein_with_opts, m)?)?;
    m.add_function(wrap_pyfunction!(simhash, m)?)?;
    m.add_function(wrap_pyfunction!(simhash_distance, m)?)?;
    m.add_function(wrap_pyfunction!(simhash_similarity, m)?)?;
    m.add_function(wrap_pyfunction!(find_similar_hashes, m)?)?;
    m.add_class::<SimilarityOpts>()?;

    // Embedding functions (feature-gated)
    #[cfg(feature = "embeddings")]
    {
        m.add_function(wrap_pyfunction!(encode_embedding, m)?)?;
        m.add_function(wrap_pyfunction!(decode_elid, m)?)?;
        m.add_function(wrap_pyfunction!(elid_hamming_distance, m)?)?;
        m.add_class::<Profile>()?;
    }

    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    Ok(())
}
