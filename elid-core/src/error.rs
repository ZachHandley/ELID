//! Error types for ELID operations

use thiserror::Error;

/// Errors that can occur during ELID operations
#[derive(Debug, Error)]
pub enum ElidError {
    /// Embedding dimension is outside the valid range for the profile
    #[error("Invalid dimension: got {got}, expected range {expected_range:?}")]
    InvalidDimension {
        /// The actual dimension provided
        got: usize,
        /// The expected dimension range (min, max)
        expected_range: (usize, usize),
    },

    /// Embedding contains NaN or Inf values
    #[error("Invalid embedding value (NaN or Inf detected)")]
    InvalidValue,

    /// Invalid base32hex encoding in ID string
    #[error("Invalid base32hex encoding")]
    InvalidEncoding,

    /// Invalid or corrupted ID header
    #[error("Invalid ID header")]
    InvalidHeader,

    /// Hamming ball radius exceeds maximum allowed value (3)
    #[error("Hamming radius too large: {0} (max 3)")]
    RadiusTooLarge(u8),

    /// Profile in ID doesn't match expected profile
    #[error("Profile mismatch: expected {expected}, got {got}")]
    ProfileMismatch {
        /// The expected profile name
        expected: String,
        /// The actual profile name found in the ID
        got: String,
    },

    /// Transform ID not found in profile configuration
    #[error("Transform not found: ID {0}")]
    TransformNotFound(u16),
}
