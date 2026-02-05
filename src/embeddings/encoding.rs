//! Base32hex encoding for ELID strings
//!
//! This module provides sortable base32hex encoding/decoding using the RFC 4648
//! base32hex alphabet (0-9, a-v) which preserves lexicographic sort order.
//!
//! # Sort Order Preservation
//!
//! The base32hex alphabet is in strict ASCII order:
//! - '0' (48) < '1' (49) < ... < '9' (57) < 'a' (97) < ... < 'v' (118)
//!
//! This ensures that encoding big-endian bytes to base32hex preserves lexicographic
//! sort order, making ELID strings suitable for database B-tree indexes.

use data_encoding::{Encoding, Specification};
use once_cell::sync::Lazy;

use super::error::ElidError;

/// Lowercase base32hex encoding without padding
///
/// Uses RFC 4648 base32hex alphabet: "0123456789abcdefghijklmnopqrstuv"
/// - No padding (no trailing '=' characters)
/// - Lowercase for case-insensitive filesystem compatibility
/// - Sort-order preserving for database indexing
pub static BASE32HEX_LOWER_NOPAD: Lazy<Encoding> = Lazy::new(|| {
    let mut spec = Specification::new();
    spec.symbols.push_str("0123456789abcdefghijklmnopqrstuv");
    spec.encoding().expect("Invalid base32hex specification")
});

/// Encode bytes to sortable base32hex string
///
/// Encodes big-endian bytes to lowercase base32hex without padding.
/// The result preserves lexicographic sort order.
///
/// # Arguments
///
/// * `bytes` - Slice of bytes to encode (typically big-endian representation)
///
/// # Returns
///
/// Base32hex encoded string (lowercase, no padding)
///
/// # Length
///
/// Output length is ceil(bits / 5) characters (5 bits per base32 character):
/// - 128 bits (16 bytes) = 26 characters
/// - 100 bits (13 bytes) = 21 characters
/// - 64 bits (8 bytes) = 13 characters
///
/// # Example
///
/// ```rust,ignore
/// use crate::embeddings::encoding::encode_sortable;
///
/// let bytes = vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef];
/// let encoded = encode_sortable(&bytes);
/// assert_eq!(encoded, "04h95d7ha6bv"); // lowercase, no padding
/// ```
pub fn encode_sortable(bytes: &[u8]) -> String {
    BASE32HEX_LOWER_NOPAD.encode(bytes)
}

/// Decode sortable base32hex string to bytes
///
/// Decodes lowercase base32hex (without padding) back to bytes.
///
/// # Arguments
///
/// * `s` - Base32hex string to decode
///
/// # Returns
///
/// * `Ok(Vec<u8>)` - Decoded bytes (big-endian)
/// * `Err(ElidError::InvalidEncoding)` - Invalid base32hex characters or format
///
/// # Example
///
/// ```rust,ignore
/// use crate::embeddings::encoding::decode_sortable;
///
/// let encoded = "04h95d7ha6bv";
/// let bytes = decode_sortable(encoded)?;
/// assert_eq!(bytes, vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef]);
/// ```
pub fn decode_sortable(s: &str) -> Result<Vec<u8>, ElidError> {
    BASE32HEX_LOWER_NOPAD
        .decode(s.as_bytes())
        .map_err(|_| ElidError::InvalidEncoding)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip() {
        let original = vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef];
        let encoded = encode_sortable(&original);
        let decoded = decode_sortable(&encoded).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_sort_order_preservation() {
        // Test that lexicographic sort order is preserved
        let bytes1 = vec![0x00, 0x00, 0x00, 0x01];
        let bytes2 = vec![0x00, 0x00, 0x00, 0x02];
        let bytes3 = vec![0x00, 0x00, 0x00, 0xFF];

        let enc1 = encode_sortable(&bytes1);
        let enc2 = encode_sortable(&bytes2);
        let enc3 = encode_sortable(&bytes3);

        assert!(enc1 < enc2);
        assert!(enc2 < enc3);
        assert!(enc1 < enc3);
    }

    #[test]
    fn test_128_bit_length() {
        // 128 bits = 16 bytes = 26 characters
        let bytes = vec![0u8; 16];
        let encoded = encode_sortable(&bytes);
        assert_eq!(encoded.len(), 26);
    }

    #[test]
    fn test_100_bit_length() {
        // 100 bits approximately 13 bytes = 21 characters
        let bytes = vec![0u8; 13];
        let encoded = encode_sortable(&bytes);
        assert_eq!(encoded.len(), 21);
    }

    #[test]
    fn test_lowercase_alphabet() {
        let bytes = vec![0xFF; 8];
        let encoded = encode_sortable(&bytes);
        // Should only contain lowercase characters and digits
        assert!(encoded
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()));
    }

    #[test]
    fn test_no_padding() {
        let bytes = vec![0x01, 0x02, 0x03];
        let encoded = encode_sortable(&bytes);
        // Should not contain padding character '='
        assert!(!encoded.contains('='));
    }

    #[test]
    fn test_invalid_encoding() {
        // Base32hex alphabet is 0-9, a-v (no w, x, y, z)
        let result = decode_sortable("xyz123");
        assert!(result.is_err());
        match result {
            Err(ElidError::InvalidEncoding) => {}
            _ => panic!("Expected InvalidEncoding error"),
        }
    }
}
