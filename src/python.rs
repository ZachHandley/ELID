//! Python bindings for ELID using PyO3
//!
//! This module provides Python bindings for all ELID functions.

use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;

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
    crate::levenshtein::levenshtein(a, b)
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
    crate::levenshtein::normalized_levenshtein(a, b)
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
    crate::jaro_winkler::jaro(a, b)
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
    crate::jaro_winkler::jaro_winkler(a, b)
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
    crate::hamming::hamming(a, b)
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
    crate::osa::osa_distance(a, b)
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
fn find_best_match(query: &str, candidates: Vec<&str>) -> PyResult<PyObject> {
    let (idx, score) = crate::find_best_match(query, &candidates);

    Python::with_gil(|py| {
        let dict = pyo3::types::PyDict::new(py);
        dict.set_item("index", idx)?;
        dict.set_item("score", score)?;
        Ok(dict.into())
    })
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
    candidates: Vec<&str>,
    threshold: f64,
) -> PyResult<PyObject> {
    let matches = crate::find_matches_above_threshold(query, &candidates, threshold);

    Python::with_gil(|py| {
        let list = pyo3::types::PyList::empty(py);
        for (idx, score) in matches {
            let dict = pyo3::types::PyDict::new(py);
            dict.set_item("index", idx)?;
            dict.set_item("score", score)?;
            list.append(dict)?;
        }
        Ok(list.into())
    })
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
    crate::levenshtein::levenshtein_with_opts(a, b, &rust_opts)
}

/// ELID - Efficient Levenshtein and String Similarity Library
///
/// A fast library for computing various string similarity metrics.
#[pymodule]
fn elid(_py: Python, m: &PyModule) -> PyResult<()> {
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
    m.add_class::<SimilarityOpts>()?;

    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    Ok(())
}
