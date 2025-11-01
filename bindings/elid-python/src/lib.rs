//! Python bindings for ELID (Embedding Locality IDentifier)
//!
//! This module provides PyO3 bindings that expose ELID encoding/decoding
//! functionality to Python with zero-copy NumPy array support.

use numpy::PyReadonlyArray1;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use rayon::prelude::*;

/// Profile enum for Python
#[pyclass]
#[derive(Clone, Copy, Debug)]
enum Profile {
    Mini128,
    Morton10x10,
    Hilbert10x10,
}

#[pymethods]
impl Profile {
    #[new]
    fn new(name: &str) -> PyResult<Self> {
        match name {
            "Mini128" => Ok(Profile::Mini128),
            "Morton10x10" => Ok(Profile::Morton10x10),
            "Hilbert10x10" => Ok(Profile::Hilbert10x10),
            _ => Err(PyValueError::new_err(format!(
                "Invalid profile: {}. Valid options: Mini128, Morton10x10, Hilbert10x10",
                name
            ))),
        }
    }

    fn __repr__(&self) -> String {
        match self {
            Profile::Mini128 => "Profile.Mini128".to_string(),
            Profile::Morton10x10 => "Profile.Morton10x10".to_string(),
            Profile::Hilbert10x10 => "Profile.Hilbert10x10".to_string(),
        }
    }

    fn __str__(&self) -> String {
        match self {
            Profile::Mini128 => "Mini128".to_string(),
            Profile::Morton10x10 => "Morton10x10".to_string(),
            Profile::Hilbert10x10 => "Hilbert10x10".to_string(),
        }
    }
}

impl Profile {
    /// Convert Python Profile to Rust Profile
    fn to_rust_profile(&self) -> elid_core::Profile {
        match self {
            Profile::Mini128 => elid_core::Profile::Mini128 {
                seed: 0x454c4944_53494d48, // "ELIDSIMH" in hex (default)
            },
            Profile::Morton10x10 => elid_core::Profile::Morton10x10 {
                dims: 10,
                bits_per_dim: 10,
                transform_id: None,
            },
            Profile::Hilbert10x10 => elid_core::Profile::Hilbert10x10 {
                dims: 10,
                bits_per_dim: 10,
                transform_id: None,
            },
        }
    }
}

/// Encode an embedding into an ELID string
///
/// Args:
///     embedding: NumPy array of shape (N,) where 64 ≤ N ≤ 2048
///     profile: Encoding profile (Mini128, Morton10x10, or Hilbert10x10)
///
/// Returns:
///     Sortable string identifier (29 chars for Mini128, ~24 for others)
///
/// Raises:
///     ValueError: If embedding dimensions are invalid or encoding fails
///
/// Examples:
///     >>> import elid
///     >>> import numpy as np
///     >>> embedding = np.random.randn(768)
///     >>> elid_id = elid.encode(embedding, elid.Profile.Mini128)
///     >>> len(elid_id)
///     29
#[pyfunction]
fn encode(embedding: PyReadonlyArray1<f32>, profile: Profile) -> PyResult<String> {
    // Zero-copy access to NumPy array data
    let embedding_slice = embedding.as_slice().map_err(|e| {
        PyValueError::new_err(format!("Failed to access embedding data: {}", e))
    })?;

    // Convert to Rust profile
    let rust_profile = profile.to_rust_profile();

    // Call elid-core encode function
    let elid = elid_core::encode(embedding_slice, &rust_profile).map_err(|e| match e {
        elid_core::ElidError::InvalidDimension { got, expected_range } => {
            PyValueError::new_err(format!(
                "Invalid embedding dimension: got {}, expected range {:?}",
                got, expected_range
            ))
        }
        elid_core::ElidError::InvalidValue => {
            PyValueError::new_err("Embedding contains NaN or Inf values")
        }
        _ => PyRuntimeError::new_err(format!("Encoding error: {}", e)),
    })?;

    Ok(elid.as_str().to_string())
}

/// Decode an ELID string to raw bytes
///
/// Args:
///     elid_str: ELID string identifier (base32hex encoded)
///
/// Returns:
///     Raw bytes (18 bytes for Mini128: 2 header + 16 payload)
///
/// Raises:
///     ValueError: If ELID string is malformed or invalid encoding
///
/// Examples:
///     >>> import elid
///     >>> elid_str = "0123456789abcdefghijklmnop"
///     >>> raw_bytes = elid.decode(elid_str)
///     >>> len(raw_bytes)
///     18
#[pyfunction]
fn decode<'py>(py: Python<'py>, elid_str: &str) -> PyResult<Bound<'py, PyBytes>> {
    // Create Elid from string
    let elid = elid_core::Elid::from_string(elid_str.to_string())
        .map_err(|e| PyValueError::new_err(format!("Invalid ELID string: {}", e)))?;

    // Decode to bytes
    let bytes = elid_core::decode(&elid)
        .map_err(|e| PyValueError::new_err(format!("Decoding failed: {}", e)))?;

    Ok(PyBytes::new_bound(py, &bytes))
}

/// Compute Hamming distance between two Mini128 ELIDs
///
/// Args:
///     elid1: First ELID string
///     elid2: Second ELID string (must use same profile as elid1)
///
/// Returns:
///     Hamming distance (0-128 for Mini128)
///     Lower distance indicates higher similarity.
///
/// Raises:
///     ValueError: If ELIDs use different profiles or are malformed
///
/// Examples:
///     >>> import elid
///     >>> import numpy as np
///     >>> embedding = np.ones(768)
///     >>> elid1 = elid.encode(embedding, elid.Profile.Mini128)
///     >>> elid2 = elid.encode(embedding, elid.Profile.Mini128)
///     >>> elid.hamming_distance(elid1, elid2)
///     0
#[pyfunction]
fn hamming_distance(elid1: &str, elid2: &str) -> PyResult<u32> {
    // Create Elid instances
    let elid_a = elid_core::Elid::from_string(elid1.to_string())
        .map_err(|e| PyValueError::new_err(format!("Invalid ELID 1: {}", e)))?;
    let elid_b = elid_core::Elid::from_string(elid2.to_string())
        .map_err(|e| PyValueError::new_err(format!("Invalid ELID 2: {}", e)))?;

    // Compute Hamming distance
    elid_core::hamming_distance(&elid_a, &elid_b)
        .map_err(|e| PyValueError::new_err(format!("Hamming distance failed: {}", e)))
}

/// Encode multiple embeddings in parallel using Rayon
///
/// Args:
///     embeddings: List of NumPy arrays, each shape (N,) where 64 ≤ N ≤ 2048
///     profile: Encoding profile for all embeddings
///
/// Returns:
///     List of ELID strings, same order as input
///
/// Raises:
///     ValueError: If any embedding has invalid dimensions
///
/// Examples:
///     >>> import elid
///     >>> import numpy as np
///     >>> embeddings = [np.random.randn(768) for _ in range(1000)]
///     >>> elids = elid.encode_batch(embeddings, elid.Profile.Mini128)
///     >>> len(elids)
///     1000
#[pyfunction]
fn encode_batch(
    py: Python,
    embeddings: Vec<PyReadonlyArray1<f32>>,
    profile: Profile,
) -> PyResult<Vec<String>> {
    // Convert profile once
    let rust_profile = profile.to_rust_profile();

    // Extract all embedding data while we have GIL
    let embeddings_vec: Result<Vec<Vec<f32>>, PyErr> = embeddings
        .iter()
        .map(|embedding| {
            let slice = embedding.as_slice().map_err(|e| {
                PyValueError::new_err(format!("Failed to access embedding data: {}", e))
            })?;
            Ok(slice.to_vec())
        })
        .collect();

    let embeddings_vec = embeddings_vec?;

    // Release GIL for parallel processing
    py.allow_threads(|| {
        // Process embeddings in parallel using Rayon
        embeddings_vec
            .par_iter()
            .map(|embedding_slice| {
                // Encode
                let elid = elid_core::encode(embedding_slice, &rust_profile).map_err(|e| {
                    match e {
                        elid_core::ElidError::InvalidDimension { got, expected_range } => {
                            PyValueError::new_err(format!(
                                "Invalid embedding dimension: got {}, expected range {:?}",
                                got, expected_range
                            ))
                        }
                        elid_core::ElidError::InvalidValue => {
                            PyValueError::new_err("Embedding contains NaN or Inf values")
                        }
                        _ => PyRuntimeError::new_err(format!("Encoding error: {}", e)),
                    }
                })?;

                Ok(elid.as_str().to_string())
            })
            .collect()
    })
}

/// ELID Python module
#[pymodule]
fn elid(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Profile>()?;
    m.add_function(wrap_pyfunction!(encode, m)?)?;
    m.add_function(wrap_pyfunction!(decode, m)?)?;
    m.add_function(wrap_pyfunction!(hamming_distance, m)?)?;
    m.add_function(wrap_pyfunction!(encode_batch, m)?)?;

    // Add module-level constants
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("__doc__", "ELID: Embedding Locality IDentifier - Compact, sortable identifiers for high-dimensional embeddings")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_conversion() {
        let mini = Profile::Mini128;
        let rust_profile = mini.to_rust_profile();
        assert!(matches!(rust_profile, elid_core::Profile::Mini128 { .. }));
    }

    #[test]
    fn test_profile_creation() {
        assert!(Profile::new("Mini128").is_ok());
        assert!(Profile::new("Morton10x10").is_ok());
        assert!(Profile::new("Hilbert10x10").is_ok());
        assert!(Profile::new("Invalid").is_err());
    }
}
