//! Python bindings for ELID using PyO3
//!
//! This module provides Python bindings for all ELID functions.

// PyO3 0.22 proc macros trigger false positive useless_conversion lints
// See: https://github.com/rust-lang/rust-clippy/issues/12039
#![allow(clippy::useless_conversion)]

use pyo3::prelude::*;

// Conditional imports for embeddings feature
#[cfg(feature = "embeddings")]
use crate::embeddings::{
    self, DimensionMode as EmbedDimensionMode, Profile as EmbedProfile,
    VectorPrecision as EmbedVectorPrecision,
};
#[cfg(feature = "embeddings")]
use numpy::{PyArray1, PyReadonlyArray1};
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
fn find_best_match(
    query: &str,
    candidates: Vec<String>,
    py: Python<'_>,
) -> PyResult<Py<pyo3::PyAny>> {
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

// ============================================================================
// FullVector Encoding Types and Functions (feature-gated)
// ============================================================================

/// Precision options for full vector encoding.
///
/// Controls how many bits are used to represent each dimension value.
/// Higher precision means more accurate reconstruction but larger output.
///
/// Variants:
///     Full32: Full 32-bit float (lossless, 4 bytes per dimension)
///     Half16: 16-bit half-precision float (2 bytes per dimension)
///     Quant8: 8-bit quantized (1 byte per dimension, ~1% error)
///
/// Example:
///     >>> import elid
///     >>> prec = elid.VectorPrecision.Full32  # Lossless
///     >>> prec = elid.VectorPrecision.Half16  # Good balance
///     >>> prec = elid.VectorPrecision.Quant8  # Smallest
#[cfg(feature = "embeddings")]
#[pyclass]
#[derive(Clone, Copy, Debug)]
pub enum VectorPrecision {
    /// Full 32-bit float (lossless)
    Full32,
    /// 16-bit half-precision float
    Half16,
    /// 8-bit quantized (~1% error)
    Quant8,
}

#[cfg(feature = "embeddings")]
impl From<VectorPrecision> for EmbedVectorPrecision {
    fn from(p: VectorPrecision) -> Self {
        match p {
            VectorPrecision::Full32 => EmbedVectorPrecision::Full32,
            VectorPrecision::Half16 => EmbedVectorPrecision::Half16,
            VectorPrecision::Quant8 => EmbedVectorPrecision::Quant8,
        }
    }
}

/// Dimension handling mode for full vector encoding.
///
/// Controls whether to preserve original dimensions, reduce them,
/// or project to a common space for cross-dimensional comparison.
///
/// Variants:
///     Preserve: Keep all original dimensions (no projection)
///     Reduce: Reduce dimensions using random projection
///     Common: Project to common space for cross-dimensional comparison
///
/// Example:
///     >>> import elid
///     >>> mode = elid.DimensionMode.Preserve  # Keep all dims
///     >>> mode = elid.DimensionMode.Reduce    # Reduce for smaller output
///     >>> mode = elid.DimensionMode.Common    # Cross-dimensional comparison
#[cfg(feature = "embeddings")]
#[pyclass]
#[derive(Clone, Copy, Debug)]
pub enum DimensionMode {
    /// Preserve all original dimensions
    Preserve,
    /// Reduce dimensions using random projection
    Reduce,
    /// Project to common space for cross-dimensional comparison
    Common,
}

/// Encode an embedding using lossless full vector encoding.
///
/// Preserves the exact embedding values (32-bit float precision) and all dimensions.
/// This produces the largest output but allows exact reconstruction.
///
/// Args:
///     embedding (numpy.ndarray): Input vector (f32, 64-2048 dimensions)
///
/// Returns:
///     str: Encoded ELID string that can be decoded back to the original embedding
///
/// Raises:
///     ValueError: If embedding dimensions are invalid or values contain NaN/Inf
///
/// Example:
///     >>> import elid
///     >>> import numpy as np
///     >>> embedding = np.random.randn(768).astype(np.float32)
///     >>> elid_str = elid.encode_lossless(embedding)
///     >>> recovered = elid.decode_to_embedding(elid_str)
///     >>> np.allclose(embedding, recovered)  # True
#[cfg(feature = "embeddings")]
#[pyfunction]
fn encode_lossless(embedding: PyReadonlyArray1<f32>) -> PyResult<String> {
    let slice = embedding.as_slice()?;
    let profile = EmbedProfile::lossless();

    embeddings::encode(slice, &profile)
        .map(|elid| elid.to_string())
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
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
/// Args:
///     embedding (numpy.ndarray): Input vector (f32, 64-2048 dimensions)
///     retention_pct (float): Information retention percentage (0.0-1.0)
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
///     >>> elid_50 = elid.encode_compressed(embedding, 0.5)   # 50% retention
///     >>> elid_25 = elid.encode_compressed(embedding, 0.25)  # 25% retention
///     >>> len(elid_25) < len(elid_50)  # True (smaller output)
#[cfg(feature = "embeddings")]
#[pyfunction]
fn encode_compressed(embedding: PyReadonlyArray1<f32>, retention_pct: f32) -> PyResult<String> {
    let slice = embedding.as_slice()?;
    let original_dims = slice.len() as u16;
    let profile = EmbedProfile::compressed(retention_pct, original_dims);

    embeddings::encode(slice, &profile)
        .map(|elid| elid.to_string())
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
}

/// Encode an embedding with a maximum output string length constraint.
///
/// Calculates the optimal precision and dimension settings to fit within
/// the specified character limit while maximizing fidelity.
///
/// Args:
///     embedding (numpy.ndarray): Input vector (f32, 64-2048 dimensions)
///     max_chars (int): Maximum output string length in characters
///
/// Returns:
///     str: Encoded ELID string guaranteed to be <= max_chars in length
///
/// Raises:
///     ValueError: If embedding dimensions are invalid or values contain NaN/Inf
///
/// Example:
///     >>> import elid
///     >>> import numpy as np
///     >>> embedding = np.random.randn(768).astype(np.float32)
///     >>> elid_str = elid.encode_max_length(embedding, 100)
///     >>> len(elid_str) <= 100  # True
#[cfg(feature = "embeddings")]
#[pyfunction]
fn encode_max_length(embedding: PyReadonlyArray1<f32>, max_chars: usize) -> PyResult<String> {
    let slice = embedding.as_slice()?;
    let original_dims = slice.len() as u16;
    let profile = EmbedProfile::max_length(max_chars, original_dims);

    embeddings::encode(slice, &profile)
        .map(|elid| elid.to_string())
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
}

/// Decode an ELID string back to an embedding vector.
///
/// Only works for ELIDs encoded with a FullVector profile (lossless,
/// compressed, or max_length). Returns None for non-reversible profiles
/// like Mini128, Morton, or Hilbert.
///
/// Args:
///     elid_str (str): A valid ELID string (base32hex encoded)
///
/// Returns:
///     Optional[numpy.ndarray]: Decoded embedding as f32 array, or None if not reversible
///
/// Note:
///     If dimension reduction was used during encoding, the decoded embedding
///     will be in the reduced dimension space, not the original.
///
/// Example:
///     >>> import elid
///     >>> import numpy as np
///     >>> embedding = np.random.randn(768).astype(np.float32)
///     >>> elid_str = elid.encode_lossless(embedding)
///     >>> recovered = elid.decode_to_embedding(elid_str)
///     >>> recovered is not None  # True
///     >>> np.allclose(embedding, recovered)  # True
#[cfg(feature = "embeddings")]
#[pyfunction]
fn decode_to_embedding<'py>(
    py: Python<'py>,
    elid_str: &str,
) -> PyResult<Option<Bound<'py, PyArray1<f32>>>> {
    let elid = embeddings::types::Elid::from_string(elid_str.to_string())
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

    // Check if reversible first
    if !embeddings::is_reversible(&elid) {
        return Ok(None);
    }

    // Decode to embedding
    let (values, _metadata) = embeddings::decode_to_embedding(&elid)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

    // Convert to numpy array
    let array = PyArray1::from_vec(py, values);
    Ok(Some(array))
}

/// Check if an ELID can be decoded back to an embedding.
///
/// Returns True if the ELID was encoded with a FullVector profile
/// (lossless, compressed, or max_length), False otherwise.
///
/// Args:
///     elid_str (str): A valid ELID string (base32hex encoded)
///
/// Returns:
///     bool: True if decode_to_embedding will return an embedding
///
/// Raises:
///     ValueError: If the ELID string is invalid
///
/// Example:
///     >>> import elid
///     >>> import numpy as np
///     >>> embedding = np.random.randn(768).astype(np.float32)
///     >>>
///     >>> # Mini128 is NOT reversible
///     >>> mini_elid = elid.encode(embedding, elid.Profile.Mini128)
///     >>> elid.is_reversible(mini_elid)  # False
///     >>>
///     >>> # Lossless IS reversible
///     >>> lossless_elid = elid.encode_lossless(embedding)
///     >>> elid.is_reversible(lossless_elid)  # True
#[cfg(feature = "embeddings")]
#[pyfunction]
fn is_reversible(elid_str: &str) -> PyResult<bool> {
    let elid = embeddings::types::Elid::from_string(elid_str.to_string())
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

    Ok(embeddings::is_reversible(&elid))
}

/// Encode an embedding for cross-dimensional comparison.
///
/// Projects the embedding to a common dimension space, allowing comparison
/// between embeddings of different original dimensions (e.g., 256d vs 768d).
///
/// Args:
///     embedding (numpy.ndarray): Input vector (f32, 64-2048 dimensions)
///     common_dims (int): Target dimension space (all vectors projected here)
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
///     >>> # Different sized embeddings from different models
///     >>> emb_256 = np.random.randn(256).astype(np.float32)
///     >>> emb_768 = np.random.randn(768).astype(np.float32)
///     >>>
///     >>> # Project both to 128-dim common space
///     >>> elid1 = elid.encode_cross_dimensional(emb_256, 128)
///     >>> elid2 = elid.encode_cross_dimensional(emb_768, 128)
///     >>>
///     >>> # Now they can be compared directly
///     >>> dec1 = elid.decode_to_embedding(elid1)
///     >>> dec2 = elid.decode_to_embedding(elid2)
///     >>> dec1.shape == dec2.shape  # True (both 128,)
#[cfg(feature = "embeddings")]
#[pyfunction]
fn encode_cross_dimensional(
    embedding: PyReadonlyArray1<f32>,
    common_dims: u16,
) -> PyResult<String> {
    let slice = embedding.as_slice()?;
    let profile = EmbedProfile::cross_dimensional(common_dims);

    embeddings::encode(slice, &profile)
        .map(|elid| elid.to_string())
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
}

/// Get metadata about a FullVector ELID.
///
/// Returns a dictionary containing information about how the ELID was encoded,
/// including original dimensions, precision, and dimension mode.
///
/// Args:
///     elid_str (str): A valid ELID string (base32hex encoded)
///
/// Returns:
///     Optional[dict]: Metadata dictionary with the following keys, or None if not FullVector:
///         - original_dims (int): Original embedding dimension count
///         - encoded_dims (int): Number of dimensions in encoded representation
///         - is_lossless (bool): Whether exact reconstruction is possible
///         - has_dimension_reduction (bool): Whether dimensions were reduced
///         - precision (str): "Full32", "Half16", "Quant8", or "Bits"
///         - precision_bits (int, optional): Bit count if precision is "Bits"
///         - dimension_mode (str): "Preserve", "Reduce", or "Common"
///
/// Raises:
///     ValueError: If the ELID string is invalid
///
/// Example:
///     >>> import elid
///     >>> import numpy as np
///     >>> embedding = np.random.randn(768).astype(np.float32)
///     >>> elid_str = elid.encode_compressed(embedding, 0.5)
///     >>> meta = elid.get_metadata(elid_str)
///     >>> print(meta['original_dims'])  # 768
///     >>> print(meta['is_lossless'])    # False
#[cfg(feature = "embeddings")]
#[pyfunction]
fn get_metadata(elid_str: &str, py: Python<'_>) -> PyResult<Option<Py<pyo3::PyAny>>> {
    let elid = embeddings::types::Elid::from_string(elid_str.to_string())
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

    // Check if reversible (FullVector)
    if !embeddings::is_reversible(&elid) {
        return Ok(None);
    }

    // Decode to get metadata
    let (_values, metadata) = embeddings::decode_to_embedding(&elid)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

    // Build result dict
    let dict = pyo3::types::PyDict::new(py);
    dict.set_item("original_dims", metadata.original_dims)?;
    dict.set_item("encoded_dims", metadata.encoded_dims)?;
    dict.set_item("is_lossless", metadata.is_lossless())?;
    dict.set_item(
        "has_dimension_reduction",
        metadata.has_dimension_reduction(),
    )?;

    // Precision as string
    let precision_str = match metadata.precision {
        EmbedVectorPrecision::Full32 => "Full32",
        EmbedVectorPrecision::Half16 => "Half16",
        EmbedVectorPrecision::Quant8 => "Quant8",
        EmbedVectorPrecision::Bits { bits } => {
            dict.set_item("precision_bits", bits)?;
            "Bits"
        }
    };
    dict.set_item("precision", precision_str)?;

    // Dimension mode as string
    let mode_str = match metadata.dimension_mode {
        EmbedDimensionMode::Preserve => "Preserve",
        EmbedDimensionMode::Reduce { .. } => "Reduce",
        EmbedDimensionMode::Common { .. } => "Common",
    };
    dict.set_item("dimension_mode", mode_str)?;

    Ok(Some(dict.unbind().into()))
}

// ============================================================================
// Model functions (feature-gated)
// ============================================================================

/// Embed text using Model2Vec potion-base-8M model
///
/// Converts input text into a 256-dimensional embedding vector using
/// the Model2Vec potion-base-8M model. This is useful for semantic
/// text similarity, search, and clustering.
///
/// Args:
///     text (str): Input text to embed
///
/// Returns:
///     list[float]: 256-dimensional embedding as list of floats
///
/// Raises:
///     ValueError: If model not available or inference fails
///
/// Example:
///     >>> import elid
///     >>> embedding = elid.embed_text("Hello, world!")
///     >>> len(embedding)
///     256
///     >>> type(embedding[0])
///     <class 'float'>
#[cfg(feature = "models-text")]
#[pyfunction]
fn embed_text(text: &str) -> PyResult<Vec<f32>> {
    crate::models::embed_text(text)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
}

/// Embed image using MobileNetV3-Small model
///
/// Converts an image into a 1024-dimensional embedding vector using
/// the MobileNetV3-Small model. Supports JPEG and PNG formats.
/// This is useful for image similarity, search, and clustering.
///
/// Args:
///     image_bytes (bytes): Raw image bytes (JPEG or PNG)
///
/// Returns:
///     list[float]: 1024-dimensional embedding as list of floats
///
/// Raises:
///     ValueError: If model not available, image decode fails, or inference fails
///
/// Example:
///     >>> import elid
///     >>> with open("image.jpg", "rb") as f:
///     ...     image_bytes = f.read()
///     >>> embedding = elid.embed_image(image_bytes)
///     >>> len(embedding)
///     1024
///     >>> type(embedding[0])
///     <class 'float'>
#[cfg(feature = "models-image")]
#[pyfunction]
fn embed_image(image_bytes: &[u8]) -> PyResult<Vec<f32>> {
    crate::models::embed_image(image_bytes)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
}

// ============================================================================
// LSH Band functions (feature-gated)
// ============================================================================

/// Generate LSH bands from an embedding for database querying
///
/// Computes a 128-bit SimHash of the embedding and splits it into
/// `num_bands` equal parts, each encoded as a base32hex string.
/// Band matching is used for efficient approximate nearest neighbor
/// search: if two embeddings share at least one identical band,
/// they are likely similar.
///
/// Args:
///     embedding (list[float]): Embedding vector as list of floats
///     num_bands (int): Number of bands (1, 2, 4, 8, or 16). Default: 4
///     seed (int, optional): Seed for SimHash generation. Default: 0x454c494453494d48
///
/// Returns:
///     list[str]: List of base32hex band strings
///
/// Band Sizes:
///     | num_bands | bits/band | chars/band |
///     |-----------|-----------|------------|
///     | 1         | 128       | 26         |
///     | 2         | 64        | 13         |
///     | 4         | 32        | 7          |
///     | 8         | 16        | 4          |
///     | 16        | 8         | 2          |
///
/// Example:
///     >>> import elid
///     >>> import numpy as np
///     >>> embedding = np.random.randn(768).astype(np.float32).tolist()
///     >>> bands = elid.embedding_to_bands(embedding, num_bands=4)
///     >>> len(bands)
///     4
///     >>> len(bands[0])  # 32 bits = 4 bytes = 7 base32hex chars
///     7
///
/// Database Usage:
///     Store each band in an indexed column for efficient querying:
///     ```sql
///     CREATE INDEX idx_band0 ON embeddings(band0);
///     -- Query for similar embeddings
///     SELECT * FROM embeddings
///     WHERE band0 = ? OR band1 = ? OR band2 = ? OR band3 = ?;
///     ```
#[cfg(feature = "embeddings")]
#[pyfunction]
#[pyo3(name = "embedding_to_bands", signature = (embedding, num_bands=4, seed=None))]
fn embedding_to_bands_py(embedding: Vec<f32>, num_bands: u8, seed: Option<u64>) -> Vec<String> {
    let seed = seed.unwrap_or(0x454c4944_53494d48); // Default "ELIDSIMH" seed
    crate::embeddings::embedding_to_bands(&embedding, num_bands, seed)
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
        // Basic embedding functions
        m.add_function(wrap_pyfunction!(encode_embedding, m)?)?;
        m.add_function(wrap_pyfunction!(decode_elid, m)?)?;
        m.add_function(wrap_pyfunction!(elid_hamming_distance, m)?)?;
        m.add_class::<Profile>()?;

        // FullVector encoding functions
        m.add_function(wrap_pyfunction!(encode_lossless, m)?)?;
        m.add_function(wrap_pyfunction!(encode_compressed, m)?)?;
        m.add_function(wrap_pyfunction!(encode_max_length, m)?)?;
        m.add_function(wrap_pyfunction!(decode_to_embedding, m)?)?;
        m.add_function(wrap_pyfunction!(is_reversible, m)?)?;
        m.add_function(wrap_pyfunction!(encode_cross_dimensional, m)?)?;
        m.add_function(wrap_pyfunction!(get_metadata, m)?)?;

        // FullVector types
        m.add_class::<VectorPrecision>()?;
        m.add_class::<DimensionMode>()?;

        // LSH band generation
        m.add_function(wrap_pyfunction!(embedding_to_bands_py, m)?)?;
    }

    // Model functions (feature-gated)
    #[cfg(feature = "models-text")]
    m.add_function(wrap_pyfunction!(embed_text, m)?)?;

    #[cfg(feature = "models-image")]
    m.add_function(wrap_pyfunction!(embed_image, m)?)?;

    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    Ok(())
}
