//! Full vector encoding and decoding for ELID
//!
//! This module provides reversible encoding of embedding vectors with configurable
//! precision and dimension reduction options. Unlike SimHash or space-filling curve
//! encodings, full vector encoding allows reconstruction of the original embedding.
//!
//! # Features
//!
//! - **Lossless mode**: Full 32-bit float preservation
//! - **Half precision**: 16-bit IEEE 754 half-precision floats
//! - **8-bit quantization**: Uniform quantization to 8 bits (~1% error)
//! - **Custom bit depth**: 1-32 bits per dimension
//! - **Dimension reduction**: Random projection for smaller output
//! - **Cross-dimensional comparison**: Project to common space for comparing
//!   vectors of different original dimensions
//!
//! # Random Projection for Dimension Reduction
//!
//! Uses the Johnson-Lindenstrauss lemma: for any set of points in high-dimensional
//! space, there exists a projection to lower dimensions that approximately preserves
//! pairwise distances with high probability.
//!
//! The projection uses a random matrix R where each element r_ij ~ N(0, 1/k)
//! where k is the target dimension. This is generated deterministically from
//! a seed using ChaCha20Rng for reproducibility.
//!
//! # Header Format
//!
//! Full vector ELID uses a 12-byte extended header:
//! - Byte 0: `(version << 4) | 0x04` (profile type)
//! - Byte 1: Reserved (flags)
//! - Bytes 2-3: Original dimensions (u16 big-endian)
//! - Byte 4: Precision type
//! - Byte 5: Dimension mode
//! - Bytes 6-7: Target/common dimensions (u16 big-endian)
//! - Bytes 8-11: Seed lower 32 bits

use blake3;
use half::f16;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use rand_distr::{Distribution, StandardNormal};

use super::error::ElidError;
use super::types::{DimensionMode, ProfileInfo, VectorPrecision};

/// Extended header size for FullVector profile
pub const FULL_VECTOR_HEADER_SIZE: usize = 12;

// ============================================================================
// Random Projection Matrix Generation
// ============================================================================

/// Derive a seed for the random projection matrix
///
/// Creates a unique 32-byte seed for ChaCha20Rng based on the base seed
/// and dimension indices.
#[inline]
fn derive_projection_seed(base_seed: u64, from_dim: u16, to_dim: u16) -> [u8; 32] {
    let mut input = [0u8; 12];
    input[0..8].copy_from_slice(&base_seed.to_le_bytes());
    input[8..10].copy_from_slice(&from_dim.to_le_bytes());
    input[10..12].copy_from_slice(&to_dim.to_le_bytes());

    let hash = blake3::hash(&input);
    *hash.as_bytes()
}

/// Generate a single row of the random projection matrix
///
/// For projecting from `from_dims` to `to_dims`, generates the `row_idx`-th
/// row of the projection matrix. Values are scaled by 1/sqrt(to_dims) to
/// preserve expected norms.
fn generate_projection_row(base_seed: u64, from_dims: u16, to_dims: u16, row_idx: u16) -> Vec<f32> {
    let seed = derive_projection_seed(base_seed, from_dims, row_idx);
    let mut rng = ChaCha20Rng::from_seed(seed);

    // Scale factor for norm preservation (Johnson-Lindenstrauss)
    let scale = 1.0 / (to_dims as f32).sqrt();

    (0..from_dims)
        .map(|_| {
            let val: f32 = StandardNormal.sample(&mut rng);
            val * scale
        })
        .collect()
}

/// Apply random projection to reduce dimensions
///
/// Projects a vector from `from_dims` to `to_dims` using a deterministic
/// random projection matrix generated from the seed.
///
/// # Algorithm
///
/// For each output dimension j:
/// ```text
/// output[j] = sum(input[i] * R[j][i]) for all i
/// ```
///
/// where R is an (to_dims x from_dims) matrix with entries ~ N(0, 1/to_dims)
pub fn project_to_lower_dims(
    input: &[f32],
    to_dims: u16,
    seed: u64,
) -> Result<Vec<f32>, ElidError> {
    let from_dims = input.len() as u16;

    if to_dims == 0 {
        return Err(ElidError::ProjectionError(
            "Target dimensions must be > 0".to_string(),
        ));
    }

    if to_dims > from_dims {
        return Err(ElidError::ProjectionError(format!(
            "Cannot project to higher dimensions: {} -> {}",
            from_dims, to_dims
        )));
    }

    let mut output = Vec::with_capacity(to_dims as usize);

    for row_idx in 0..to_dims {
        let projection_row = generate_projection_row(seed, from_dims, to_dims, row_idx);

        // Dot product of input with this projection row
        let projected_value: f32 = input
            .iter()
            .zip(projection_row.iter())
            .map(|(a, b)| a * b)
            .sum();

        output.push(projected_value);
    }

    Ok(output)
}

/// Apply random projection for cross-dimensional comparison
///
/// Projects a vector to a common dimension space. This handles both
/// up-projection (padding with zeros + projection) and down-projection.
///
/// For comparing vectors of different original dimensions, both vectors
/// should be projected to the same common space using the same seed.
pub fn project_to_common_space(
    input: &[f32],
    original_dims: u16,
    common_dims: u16,
    seed: u64,
) -> Result<Vec<f32>, ElidError> {
    if common_dims == 0 {
        return Err(ElidError::ProjectionError(
            "Common dimensions must be > 0".to_string(),
        ));
    }

    // Always use MAX_COMMON_SOURCE_DIMS as the "virtual" source dimension
    // This ensures vectors of different original dims project consistently
    const MAX_COMMON_SOURCE_DIMS: u16 = 2048;

    // Pad input to virtual dimension (zeros for missing dimensions)
    let mut padded = vec![0.0f32; MAX_COMMON_SOURCE_DIMS as usize];
    for (i, &val) in input.iter().take(original_dims as usize).enumerate() {
        padded[i] = val;
    }

    // Project from virtual dimension to common space
    project_to_lower_dims(&padded, common_dims, seed)
}

// ============================================================================
// Quantization and Dequantization
// ============================================================================

/// Quantize f32 values to specified bit depth
///
/// Maps values from the range [min_val, max_val] to integers in [0, 2^bits - 1].
/// Returns the quantized values along with the min/max used for dequantization.
pub fn quantize_values(values: &[f32], bits: u8) -> Result<(Vec<u32>, f32, f32), ElidError> {
    if bits == 0 || bits > 32 {
        return Err(ElidError::InvalidPrecision(format!(
            "Bits must be 1-32, got {}",
            bits
        )));
    }

    if values.is_empty() {
        return Ok((vec![], 0.0, 0.0));
    }

    // Find value range
    let min_val = values.iter().cloned().fold(f32::INFINITY, f32::min);
    let max_val = values.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

    // Handle constant values
    let range = max_val - min_val;
    if range == 0.0 {
        // All values are the same, encode as middle of range
        let mid = (1u32 << bits) / 2;
        return Ok((vec![mid; values.len()], min_val, max_val));
    }

    let max_quant = ((1u64 << bits) - 1) as f32;

    let quantized: Vec<u32> = values
        .iter()
        .map(|&val| {
            let normalized = (val - min_val) / range;
            let scaled = normalized * max_quant;
            scaled.round().clamp(0.0, max_quant) as u32
        })
        .collect();

    Ok((quantized, min_val, max_val))
}

/// Dequantize values from specified bit depth back to f32
pub fn dequantize_values(quantized: &[u32], bits: u8, min_val: f32, max_val: f32) -> Vec<f32> {
    if quantized.is_empty() {
        return vec![];
    }

    let range = max_val - min_val;
    if range == 0.0 {
        return vec![min_val; quantized.len()];
    }

    let max_quant = ((1u64 << bits) - 1) as f32;

    quantized
        .iter()
        .map(|&q| {
            let normalized = q as f32 / max_quant;
            min_val + normalized * range
        })
        .collect()
}

// ============================================================================
// Full32 Encoding (Lossless)
// ============================================================================

/// Encode values as full 32-bit floats (lossless)
pub fn encode_full32(values: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(values.len() * 4);
    for &val in values {
        bytes.extend_from_slice(&val.to_be_bytes());
    }
    bytes
}

/// Decode full 32-bit float values
pub fn decode_full32(bytes: &[u8]) -> Result<Vec<f32>, ElidError> {
    if !bytes.len().is_multiple_of(4) {
        return Err(ElidError::InvalidEncoding);
    }

    let mut values = Vec::with_capacity(bytes.len() / 4);
    for chunk in bytes.chunks(4) {
        let arr: [u8; 4] = chunk.try_into().map_err(|_| ElidError::InvalidEncoding)?;
        values.push(f32::from_be_bytes(arr));
    }

    Ok(values)
}

// ============================================================================
// Half16 Encoding (16-bit half precision)
// ============================================================================

/// Encode values as 16-bit half-precision floats
pub fn encode_half16(values: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(values.len() * 2);
    for &val in values {
        let half = f16::from_f32(val);
        bytes.extend_from_slice(&half.to_be_bytes());
    }
    bytes
}

/// Decode 16-bit half-precision float values
pub fn decode_half16(bytes: &[u8]) -> Result<Vec<f32>, ElidError> {
    if !bytes.len().is_multiple_of(2) {
        return Err(ElidError::InvalidEncoding);
    }

    let mut values = Vec::with_capacity(bytes.len() / 2);
    for chunk in bytes.chunks(2) {
        let arr: [u8; 2] = chunk.try_into().map_err(|_| ElidError::InvalidEncoding)?;
        let half = f16::from_be_bytes(arr);
        values.push(half.to_f32());
    }

    Ok(values)
}

// ============================================================================
// Quant8 Encoding (8-bit uniform quantization)
// ============================================================================

/// Encode values as 8-bit quantized (includes min/max header)
///
/// Format: [min_f32 (4 bytes)][max_f32 (4 bytes)][quantized values (N bytes)]
pub fn encode_quant8(values: &[f32]) -> Result<Vec<u8>, ElidError> {
    let (quantized, min_val, max_val) = quantize_values(values, 8)?;

    let mut bytes = Vec::with_capacity(8 + values.len());
    bytes.extend_from_slice(&min_val.to_be_bytes());
    bytes.extend_from_slice(&max_val.to_be_bytes());

    for q in quantized {
        bytes.push(q as u8);
    }

    Ok(bytes)
}

/// Decode 8-bit quantized values
pub fn decode_quant8(bytes: &[u8]) -> Result<Vec<f32>, ElidError> {
    if bytes.len() < 8 {
        return Err(ElidError::InsufficientData {
            expected: 8,
            got: bytes.len(),
        });
    }

    let min_val = f32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let max_val = f32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);

    let quantized: Vec<u32> = bytes[8..].iter().map(|&b| b as u32).collect();

    Ok(dequantize_values(&quantized, 8, min_val, max_val))
}

// ============================================================================
// Custom Bits Encoding
// ============================================================================

/// Encode values with custom bit depth
///
/// Format: [min_f32 (4 bytes)][max_f32 (4 bytes)][packed bits]
///
/// Bits are packed in big-endian order (MSB first).
pub fn encode_bits(values: &[f32], bits: u8) -> Result<Vec<u8>, ElidError> {
    let (quantized, min_val, max_val) = quantize_values(values, bits)?;

    // Calculate total bits and bytes needed
    let total_bits = values.len() * bits as usize;
    let total_bytes = total_bits.div_ceil(8);

    let mut bytes = Vec::with_capacity(8 + total_bytes);
    bytes.extend_from_slice(&min_val.to_be_bytes());
    bytes.extend_from_slice(&max_val.to_be_bytes());

    // Pack bits into bytes
    let mut packed = vec![0u8; total_bytes];
    for (i, &q) in quantized.iter().enumerate() {
        let bit_offset = i * bits as usize;
        pack_value(&mut packed, q, bit_offset, bits);
    }

    bytes.extend_from_slice(&packed);
    Ok(bytes)
}

/// Decode values from custom bit depth
pub fn decode_bits(bytes: &[u8], bits: u8, num_values: usize) -> Result<Vec<f32>, ElidError> {
    if bytes.len() < 8 {
        return Err(ElidError::InsufficientData {
            expected: 8,
            got: bytes.len(),
        });
    }

    let min_val = f32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let max_val = f32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);

    let packed = &bytes[8..];

    // Unpack bits
    let mut quantized = Vec::with_capacity(num_values);
    for i in 0..num_values {
        let bit_offset = i * bits as usize;
        let value = unpack_value(packed, bit_offset, bits);
        quantized.push(value);
    }

    Ok(dequantize_values(&quantized, bits, min_val, max_val))
}

/// Pack a value into a byte array at the given bit offset
fn pack_value(bytes: &mut [u8], value: u32, bit_offset: usize, bits: u8) {
    let bits = bits as usize;
    let byte_idx = bit_offset / 8;
    let bit_within_byte = bit_offset % 8;

    // Value may span multiple bytes
    let mut remaining_bits = bits;
    let mut current_byte = byte_idx;
    let mut current_bit = bit_within_byte;

    while remaining_bits > 0 && current_byte < bytes.len() {
        let bits_in_this_byte = (8 - current_bit).min(remaining_bits);
        let mask = ((1u32 << bits_in_this_byte) - 1) as u8;

        // Extract bits from the value (starting from MSB)
        let shift = remaining_bits - bits_in_this_byte;
        let extracted = ((value >> shift) & mask as u32) as u8;

        // Place in the byte
        let byte_shift = 8 - current_bit - bits_in_this_byte;
        bytes[current_byte] |= extracted << byte_shift;

        remaining_bits -= bits_in_this_byte;
        current_byte += 1;
        current_bit = 0;
    }
}

/// Unpack a value from a byte array at the given bit offset
fn unpack_value(bytes: &[u8], bit_offset: usize, bits: u8) -> u32 {
    let bits = bits as usize;
    let byte_idx = bit_offset / 8;
    let bit_within_byte = bit_offset % 8;

    let mut value = 0u32;
    let mut remaining_bits = bits;
    let mut current_byte = byte_idx;
    let mut current_bit = bit_within_byte;

    while remaining_bits > 0 && current_byte < bytes.len() {
        let bits_in_this_byte = (8 - current_bit).min(remaining_bits);

        // Extract bits from this byte
        let byte_shift = 8 - current_bit - bits_in_this_byte;
        let mask = ((1u32 << bits_in_this_byte) - 1) as u8;
        let extracted = (bytes[current_byte] >> byte_shift) & mask;

        // Shift value and add extracted bits
        value = (value << bits_in_this_byte) | extracted as u32;

        remaining_bits -= bits_in_this_byte;
        current_byte += 1;
        current_bit = 0;
    }

    value
}

// ============================================================================
// Main Encoding/Decoding Functions
// ============================================================================

/// Encode a full vector embedding
///
/// # Parameters
///
/// - `embedding`: Input embedding values
/// - `precision`: Precision level for encoding
/// - `dimensions`: Dimension handling mode
/// - `seed`: Seed for random projection (if dimension reduction is used)
///
/// # Returns
///
/// Encoded bytes (header + payload)
pub fn encode_full_vector(
    embedding: &[f32],
    precision: VectorPrecision,
    dimensions: DimensionMode,
    seed: u64,
) -> Result<Vec<u8>, ElidError> {
    let original_dims = embedding.len() as u16;

    // Validate inputs
    precision.validate()?;
    dimensions.validate(original_dims)?;

    // Apply dimension transformation if needed
    let values_to_encode: Vec<f32> = match dimensions {
        DimensionMode::Preserve => embedding.to_vec(),
        DimensionMode::Reduce { target_dims } => {
            project_to_lower_dims(embedding, target_dims, seed)?
        }
        DimensionMode::Common { dims } => {
            project_to_common_space(embedding, original_dims, dims, seed)?
        }
    };

    // Create header
    let header_info = ProfileInfo::from_full_vector(original_dims, precision, dimensions, seed);
    let header = header_info.to_header();

    // Encode payload based on precision
    let payload = match precision {
        VectorPrecision::Full32 => encode_full32(&values_to_encode),
        VectorPrecision::Half16 => encode_half16(&values_to_encode),
        VectorPrecision::Quant8 => encode_quant8(&values_to_encode)?,
        VectorPrecision::Bits { bits } => encode_bits(&values_to_encode, bits)?,
    };

    // Combine header and payload
    let mut result = header;
    result.extend_from_slice(&payload);

    Ok(result)
}

/// Decode a full vector embedding from bytes
///
/// # Parameters
///
/// - `bytes`: Encoded bytes (header + payload)
///
/// # Returns
///
/// Tuple of (decoded values, metadata)
///
/// Note: If dimension reduction was used, the decoded values are in the
/// reduced space and cannot be directly compared to the original embedding.
/// The original dimensions are stored in the metadata.
pub fn decode_full_vector(bytes: &[u8]) -> Result<(Vec<f32>, FullVectorMetadata), ElidError> {
    if bytes.len() < FULL_VECTOR_HEADER_SIZE {
        return Err(ElidError::InvalidHeader);
    }

    // Parse header
    let header_info = ProfileInfo::from_header(bytes)?;

    if header_info.profile_type != 0x04 {
        return Err(ElidError::ProfileMismatch {
            expected: "FullVector".to_string(),
            got: format!("Type {:#x}", header_info.profile_type),
        });
    }

    let original_dims = header_info
        .original_dims
        .ok_or_else(|| ElidError::InvalidMetadata("Missing original dimensions".to_string()))?;

    let precision = header_info
        .precision
        .ok_or_else(|| ElidError::InvalidMetadata("Missing precision".to_string()))?;

    let dimension_mode = header_info
        .dimension_mode
        .ok_or_else(|| ElidError::InvalidMetadata("Missing dimension mode".to_string()))?;

    let seed = header_info.seed.unwrap_or(0);

    // Determine number of encoded values
    let encoded_dims = dimension_mode.output_dims(original_dims) as usize;

    // Decode payload
    let payload = &bytes[FULL_VECTOR_HEADER_SIZE..];

    let decoded_values = match precision {
        VectorPrecision::Full32 => decode_full32(payload)?,
        VectorPrecision::Half16 => decode_half16(payload)?,
        VectorPrecision::Quant8 => decode_quant8(payload)?,
        VectorPrecision::Bits { bits } => decode_bits(payload, bits, encoded_dims)?,
    };

    let metadata = FullVectorMetadata {
        original_dims,
        encoded_dims: encoded_dims as u16,
        precision,
        dimension_mode,
        seed,
    };

    Ok((decoded_values, metadata))
}

/// Metadata about a decoded full vector
#[derive(Clone, Debug, PartialEq)]
pub struct FullVectorMetadata {
    /// Original embedding dimension count
    pub original_dims: u16,
    /// Number of dimensions in the encoded representation
    pub encoded_dims: u16,
    /// Precision used for encoding
    pub precision: VectorPrecision,
    /// Dimension handling mode
    pub dimension_mode: DimensionMode,
    /// Seed used for random projection
    pub seed: u64,
}

impl FullVectorMetadata {
    /// Check if the encoding is lossless (exact reconstruction possible)
    pub fn is_lossless(&self) -> bool {
        matches!(self.precision, VectorPrecision::Full32)
            && matches!(self.dimension_mode, DimensionMode::Preserve)
    }

    /// Check if dimension reduction was applied
    pub fn has_dimension_reduction(&self) -> bool {
        self.encoded_dims < self.original_dims
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Quantization Tests
    // ========================================================================

    #[test]
    fn test_quantize_roundtrip_8bit() {
        let values = vec![-1.0, -0.5, 0.0, 0.5, 1.0];
        let (quantized, min_val, max_val) = quantize_values(&values, 8).unwrap();

        let recovered = dequantize_values(&quantized, 8, min_val, max_val);

        for (orig, rec) in values.iter().zip(recovered.iter()) {
            let error = (orig - rec).abs();
            assert!(
                error < 0.01,
                "8-bit roundtrip error too large: {} vs {}",
                orig,
                rec
            );
        }
    }

    #[test]
    fn test_quantize_constant_values() {
        let values = vec![0.5, 0.5, 0.5, 0.5];
        let (quantized, min_val, max_val) = quantize_values(&values, 8).unwrap();

        let recovered = dequantize_values(&quantized, 8, min_val, max_val);

        for (orig, rec) in values.iter().zip(recovered.iter()) {
            assert_eq!(*orig, *rec, "Constant values should recover exactly");
        }
    }

    #[test]
    fn test_quantize_empty() {
        let values: Vec<f32> = vec![];
        let (quantized, _, _) = quantize_values(&values, 8).unwrap();
        assert!(quantized.is_empty());
    }

    // ========================================================================
    // Full32 Tests
    // ========================================================================

    #[test]
    fn test_full32_roundtrip() {
        let values = vec![0.123456, -0.987654, std::f32::consts::PI, 0.0];
        let encoded = encode_full32(&values);
        let decoded = decode_full32(&encoded).unwrap();

        assert_eq!(values, decoded, "Full32 should be lossless");
    }

    // ========================================================================
    // Half16 Tests
    // ========================================================================

    #[test]
    fn test_half16_roundtrip() {
        let values = vec![0.123, -0.987, 3.14, 0.0];
        let encoded = encode_half16(&values);
        let decoded = decode_half16(&encoded).unwrap();

        for (orig, rec) in values.iter().zip(decoded.iter()) {
            let error = (orig - rec).abs() / orig.abs().max(1e-6);
            assert!(
                error < 0.01,
                "Half16 relative error too large: {} vs {}",
                orig,
                rec
            );
        }
    }

    // ========================================================================
    // Quant8 Tests
    // ========================================================================

    #[test]
    fn test_quant8_roundtrip() {
        let values: Vec<f32> = (0..256).map(|i| (i as f32 / 128.0) - 1.0).collect();
        let encoded = encode_quant8(&values).unwrap();
        let decoded = decode_quant8(&encoded).unwrap();

        for (orig, rec) in values.iter().zip(decoded.iter()) {
            let error = (orig - rec).abs();
            assert!(error < 0.02, "Quant8 error too large: {} vs {}", orig, rec);
        }
    }

    // ========================================================================
    // Custom Bits Tests
    // ========================================================================

    #[test]
    fn test_bits_roundtrip_4bit() {
        let values = vec![-1.0, -0.5, 0.0, 0.5, 1.0];
        let encoded = encode_bits(&values, 4).unwrap();
        let decoded = decode_bits(&encoded, 4, values.len()).unwrap();

        for (orig, rec) in values.iter().zip(decoded.iter()) {
            let error = (orig - rec).abs();
            // 4 bits = 16 levels, so error ~1/16 = 0.0625
            assert!(
                error < 0.15,
                "4-bit roundtrip error too large: {} vs {}",
                orig,
                rec
            );
        }
    }

    #[test]
    fn test_bits_packing() {
        // Test that bit packing works correctly
        let mut bytes = vec![0u8; 3];

        // Pack 5 values of 4 bits each = 20 bits = 3 bytes (with 4 unused bits)
        pack_value(&mut bytes, 0b1010, 0, 4); // bits 0-3
        pack_value(&mut bytes, 0b0101, 4, 4); // bits 4-7
        pack_value(&mut bytes, 0b1100, 8, 4); // bits 8-11
        pack_value(&mut bytes, 0b0011, 12, 4); // bits 12-15
        pack_value(&mut bytes, 0b1111, 16, 4); // bits 16-19

        // Verify unpacking
        assert_eq!(unpack_value(&bytes, 0, 4), 0b1010);
        assert_eq!(unpack_value(&bytes, 4, 4), 0b0101);
        assert_eq!(unpack_value(&bytes, 8, 4), 0b1100);
        assert_eq!(unpack_value(&bytes, 12, 4), 0b0011);
        assert_eq!(unpack_value(&bytes, 16, 4), 0b1111);
    }

    // ========================================================================
    // Random Projection Tests
    // ========================================================================

    #[test]
    fn test_projection_deterministic() {
        let input: Vec<f32> = (0..768).map(|i| (i as f32) / 768.0).collect();
        let seed = 0x12345678;

        let proj1 = project_to_lower_dims(&input, 256, seed).unwrap();
        let proj2 = project_to_lower_dims(&input, 256, seed).unwrap();

        assert_eq!(proj1, proj2, "Projection must be deterministic");
    }

    #[test]
    fn test_projection_different_seeds() {
        let input: Vec<f32> = (0..768).map(|i| (i as f32) / 768.0).collect();

        let proj1 = project_to_lower_dims(&input, 256, 0x1111).unwrap();
        let proj2 = project_to_lower_dims(&input, 256, 0x2222).unwrap();

        assert_ne!(
            proj1, proj2,
            "Different seeds should give different projections"
        );
    }

    #[test]
    fn test_projection_preserves_similarity() {
        // Johnson-Lindenstrauss: distances should be approximately preserved
        let seed = 0x454c4944;

        // Create two similar vectors
        let v1: Vec<f32> = (0..768).map(|i| (i as f32) / 768.0).collect();
        let mut v2 = v1.clone();
        v2[0] += 0.01; // Small perturbation

        // Create a dissimilar vector
        let v3: Vec<f32> = (0..768).map(|i| -((i as f32) / 768.0)).collect();

        // Project all
        let p1 = project_to_lower_dims(&v1, 128, seed).unwrap();
        let p2 = project_to_lower_dims(&v2, 128, seed).unwrap();
        let p3 = project_to_lower_dims(&v3, 128, seed).unwrap();

        // Check that similar vectors remain closer than dissimilar
        let dist_12: f32 = p1.iter().zip(p2.iter()).map(|(a, b)| (a - b).powi(2)).sum();
        let dist_13: f32 = p1.iter().zip(p3.iter()).map(|(a, b)| (a - b).powi(2)).sum();

        assert!(
            dist_12 < dist_13,
            "Similar vectors should remain closer after projection"
        );
    }

    // ========================================================================
    // Full Vector Encoding Tests
    // ========================================================================

    #[test]
    fn test_full_vector_lossless_roundtrip() {
        let embedding: Vec<f32> = (0..128).map(|i| (i as f32 / 64.0) - 1.0).collect();

        let encoded = encode_full_vector(
            &embedding,
            VectorPrecision::Full32,
            DimensionMode::Preserve,
            0,
        )
        .unwrap();

        let (decoded, metadata) = decode_full_vector(&encoded).unwrap();

        assert_eq!(
            embedding, decoded,
            "Lossless encoding should preserve exact values"
        );
        assert!(metadata.is_lossless());
        assert_eq!(metadata.original_dims, 128);
        assert_eq!(metadata.encoded_dims, 128);
    }

    #[test]
    fn test_full_vector_half16_roundtrip() {
        let embedding: Vec<f32> = (0..768).map(|i| (i as f32 / 384.0) - 1.0).collect();

        let encoded = encode_full_vector(
            &embedding,
            VectorPrecision::Half16,
            DimensionMode::Preserve,
            0,
        )
        .unwrap();

        let (decoded, metadata) = decode_full_vector(&encoded).unwrap();

        assert_eq!(embedding.len(), decoded.len());
        assert!(!metadata.is_lossless());

        // Check error is reasonable
        for (orig, rec) in embedding.iter().zip(decoded.iter()) {
            let error = (orig - rec).abs();
            assert!(error < 0.01, "Half16 error too large: {} vs {}", orig, rec);
        }
    }

    #[test]
    fn test_full_vector_dimension_reduction() {
        let embedding: Vec<f32> = (0..768).map(|i| (i as f32 / 384.0) - 1.0).collect();

        let encoded = encode_full_vector(
            &embedding,
            VectorPrecision::Full32,
            DimensionMode::Reduce { target_dims: 256 },
            0x12345678,
        )
        .unwrap();

        let (decoded, metadata) = decode_full_vector(&encoded).unwrap();

        assert_eq!(decoded.len(), 256);
        assert_eq!(metadata.original_dims, 768);
        assert_eq!(metadata.encoded_dims, 256);
        assert!(metadata.has_dimension_reduction());
    }

    #[test]
    fn test_full_vector_common_space() {
        let seed = 0x454c4944_58444949;

        // Two embeddings of different dimensions
        let emb_256: Vec<f32> = (0..256).map(|i| (i as f32 / 128.0) - 1.0).collect();
        let emb_768: Vec<f32> = (0..768).map(|i| (i as f32 / 384.0) - 1.0).collect();

        // Encode both to common 128-dim space
        let enc_256 = encode_full_vector(
            &emb_256,
            VectorPrecision::Full32,
            DimensionMode::Common { dims: 128 },
            seed,
        )
        .unwrap();

        let enc_768 = encode_full_vector(
            &emb_768,
            VectorPrecision::Full32,
            DimensionMode::Common { dims: 128 },
            seed,
        )
        .unwrap();

        let (dec_256, meta_256) = decode_full_vector(&enc_256).unwrap();
        let (dec_768, meta_768) = decode_full_vector(&enc_768).unwrap();

        // Both should be 128-dimensional
        assert_eq!(dec_256.len(), 128);
        assert_eq!(dec_768.len(), 128);

        // Metadata should reflect original dimensions
        assert_eq!(meta_256.original_dims, 256);
        assert_eq!(meta_768.original_dims, 768);

        // Now they can be compared directly (same dimensionality)
        let _similarity: f32 = dec_256.iter().zip(dec_768.iter()).map(|(a, b)| a * b).sum();
        // Note: actual similarity depends on the vectors and projection
    }

    // ========================================================================
    // Profile Convenience Constructor Tests
    // ========================================================================

    #[test]
    fn test_profile_lossless() {
        let profile = super::super::types::Profile::lossless();
        match profile {
            super::super::types::Profile::FullVector {
                precision,
                dimensions,
                ..
            } => {
                assert_eq!(precision, VectorPrecision::Full32);
                assert_eq!(dimensions, DimensionMode::Preserve);
            }
            _ => panic!("Expected FullVector profile"),
        }
    }

    #[test]
    fn test_profile_compressed() {
        let profile = super::super::types::Profile::compressed(0.5, 768);
        match profile {
            super::super::types::Profile::FullVector {
                precision,
                dimensions,
                ..
            } => {
                // At 50% retention, should use Full32 with ~384 dims
                // or Half16 with ~768 dims
                let output_dims = dimensions.output_dims(768);
                let total_bits = (output_dims as usize) * (precision.bits_per_dim() as usize);
                let full_bits = 768 * 32;
                let retention = total_bits as f32 / full_bits as f32;

                // Should be approximately 50% (with some tolerance for rounding)
                assert!(
                    retention > 0.4 && retention < 0.6,
                    "Retention should be ~50%, got {}%",
                    retention * 100.0
                );
            }
            _ => panic!("Expected FullVector profile"),
        }
    }

    #[test]
    fn test_profile_max_length() {
        let profile = super::super::types::Profile::max_length(100, 768);
        match profile {
            super::super::types::Profile::FullVector { .. } => {
                // String length should be <= 100 chars
                let string_len = profile.string_length_for_dims(768);
                assert!(
                    string_len <= 100,
                    "String length {} exceeds max {}",
                    string_len,
                    100
                );
            }
            _ => panic!("Expected FullVector profile"),
        }
    }

    #[test]
    fn test_profile_cross_dimensional() {
        let profile = super::super::types::Profile::cross_dimensional(128);
        match profile {
            super::super::types::Profile::FullVector {
                precision,
                dimensions,
                ..
            } => {
                assert_eq!(precision, VectorPrecision::Half16);
                assert_eq!(dimensions, DimensionMode::Common { dims: 128 });
            }
            _ => panic!("Expected FullVector profile"),
        }
    }
}
