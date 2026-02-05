//! Core data types for ELID encoding and decoding
//!
//! This module defines the fundamental types used throughout the ELID embedding system:
//! - [`Elid`]: The output identifier (base32hex-encoded string)
//! - [`Profile`]: Encoding strategy configuration
//! - [`ProfileInfo`]: Metadata extracted from ID headers
//! - [`Embedding`]: Input vector representation
//! - [`VectorPrecision`]: Precision options for full vector encoding
//! - [`DimensionMode`]: Dimension handling modes for projection/reduction

use super::encoding::decode_sortable;
use super::error::ElidError;
use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// VectorPrecision - Precision options for full vector encoding
// ============================================================================

/// Precision options for full vector encoding
///
/// Controls how many bits are used to represent each dimension value.
/// Higher precision means more accurate reconstruction but larger output.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum VectorPrecision {
    /// Full 32-bit float (lossless, 4 bytes per dimension)
    #[default]
    Full32,
    /// 16-bit half-precision float (2 bytes per dimension)
    Half16,
    /// 8-bit quantized (1 byte per dimension, ~1% error)
    Quant8,
    /// Custom bit depth (1-32 bits per dimension)
    Bits {
        /// Number of bits per dimension (1-32)
        bits: u8,
    },
}

impl VectorPrecision {
    /// Get the number of bits used per dimension
    #[must_use]
    pub fn bits_per_dim(&self) -> u8 {
        match self {
            VectorPrecision::Full32 => 32,
            VectorPrecision::Half16 => 16,
            VectorPrecision::Quant8 => 8,
            VectorPrecision::Bits { bits } => *bits,
        }
    }

    /// Validate the precision settings
    pub fn validate(&self) -> Result<(), ElidError> {
        match self {
            VectorPrecision::Bits { bits } if *bits == 0 || *bits > 32 => Err(
                ElidError::InvalidPrecision(format!("Bits must be 1-32, got {}", bits)),
            ),
            _ => Ok(()),
        }
    }
}

// ============================================================================
// DimensionMode - Dimension handling modes
// ============================================================================

/// Dimension handling mode for full vector encoding
///
/// Controls whether to preserve original dimensions, reduce them,
/// or project to a common space for cross-dimensional comparison.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum DimensionMode {
    /// Preserve all original dimensions (no projection)
    #[default]
    Preserve,
    /// Reduce to target dimensions using random projection
    Reduce {
        /// Target number of dimensions (must be < original)
        target_dims: u16,
    },
    /// Project to common space for cross-dimensional comparison
    ///
    /// This allows comparing vectors of different original dimensions
    /// by projecting them to the same intermediate space.
    Common {
        /// Common dimension space (all vectors projected to this)
        dims: u16,
    },
}

impl DimensionMode {
    /// Get the output dimension count for a given input dimension
    #[must_use]
    pub fn output_dims(&self, input_dims: u16) -> u16 {
        match self {
            DimensionMode::Preserve => input_dims,
            DimensionMode::Reduce { target_dims } => *target_dims,
            DimensionMode::Common { dims } => *dims,
        }
    }

    /// Validate the dimension mode against input dimensions
    pub fn validate(&self, input_dims: u16) -> Result<(), ElidError> {
        match self {
            DimensionMode::Preserve => Ok(()),
            DimensionMode::Reduce { target_dims } => {
                if *target_dims == 0 {
                    Err(ElidError::InvalidDimension {
                        got: 0,
                        expected_range: (1, input_dims as usize),
                    })
                } else if *target_dims >= input_dims {
                    Err(ElidError::ProjectionError(format!(
                        "Target dims {} must be less than input dims {}",
                        target_dims, input_dims
                    )))
                } else {
                    Ok(())
                }
            }
            DimensionMode::Common { dims } => {
                if *dims == 0 {
                    Err(ElidError::InvalidDimension {
                        got: 0,
                        expected_range: (1, 2048),
                    })
                } else {
                    Ok(())
                }
            }
        }
    }
}

// ============================================================================
// Elid - The primary output type
// ============================================================================

/// An ELID string (base32hex-encoded, lexicographically sortable)
///
/// This is the primary output type representing an encoded embedding ID.
/// The string contains only base32hex characters (0-9, a-v) and maintains
/// lexicographic ordering that matches the binary ordering of underlying bytes.
///
/// # Examples
///
/// ```rust,ignore
/// let id = Elid::from_string("0123456789abcdefghijk".to_string())?;
/// assert_eq!(id.as_str(), "0123456789abcdefghijk");
/// ```
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Elid(String);

impl Elid {
    /// Create from validated base32hex string
    ///
    /// # Errors
    ///
    /// Returns [`ElidError::InvalidEncoding`] if the string contains characters
    /// outside the base32hex alphabet (0-9, a-v).
    pub fn from_string(s: String) -> Result<Self, ElidError> {
        // Validate: only 0-9, a-v characters
        if s.chars().all(|c| matches!(c, '0'..='9' | 'a'..='v')) {
            Ok(Elid(s))
        } else {
            Err(ElidError::InvalidEncoding)
        }
    }

    /// Get the underlying string (zero-copy)
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Decode to raw bytes (big-endian)
    ///
    /// # Errors
    ///
    /// Returns [`ElidError::InvalidEncoding`] if the base32hex decoding fails.
    pub fn to_bytes(&self) -> Result<Vec<u8>, ElidError> {
        decode_sortable(self.as_str())
    }

    /// Extract profile information from header (first 2 bytes)
    ///
    /// # Errors
    ///
    /// Returns an error if decoding fails or if the header is invalid.
    pub fn profile(&self) -> Result<ProfileInfo, ElidError> {
        let bytes = self.to_bytes()?;
        if bytes.len() < 2 {
            return Err(ElidError::InvalidHeader);
        }
        ProfileInfo::from_header(&bytes[0..2])
    }
}

impl fmt::Debug for Elid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Elid(\"{}\")", self.0)
    }
}

impl fmt::Display for Elid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ============================================================================
// Profile - Encoding strategy configuration
// ============================================================================

/// Encoding profile determining ID generation strategy
///
/// Profiles define how embeddings are transformed into compact identifiers.
/// Different profiles offer trade-offs between speed, locality preservation,
/// and compression.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Profile {
    /// 128-bit SimHash (cosine similarity via Hamming distance)
    ///
    /// Uses signed random projections to create a locality-sensitive hash.
    /// Fast encoding, supports similarity search via Hamming distance.
    Mini128 {
        /// Seed for deterministic random projections
        seed: u64,
    },

    /// Morton curve encoding (sortable locality)
    ///
    /// Interleaves quantized dimension bits for Z-order curve traversal.
    /// Fast encoding with good locality preservation.
    Morton10x10 {
        /// Number of dimensions to encode (1-32)
        dims: u8,
        /// Bits per dimension (1-16)
        bits_per_dim: u8,
        /// Rotation/PCA transform ID (optional, for v0.2+)
        #[serde(skip_serializing_if = "Option::is_none")]
        transform_id: Option<u16>,
    },

    /// Hilbert curve encoding (higher quality, slower)
    ///
    /// Maps to Hilbert space-filling curve for better locality preservation
    /// than Morton encoding. Slower to compute but better for range queries.
    Hilbert10x10 {
        /// Number of dimensions to encode (1-32)
        dims: u8,
        /// Bits per dimension (1-16)
        bits_per_dim: u8,
        /// Rotation/PCA transform ID (optional, for v0.2+)
        #[serde(skip_serializing_if = "Option::is_none")]
        transform_id: Option<u16>,
    },

    /// Full vector encoding (reversible, supports various precisions)
    ///
    /// Encodes the full embedding vector with configurable precision and
    /// optional dimension reduction. Supports lossless and lossy modes.
    ///
    /// Key features:
    /// - Reversible: Can decode back to original embedding
    /// - Precision options: Full32, Half16, Quant8, or custom bits
    /// - Dimension reduction: Random projection for size reduction
    /// - Cross-dimensional comparison: Project different-sized vectors to common space
    FullVector {
        /// Precision for each dimension value
        precision: VectorPrecision,
        /// How to handle dimensions (preserve, reduce, or common space)
        dimensions: DimensionMode,
        /// Seed for deterministic random projections (used for dimension reduction)
        seed: u64,
    },
}

impl Profile {
    /// Expected output length in bits (excluding header)
    ///
    /// For FullVector, this is approximate and depends on original dimensions.
    /// Use `bit_length_for_dims` for FullVector with known dimensions.
    #[must_use]
    pub fn bit_length(&self) -> usize {
        match self {
            Profile::Mini128 { .. } => 128,
            Profile::Morton10x10 {
                dims, bits_per_dim, ..
            }
            | Profile::Hilbert10x10 {
                dims, bits_per_dim, ..
            } => (*dims as usize) * (*bits_per_dim as usize),
            Profile::FullVector {
                precision,
                dimensions,
                ..
            } => {
                // Default estimate for 768-dim embeddings
                let output_dims = dimensions.output_dims(768);
                (output_dims as usize) * (precision.bits_per_dim() as usize)
            }
        }
    }

    /// Expected output length in bits for a given input dimension count
    ///
    /// More accurate than `bit_length()` for FullVector profiles.
    #[must_use]
    pub fn bit_length_for_dims(&self, input_dims: u16) -> usize {
        match self {
            Profile::Mini128 { .. } => 128,
            Profile::Morton10x10 {
                dims, bits_per_dim, ..
            }
            | Profile::Hilbert10x10 {
                dims, bits_per_dim, ..
            } => (*dims as usize) * (*bits_per_dim as usize),
            Profile::FullVector {
                precision,
                dimensions,
                ..
            } => {
                let output_dims = dimensions.output_dims(input_dims);
                (output_dims as usize) * (precision.bits_per_dim() as usize)
            }
        }
    }

    /// Expected base32hex string length (rounded up)
    ///
    /// Base32hex uses 5 bits per character, so string length is
    /// `ceil(bit_length / 5)`.
    #[must_use]
    pub fn string_length(&self) -> usize {
        // Add header bits (16 bits for basic header, up to 96 for extended)
        let header_bits = match self {
            Profile::FullVector { .. } => 96, // Extended header for full vector
            _ => 16,
        };
        (self.bit_length() + header_bits).div_ceil(5)
    }

    /// Expected base32hex string length for given input dimensions
    #[must_use]
    pub fn string_length_for_dims(&self, input_dims: u16) -> usize {
        let header_bits = match self {
            Profile::FullVector { .. } => 96, // Extended header for full vector
            _ => 16,
        };
        (self.bit_length_for_dims(input_dims) + header_bits).div_ceil(5)
    }

    /// Profile type ID (for header encoding)
    ///
    /// Returns the numeric identifier used in the ID header byte:
    /// - `0x01`: Mini128
    /// - `0x02`: Morton10x10
    /// - `0x03`: Hilbert10x10
    /// - `0x04`: FullVector
    #[must_use]
    pub fn type_id(&self) -> u8 {
        match self {
            Profile::Mini128 { .. } => 0x01,
            Profile::Morton10x10 { .. } => 0x02,
            Profile::Hilbert10x10 { .. } => 0x03,
            Profile::FullVector { .. } => 0x04,
        }
    }

    /// Check if this profile supports decoding back to the original embedding
    #[must_use]
    pub fn is_reversible(&self) -> bool {
        matches!(self, Profile::FullVector { .. })
    }

    // ========================================================================
    // Convenience Constructors
    // ========================================================================

    /// Create a lossless full vector profile (Full32 precision, preserve all dims)
    ///
    /// This produces the largest output but allows exact reconstruction
    /// of the original embedding.
    #[must_use]
    pub fn lossless() -> Self {
        Profile::FullVector {
            precision: VectorPrecision::Full32,
            dimensions: DimensionMode::Preserve,
            seed: 0x454c4944_46554c4c, // "ELIDFULL"
        }
    }

    /// Create a compressed profile with specified retention percentage
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
    /// - `retention_pct`: Information retention (0.0-1.0)
    /// - `original_dims`: Original embedding dimension count
    ///
    /// # Returns
    ///
    /// A FullVector profile configured for the target retention
    #[must_use]
    pub fn compressed(retention_pct: f32, original_dims: u16) -> Self {
        let retention = retention_pct.clamp(0.01, 1.0);

        // Full lossless = 32 bits * original_dims
        let full_bits = 32.0 * original_dims as f32;
        let target_bits = full_bits * retention;

        // Strategy: prefer dimension reduction over precision reduction
        // because it preserves more geometric relationships

        // Start with Full32 and reduce dimensions
        let full32_target_dims = (target_bits / 32.0).round() as u16;
        if full32_target_dims >= original_dims {
            // No reduction needed
            return Profile::lossless();
        }

        if full32_target_dims >= original_dims / 4 {
            // Use Full32 with dimension reduction
            return Profile::FullVector {
                precision: VectorPrecision::Full32,
                dimensions: DimensionMode::Reduce {
                    target_dims: full32_target_dims.max(1),
                },
                seed: 0x454c4944_434f4d50, // "ELIDCOMP"
            };
        }

        // Try Half16 with dimension reduction
        let half16_target_dims = (target_bits / 16.0).round() as u16;
        if half16_target_dims >= original_dims / 4 {
            return Profile::FullVector {
                precision: VectorPrecision::Half16,
                dimensions: if half16_target_dims >= original_dims {
                    DimensionMode::Preserve
                } else {
                    DimensionMode::Reduce {
                        target_dims: half16_target_dims.max(1),
                    }
                },
                seed: 0x454c4944_434f4d50,
            };
        }

        // Try Quant8 with dimension reduction
        let quant8_target_dims = (target_bits / 8.0).round() as u16;
        Profile::FullVector {
            precision: VectorPrecision::Quant8,
            dimensions: if quant8_target_dims >= original_dims {
                DimensionMode::Preserve
            } else {
                DimensionMode::Reduce {
                    target_dims: quant8_target_dims.max(1),
                }
            },
            seed: 0x454c4944_434f4d50,
        }
    }

    /// Create a profile optimized for a maximum output string length
    ///
    /// Calculates the optimal precision and dimension settings to fit
    /// within the specified character limit while maximizing fidelity.
    ///
    /// # Parameters
    ///
    /// - `max_chars`: Maximum output string length in characters
    /// - `original_dims`: Original embedding dimension count
    ///
    /// # Returns
    ///
    /// A FullVector profile configured for the target length
    #[must_use]
    pub fn max_length(max_chars: usize, original_dims: u16) -> Self {
        // Base32hex: 5 bits per character
        // Header: 12 bytes (96 bits = ~20 chars)
        let header_chars = 20;
        let payload_chars = max_chars.saturating_sub(header_chars);
        let payload_bits = payload_chars * 5;

        if payload_bits == 0 {
            // Minimum viable encoding
            return Profile::FullVector {
                precision: VectorPrecision::Bits { bits: 1 },
                dimensions: DimensionMode::Reduce { target_dims: 1 },
                seed: 0x454c4944_4d41584c, // "ELIDMAXL"
            };
        }

        // Calculate what we can fit
        let bits_per_dim_full32 = payload_bits / original_dims as usize;

        if bits_per_dim_full32 >= 32 {
            // Can fit full lossless
            return Profile::lossless();
        }

        // Try different precision levels
        let precisions = [
            (VectorPrecision::Full32, 32),
            (VectorPrecision::Half16, 16),
            (VectorPrecision::Quant8, 8),
            (VectorPrecision::Bits { bits: 4 }, 4),
            (VectorPrecision::Bits { bits: 2 }, 2),
            (VectorPrecision::Bits { bits: 1 }, 1),
        ];

        for (precision, bits) in precisions {
            let dims_that_fit = payload_bits / bits;
            if dims_that_fit >= original_dims as usize {
                // All dimensions fit at this precision
                return Profile::FullVector {
                    precision,
                    dimensions: DimensionMode::Preserve,
                    seed: 0x454c4944_4d41584c,
                };
            } else if dims_that_fit >= 16 {
                // Reasonable number of dimensions at this precision
                return Profile::FullVector {
                    precision,
                    dimensions: DimensionMode::Reduce {
                        target_dims: dims_that_fit as u16,
                    },
                    seed: 0x454c4944_4d41584c,
                };
            }
        }

        // Fallback to minimum
        Profile::FullVector {
            precision: VectorPrecision::Bits { bits: 1 },
            dimensions: DimensionMode::Reduce {
                target_dims: (payload_bits as u16).max(1),
            },
            seed: 0x454c4944_4d41584c,
        }
    }

    /// Create a profile for cross-dimensional comparison
    ///
    /// Projects all vectors to a common dimension space, allowing comparison
    /// between embeddings of different original dimensions (e.g., 256d vs 768d).
    ///
    /// # Parameters
    ///
    /// - `common_dims`: Target dimension space (vectors will be projected here)
    /// - `precision`: Precision for the projected values (default: Half16)
    ///
    /// # Returns
    ///
    /// A FullVector profile configured for cross-dimensional comparison
    #[must_use]
    pub fn cross_dimensional(common_dims: u16) -> Self {
        Profile::FullVector {
            precision: VectorPrecision::Half16,
            dimensions: DimensionMode::Common { dims: common_dims },
            seed: 0x454c4944_58444949, // "ELIDXDIM"
        }
    }

    /// Create a cross-dimensional profile with custom precision
    #[must_use]
    pub fn cross_dimensional_with_precision(common_dims: u16, precision: VectorPrecision) -> Self {
        Profile::FullVector {
            precision,
            dimensions: DimensionMode::Common { dims: common_dims },
            seed: 0x454c4944_58444949,
        }
    }
}

impl Default for Profile {
    /// Default profile is Mini128 with standard seed
    ///
    /// The seed `0x454c4944_53494d48` is the ASCII encoding of "ELIDSIMH".
    fn default() -> Self {
        Profile::Mini128 {
            seed: 0x454c4944_53494d48, // "ELIDSIMH" in hex
        }
    }
}

// ============================================================================
// ProfileInfo - Metadata from ID headers
// ============================================================================

/// Profile information decoded from ELID header
///
/// The first 2 bytes of an ELID contain metadata about the encoding profile
/// used. Extended headers may include additional fields like transform IDs.
///
/// For FullVector profiles, the extended header contains:
/// - Bytes 2-3: Original dimension count (u16 big-endian)
/// - Byte 4: Precision type (0=Full32, 1=Half16, 2=Quant8, 3+=Bits(n-3))
/// - Byte 5: Dimension mode (0=Preserve, 1=Reduce, 2=Common)
/// - Bytes 6-7: Target/common dimensions if applicable (u16 big-endian)
/// - Bytes 8-11: Seed lower 32 bits (optional, for reproducibility)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProfileInfo {
    /// Version of ELID format (for backward compatibility)
    pub version: u8,

    /// Profile type ID (0x01=Mini128, 0x02=Morton, 0x03=Hilbert, 0x04=FullVector)
    pub profile_type: u8,

    /// Optional transform ID (for PCA/OPQ rotation)
    pub transform_id: Option<u16>,

    /// Optional model ID (for tracking which embedding model was used)
    pub model_id: Option<u16>,

    /// Original dimension count (FullVector only)
    pub original_dims: Option<u16>,

    /// Precision type for FullVector
    pub precision: Option<VectorPrecision>,

    /// Dimension mode for FullVector
    pub dimension_mode: Option<DimensionMode>,

    /// Seed for deterministic operations
    pub seed: Option<u64>,
}

impl ProfileInfo {
    /// Decode from ID header bytes
    ///
    /// The header format is:
    /// - Byte 0: `(version << 4) | profile_type`
    /// - Byte 1: Reserved / flags
    /// - Bytes 2-3: Optional transform_id (big-endian u16) for non-FullVector
    ///
    /// For FullVector (type 0x04), extended header:
    /// - Bytes 2-3: Original dimensions (u16 big-endian)
    /// - Byte 4: Precision type
    /// - Byte 5: Dimension mode
    /// - Bytes 6-7: Target/common dimensions (u16 big-endian)
    /// - Bytes 8-11: Seed lower 32 bits
    ///
    /// # Errors
    ///
    /// Returns [`ElidError::InvalidHeader`] if the header is too short.
    pub fn from_header(header: &[u8]) -> Result<Self, ElidError> {
        if header.len() < 2 {
            return Err(ElidError::InvalidHeader);
        }

        let version = (header[0] & 0xF0) >> 4; // Upper 4 bits
        let profile_type = header[0] & 0x0F; // Lower 4 bits

        // FullVector has extended header
        if profile_type == 0x04 {
            return Self::from_full_vector_header(version, header);
        }

        // Extended header fields (optional, for v0.2+)
        let transform_id = if header.len() >= 4 {
            Some(u16::from_be_bytes([header[2], header[3]]))
        } else {
            None
        };

        Ok(ProfileInfo {
            version,
            profile_type,
            transform_id,
            model_id: None,
            original_dims: None,
            precision: None,
            dimension_mode: None,
            seed: None,
        })
    }

    /// Decode FullVector extended header
    fn from_full_vector_header(version: u8, header: &[u8]) -> Result<Self, ElidError> {
        // FullVector requires at least 12 bytes header
        if header.len() < 12 {
            return Err(ElidError::InvalidHeader);
        }

        // Bytes 2-3: Original dimensions
        let original_dims = u16::from_be_bytes([header[2], header[3]]);

        // Byte 4: Precision type
        let precision = match header[4] {
            0 => VectorPrecision::Full32,
            1 => VectorPrecision::Half16,
            2 => VectorPrecision::Quant8,
            n if (3..=35).contains(&n) => VectorPrecision::Bits { bits: n - 3 + 1 },
            _ => {
                return Err(ElidError::InvalidMetadata(
                    "Invalid precision type".to_string(),
                ))
            }
        };

        // Byte 5: Dimension mode
        let dim_mode_type = header[5];

        // Bytes 6-7: Target/common dimensions
        let target_dims = u16::from_be_bytes([header[6], header[7]]);

        let dimension_mode = match dim_mode_type {
            0 => DimensionMode::Preserve,
            1 => DimensionMode::Reduce { target_dims },
            2 => DimensionMode::Common { dims: target_dims },
            _ => {
                return Err(ElidError::InvalidMetadata(
                    "Invalid dimension mode".to_string(),
                ))
            }
        };

        // Bytes 8-11: Seed (lower 32 bits, extend to u64)
        let seed_low = u32::from_be_bytes([header[8], header[9], header[10], header[11]]);

        Ok(ProfileInfo {
            version,
            profile_type: 0x04,
            transform_id: None,
            model_id: None,
            original_dims: Some(original_dims),
            precision: Some(precision),
            dimension_mode: Some(dimension_mode),
            seed: Some(seed_low as u64),
        })
    }

    /// Encode to header bytes
    ///
    /// Creates a byte vector containing the encoded header. The format matches
    /// the one expected by [`from_header`](Self::from_header).
    #[must_use]
    pub fn to_header(&self) -> Vec<u8> {
        let mut bytes = vec![
            (self.version << 4) | (self.profile_type & 0x0F),
            0x00, // Reserved / flags
        ];

        // FullVector has extended header
        if self.profile_type == 0x04 {
            // Original dimensions (bytes 2-3)
            let orig_dims = self.original_dims.unwrap_or(0);
            bytes.extend_from_slice(&orig_dims.to_be_bytes());

            // Precision type (byte 4)
            let precision_byte = match self.precision {
                Some(VectorPrecision::Full32) => 0,
                Some(VectorPrecision::Half16) => 1,
                Some(VectorPrecision::Quant8) => 2,
                Some(VectorPrecision::Bits { bits }) => 3 + bits - 1,
                None => 0,
            };
            bytes.push(precision_byte);

            // Dimension mode (byte 5) and target dims (bytes 6-7)
            let (mode_byte, target_dims) = match self.dimension_mode {
                Some(DimensionMode::Preserve) => (0u8, 0u16),
                Some(DimensionMode::Reduce { target_dims }) => (1u8, target_dims),
                Some(DimensionMode::Common { dims }) => (2u8, dims),
                None => (0u8, 0u16),
            };
            bytes.push(mode_byte);
            bytes.extend_from_slice(&target_dims.to_be_bytes());

            // Seed lower 32 bits (bytes 8-11)
            let seed_low = (self.seed.unwrap_or(0) & 0xFFFF_FFFF) as u32;
            bytes.extend_from_slice(&seed_low.to_be_bytes());

            return bytes;
        }

        // Non-FullVector: optional transform_id
        if let Some(tid) = self.transform_id {
            bytes.extend_from_slice(&tid.to_be_bytes());
        }

        bytes
    }

    /// Create ProfileInfo from a FullVector profile
    #[must_use]
    pub fn from_full_vector(
        original_dims: u16,
        precision: VectorPrecision,
        dimensions: DimensionMode,
        seed: u64,
    ) -> Self {
        ProfileInfo {
            version: 0,
            profile_type: 0x04,
            transform_id: None,
            model_id: None,
            original_dims: Some(original_dims),
            precision: Some(precision),
            dimension_mode: Some(dimensions),
            seed: Some(seed),
        }
    }
}

// ============================================================================
// Embedding - Input vector representation
// ============================================================================

/// Embedding vector (input to ELID encoding)
///
/// Represents a high-dimensional vector from an ML model. Embeddings must
/// have dimensions between 64 and 2048, and all values must be finite.
///
/// # Examples
///
/// ```rust,ignore
/// let values = vec![0.1, 0.2, 0.3, 0.4];
/// let embedding = Embedding::new(values)?;
/// assert_eq!(embedding.dim(), 4);
/// ```
#[derive(Clone, Debug)]
pub struct Embedding {
    /// Vector components (f32 for compatibility with most ML frameworks)
    values: Vec<f32>,
}

impl Embedding {
    /// Create from f32 vector
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Dimension is outside the range [64, 2048]
    /// - Any value is NaN or infinite
    pub fn new(values: Vec<f32>) -> Result<Self, ElidError> {
        Self::validate(&values)?;
        Ok(Embedding { values })
    }

    /// Create from f64 vector (converts to f32)
    ///
    /// # Errors
    ///
    /// Returns the same errors as [`new`](Self::new).
    pub fn from_f64(values: Vec<f64>) -> Result<Self, ElidError> {
        let values_f32: Vec<f32> = values.iter().map(|&v| v as f32).collect();
        Self::new(values_f32)
    }

    /// Validate embedding constraints
    fn validate(values: &[f32]) -> Result<(), ElidError> {
        // Check dimensionality
        if values.len() < 64 || values.len() > 2048 {
            return Err(ElidError::InvalidDimension {
                got: values.len(),
                expected_range: (64, 2048),
            });
        }

        // Check for NaN/Inf
        if values.iter().any(|v| !v.is_finite()) {
            return Err(ElidError::InvalidValue);
        }

        // Warn on all-zero (valid but potentially meaningless)
        if values.iter().all(|v| *v == 0.0) {
            // TODO: Add logging when log dependency is added
            // log::warn!("All-zero embedding detected");
        }

        Ok(())
    }

    /// Get values as slice (zero-copy)
    #[must_use]
    pub fn as_slice(&self) -> &[f32] {
        &self.values
    }

    /// Normalize to unit length (L2 norm)
    ///
    /// Divides each component by the L2 norm (Euclidean length) of the vector.
    /// If the norm is zero, the vector remains unchanged.
    pub fn normalize(&mut self) {
        let norm: f32 = self.values.iter().map(|v| v * v).sum::<f32>().sqrt();
        if norm > 0.0 {
            self.values.iter_mut().for_each(|v| *v /= norm);
        }
    }

    /// Dimensionality
    #[must_use]
    pub fn dim(&self) -> usize {
        self.values.len()
    }
}

// ============================================================================
// QuantizedCoords - Space-filling curve coordinates
// ============================================================================

/// Quantized coordinates for space-filling curve encoding
///
/// This type represents an embedding's dimensions quantized to a fixed bit-width
/// for use in Morton or Hilbert curve encoding. Each dimension is mapped from
/// the embedding's value range (assumed to be [-1, 1] after normalization) to
/// an integer range [0, 2^bits_per_dim - 1].
///
/// # Quantization Algorithm
///
/// 1. Normalize embedding value from [-1, 1] to [0, 1]:
///    ```text
///    normalized = (value + 1.0) / 2.0
///    ```
///
/// 2. Scale to integer range and clamp:
///    ```text
///    max_val = (1 << bits_per_dim) - 1
///    quantized = (normalized * max_val).clamp(0, max_val) as u16
///    ```
///
/// # Examples
///
/// ```rust,ignore
/// let embedding = Embedding::new(vec![0.0; 128])?;
/// let coords = QuantizedCoords::from_embedding(&embedding, 10, 10)?;
/// assert_eq!(coords.len(), 10);
/// assert_eq!(coords.bits_per_dim(), 10);
/// ```
#[derive(Clone, Debug)]
pub struct QuantizedCoords {
    /// Coordinate values (one per dimension)
    coords: Vec<u16>,
    /// Bits per dimension
    bits: u8,
}

impl QuantizedCoords {
    /// Create quantized coordinates from an embedding
    ///
    /// Takes the first `dims` dimensions from the embedding and quantizes each
    /// dimension from the embedding value range to [0, 2^bits_per_dim - 1].
    ///
    /// # Parameters
    ///
    /// - `embedding`: Input embedding vector
    /// - `dims`: Number of dimensions to use (must be <= embedding dimension)
    /// - `bits_per_dim`: Bits per dimension (1-16)
    ///
    /// # Quantization Mapping
    ///
    /// Assumes embedding values are in range [-1, 1] after normalization:
    /// - `-1.0` -> `0`
    /// - `0.0` -> `2^(bits_per_dim-1)` (midpoint)
    /// - `1.0` -> `2^bits_per_dim - 1` (max value)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `dims` is 0 or greater than 32
    /// - `bits_per_dim` is 0 or greater than 16
    /// - `dims` exceeds the embedding dimension
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let embedding = Embedding::new(vec![0.5; 256])?;
    /// let coords = QuantizedCoords::from_embedding(&embedding, 8, 12)?;
    /// assert_eq!(coords.len(), 8);
    /// ```
    pub fn from_embedding(
        embedding: &Embedding,
        dims: u8,
        bits_per_dim: u8,
    ) -> Result<Self, ElidError> {
        // Validate parameters
        if dims == 0 || dims > 32 {
            return Err(ElidError::InvalidDimension {
                got: dims as usize,
                expected_range: (1, 32),
            });
        }

        if bits_per_dim == 0 || bits_per_dim > 16 {
            return Err(ElidError::InvalidDimension {
                got: bits_per_dim as usize,
                expected_range: (1, 16),
            });
        }

        if (dims as usize) > embedding.dim() {
            return Err(ElidError::InvalidDimension {
                got: dims as usize,
                expected_range: (1, embedding.dim()),
            });
        }

        // Calculate max value for this bit width
        let max_val = ((1u32 << bits_per_dim) - 1) as f32;

        // Quantize each dimension
        let coords: Vec<u16> = embedding
            .as_slice()
            .iter()
            .take(dims as usize)
            .map(|&value| {
                // Map from [-1, 1] to [0, 1]
                let normalized = (value + 1.0) / 2.0;

                // Scale to [0, max_val] and clamp
                let scaled = normalized * max_val;
                let clamped = scaled.clamp(0.0, max_val);

                // Convert to integer
                clamped.round() as u16
            })
            .collect();

        Ok(QuantizedCoords {
            coords,
            bits: bits_per_dim,
        })
    }

    /// Get coordinate values as slice (zero-copy)
    ///
    /// Returns a reference to the underlying coordinate array without
    /// allocating a new vector.
    #[must_use]
    pub fn as_slice(&self) -> &[u16] {
        &self.coords
    }

    /// Get bits per dimension
    ///
    /// Returns the number of bits allocated for each coordinate dimension.
    #[must_use]
    pub fn bits_per_dim(&self) -> u8 {
        self.bits
    }

    /// Get number of dimensions
    ///
    /// Returns the total number of quantized coordinates.
    #[must_use]
    pub fn len(&self) -> usize {
        self.coords.len()
    }

    /// Check if empty (always false for valid instances)
    ///
    /// This method is provided for clippy::len_without_is_empty compliance.
    /// Valid `QuantizedCoords` instances are never empty due to validation
    /// in `from_embedding`.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.coords.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Elid Tests
    // ========================================================================

    #[test]
    fn test_elid_valid_string() {
        let id = Elid::from_string("0123456789abcdefghijklmnopqrstuv".to_string());
        assert!(id.is_ok());
        assert_eq!(id.unwrap().as_str(), "0123456789abcdefghijklmnopqrstuv");
    }

    #[test]
    fn test_elid_invalid_chars() {
        // 'w' is not in base32hex alphabet (only 0-9, a-v)
        let id = Elid::from_string("0123456789abcdefghijklmnopqrstuvw".to_string());
        assert!(matches!(id, Err(ElidError::InvalidEncoding)));
    }

    #[test]
    fn test_elid_ordering() {
        let id1 = Elid::from_string("00000000".to_string()).unwrap();
        let id2 = Elid::from_string("00000001".to_string()).unwrap();
        assert!(id1 < id2);
    }

    // ========================================================================
    // Profile Tests
    // ========================================================================

    #[test]
    fn test_profile_default() {
        let profile = Profile::default();
        assert_eq!(
            profile,
            Profile::Mini128 {
                seed: 0x454c4944_53494d48
            }
        );
    }

    #[test]
    fn test_profile_bit_length() {
        let mini = Profile::Mini128 { seed: 0 };
        assert_eq!(mini.bit_length(), 128);

        let morton = Profile::Morton10x10 {
            dims: 10,
            bits_per_dim: 10,
            transform_id: None,
        };
        assert_eq!(morton.bit_length(), 100);

        let hilbert = Profile::Hilbert10x10 {
            dims: 8,
            bits_per_dim: 12,
            transform_id: None,
        };
        assert_eq!(hilbert.bit_length(), 96);
    }

    #[test]
    fn test_profile_string_length() {
        // string_length now includes header bits (16 bits = 2 bytes for non-FullVector)
        let mini = Profile::Mini128 { seed: 0 };
        // 128 bits payload + 16 bits header = 144 bits -> ceil(144/5) = 29
        assert_eq!(mini.string_length(), 29);

        let morton = Profile::Morton10x10 {
            dims: 10,
            bits_per_dim: 10,
            transform_id: None,
        };
        // 100 bits payload + 16 bits header = 116 bits -> ceil(116/5) = 24
        assert_eq!(morton.string_length(), 24);
    }

    #[test]
    fn test_profile_type_id() {
        assert_eq!(Profile::Mini128 { seed: 0 }.type_id(), 0x01);
        assert_eq!(
            Profile::Morton10x10 {
                dims: 10,
                bits_per_dim: 10,
                transform_id: None
            }
            .type_id(),
            0x02
        );
        assert_eq!(
            Profile::Hilbert10x10 {
                dims: 10,
                bits_per_dim: 10,
                transform_id: None
            }
            .type_id(),
            0x03
        );
    }

    // ========================================================================
    // ProfileInfo Tests
    // ========================================================================

    #[test]
    fn test_profile_info_basic_header() {
        let header = vec![0x01, 0x00]; // version=0, type=1 (Mini128)
        let info = ProfileInfo::from_header(&header).unwrap();
        assert_eq!(info.version, 0);
        assert_eq!(info.profile_type, 1);
        assert_eq!(info.transform_id, None);
    }

    #[test]
    fn test_profile_info_extended_header() {
        let header = vec![0x12, 0x00, 0x00, 0xFF]; // version=1, type=2, transform=255
        let info = ProfileInfo::from_header(&header).unwrap();
        assert_eq!(info.version, 1);
        assert_eq!(info.profile_type, 2);
        assert_eq!(info.transform_id, Some(255));
    }

    #[test]
    fn test_profile_info_to_header() {
        let info = ProfileInfo {
            version: 1,
            profile_type: 3,
            transform_id: Some(0x1234),
            model_id: None,
            original_dims: None,
            precision: None,
            dimension_mode: None,
            seed: None,
        };
        let header = info.to_header();
        assert_eq!(header[0], 0x13); // (1 << 4) | 3
        assert_eq!(header[1], 0x00);
        assert_eq!(header[2..4], [0x12, 0x34]);
    }

    #[test]
    fn test_profile_info_roundtrip() {
        let info = ProfileInfo {
            version: 2,
            profile_type: 1,
            transform_id: Some(42),
            model_id: None,
            original_dims: None,
            precision: None,
            dimension_mode: None,
            seed: None,
        };
        let header = info.to_header();
        let decoded = ProfileInfo::from_header(&header).unwrap();
        assert_eq!(decoded.version, info.version);
        assert_eq!(decoded.profile_type, info.profile_type);
        assert_eq!(decoded.transform_id, info.transform_id);
    }

    #[test]
    fn test_profile_info_full_vector_roundtrip() {
        let info = ProfileInfo::from_full_vector(
            768,
            VectorPrecision::Half16,
            DimensionMode::Reduce { target_dims: 256 },
            0x12345678,
        );
        let header = info.to_header();
        assert_eq!(header.len(), 12);

        let decoded = ProfileInfo::from_header(&header).unwrap();
        assert_eq!(decoded.version, 0);
        assert_eq!(decoded.profile_type, 0x04);
        assert_eq!(decoded.original_dims, Some(768));
        assert_eq!(decoded.precision, Some(VectorPrecision::Half16));
        assert_eq!(
            decoded.dimension_mode,
            Some(DimensionMode::Reduce { target_dims: 256 })
        );
        assert_eq!(decoded.seed, Some(0x12345678));
    }

    #[test]
    fn test_profile_info_invalid_header() {
        let header = vec![0x01]; // Too short
        assert!(matches!(
            ProfileInfo::from_header(&header),
            Err(ElidError::InvalidHeader)
        ));
    }

    #[test]
    fn test_profile_info_full_vector_short_header() {
        // FullVector (type 0x04) needs 12 bytes
        let header = vec![0x04, 0x00, 0x00, 0x00]; // Only 4 bytes
        assert!(matches!(
            ProfileInfo::from_header(&header),
            Err(ElidError::InvalidHeader)
        ));
    }

    // ========================================================================
    // Embedding Tests
    // ========================================================================

    #[test]
    fn test_embedding_valid() {
        let values = vec![0.1; 128];
        let embedding = Embedding::new(values);
        assert!(embedding.is_ok());
        assert_eq!(embedding.unwrap().dim(), 128);
    }

    #[test]
    fn test_embedding_too_small() {
        let values = vec![0.1; 32]; // < 64
        let embedding = Embedding::new(values);
        assert!(matches!(embedding, Err(ElidError::InvalidDimension { .. })));
    }

    #[test]
    fn test_embedding_too_large() {
        let values = vec![0.1; 4096]; // > 2048
        let embedding = Embedding::new(values);
        assert!(matches!(embedding, Err(ElidError::InvalidDimension { .. })));
    }

    #[test]
    fn test_embedding_nan() {
        let mut values = vec![0.1; 128];
        values[64] = f32::NAN;
        let embedding = Embedding::new(values);
        assert!(matches!(embedding, Err(ElidError::InvalidValue)));
    }

    #[test]
    fn test_embedding_inf() {
        let mut values = vec![0.1; 128];
        values[64] = f32::INFINITY;
        let embedding = Embedding::new(values);
        assert!(matches!(embedding, Err(ElidError::InvalidValue)));
    }

    #[test]
    fn test_embedding_from_f64() {
        let values = vec![0.1_f64; 128];
        let embedding = Embedding::from_f64(values);
        assert!(embedding.is_ok());
    }

    #[test]
    fn test_embedding_normalize() {
        let values = vec![3.0, 4.0].into_iter().cycle().take(128).collect();
        let mut embedding = Embedding::new(values).unwrap();
        embedding.normalize();

        let norm: f32 = embedding
            .as_slice()
            .iter()
            .map(|v| v * v)
            .sum::<f32>()
            .sqrt();
        assert!((norm - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_embedding_normalize_zero() {
        let values = vec![0.0; 128];
        let mut embedding = Embedding::new(values).unwrap();
        embedding.normalize();
        // Should not panic or change values
        assert!(embedding.as_slice().iter().all(|&v| v == 0.0));
    }
}

#[cfg(test)]
mod quantized_coords_tests {
    use super::*;

    // ========================================================================
    // QuantizedCoords Tests
    // ========================================================================

    #[test]
    fn test_quantized_coords_basic() {
        let values = vec![0.0; 128]; // All zeros (midpoint)
        let embedding = Embedding::new(values).unwrap();

        let coords = QuantizedCoords::from_embedding(&embedding, 10, 10).unwrap();

        assert_eq!(coords.len(), 10);
        assert_eq!(coords.bits_per_dim(), 10);
        assert!(!coords.is_empty());

        // 0.0 should map to midpoint: (0 + 1) / 2 * 1023 = 511.5 ~ 512
        let expected = 512u16;
        assert!(
            coords.as_slice().iter().all(|&c| c == expected),
            "All coords should be at midpoint, got {:?}",
            coords.as_slice()
        );
    }

    #[test]
    fn test_quantized_coords_min_value() {
        let values = vec![-1.0; 128]; // Minimum value
        let embedding = Embedding::new(values).unwrap();

        let coords = QuantizedCoords::from_embedding(&embedding, 8, 10).unwrap();

        // -1.0 should map to 0: (-1 + 1) / 2 * 1023 = 0
        assert!(
            coords.as_slice().iter().all(|&c| c == 0),
            "Min values should quantize to 0, got {:?}",
            coords.as_slice()
        );
    }

    #[test]
    fn test_quantized_coords_max_value() {
        let values = vec![1.0; 128]; // Maximum value
        let embedding = Embedding::new(values).unwrap();

        let coords = QuantizedCoords::from_embedding(&embedding, 8, 10).unwrap();

        // 1.0 should map to max: (1 + 1) / 2 * 1023 = 1023
        let expected = (1u16 << 10) - 1; // 1023
        assert!(
            coords.as_slice().iter().all(|&c| c == expected),
            "Max values should quantize to {}, got {:?}",
            expected,
            coords.as_slice()
        );
    }

    #[test]
    fn test_quantized_coords_zero_dims() {
        let values = vec![0.0; 128];
        let embedding = Embedding::new(values).unwrap();

        let result = QuantizedCoords::from_embedding(&embedding, 0, 10);
        assert!(
            matches!(result, Err(ElidError::InvalidDimension { .. })),
            "Should reject 0 dimensions"
        );
    }

    #[test]
    fn test_quantized_coords_too_many_dims() {
        let values = vec![0.0; 128];
        let embedding = Embedding::new(values).unwrap();

        let result = QuantizedCoords::from_embedding(&embedding, 33, 10);
        assert!(
            matches!(result, Err(ElidError::InvalidDimension { .. })),
            "Should reject > 32 dimensions"
        );
    }

    #[test]
    fn test_quantized_coords_zero_bits() {
        let values = vec![0.0; 128];
        let embedding = Embedding::new(values).unwrap();

        let result = QuantizedCoords::from_embedding(&embedding, 10, 0);
        assert!(
            matches!(result, Err(ElidError::InvalidDimension { .. })),
            "Should reject 0 bits per dimension"
        );
    }

    #[test]
    fn test_quantized_coords_too_many_bits() {
        let values = vec![0.0; 128];
        let embedding = Embedding::new(values).unwrap();

        let result = QuantizedCoords::from_embedding(&embedding, 10, 17);
        assert!(
            matches!(result, Err(ElidError::InvalidDimension { .. })),
            "Should reject > 16 bits per dimension"
        );
    }
}
