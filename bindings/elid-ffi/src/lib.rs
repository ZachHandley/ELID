//! FFI layer for ELID language bindings
//!
//! This crate provides C-compatible exports for use by language bindings.
//! It includes both basic C exports for PHP FFI and UniFFI for Swift/Kotlin/Ruby bindings.

use elid_core::{decode, encode, hamming_distance, Elid, Profile};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

// ============================================================================
// UniFFI Bindings (for Swift, Kotlin, Ruby)
// ============================================================================

/// ELID encoding profile (UniFFI version)
#[derive(Debug, Clone)]
pub enum UniProfile {
    Mini128,
    Morton10x10,
    Hilbert10x10,
}

impl UniProfile {
    fn to_core(&self) -> Profile {
        match self {
            UniProfile::Mini128 => Profile::Mini128 {
                seed: 0x454c4944_53494d48,
            },
            UniProfile::Morton10x10 => Profile::Morton10x10 {
                dims: 10,
                bits_per_dim: 10,
                transform_id: None,
            },
            UniProfile::Hilbert10x10 => Profile::Hilbert10x10 {
                dims: 10,
                bits_per_dim: 10,
                transform_id: None,
            },
        }
    }
}

/// Errors for UniFFI bindings
#[derive(Debug, thiserror::Error)]
pub enum UniElidError {
    #[error("Invalid embedding dimension (must be 64-2048)")]
    InvalidDimension,
    #[error("Invalid embedding value (NaN or Inf)")]
    InvalidValue,
    #[error("Invalid base32hex encoding")]
    InvalidEncoding,
    #[error("Invalid ELID header")]
    InvalidHeader,
    #[error("Profile mismatch (Hamming distance requires Mini128)")]
    ProfileMismatch,
    #[error("Transform not found")]
    TransformNotFound,
}

impl From<elid_core::ElidError> for UniElidError {
    fn from(err: elid_core::ElidError) -> Self {
        match err {
            elid_core::ElidError::InvalidDimension { .. } => UniElidError::InvalidDimension,
            elid_core::ElidError::InvalidValue => UniElidError::InvalidValue,
            elid_core::ElidError::InvalidEncoding => UniElidError::InvalidEncoding,
            elid_core::ElidError::InvalidHeader => UniElidError::InvalidHeader,
            elid_core::ElidError::ProfileMismatch { .. } => UniElidError::ProfileMismatch,
            elid_core::ElidError::TransformNotFound(_) => UniElidError::TransformNotFound,
            elid_core::ElidError::RadiusTooLarge(_) => UniElidError::InvalidValue,
        }
    }
}

/// Encode an embedding into an ELID string (UniFFI version)
pub fn uni_encode(embedding: Vec<f32>, profile: UniProfile) -> Result<String, UniElidError> {
    let core_profile = profile.to_core();
    let elid = encode(&embedding, &core_profile)?;
    Ok(elid.as_str().to_string())
}

/// Decode an ELID string to raw bytes (UniFFI version)
pub fn uni_decode(elid: String) -> Result<Vec<u8>, UniElidError> {
    let elid_obj = Elid::from_string(elid)?;
    let bytes = decode(&elid_obj)?;
    Ok(bytes)
}

/// Compute Hamming distance between two Mini128 ELIDs (UniFFI version)
pub fn uni_hamming_distance(elid1: String, elid2: String) -> Result<u32, UniElidError> {
    let elid_a = Elid::from_string(elid1)?;
    let elid_b = Elid::from_string(elid2)?;
    let distance = hamming_distance(&elid_a, &elid_b)?;
    Ok(distance)
}

/// Encode a batch of embeddings (UniFFI version)
pub fn uni_encode_batch(embeddings: Vec<Vec<f32>>, profile: UniProfile) -> Result<Vec<String>, UniElidError> {
    let core_profile = profile.to_core();
    let mut elids = Vec::with_capacity(embeddings.len());
    for embedding in embeddings {
        let elid = encode(&embedding, &core_profile)?;
        elids.push(elid.as_str().to_string());
    }
    Ok(elids)
}

// Include UniFFI scaffolding
uniffi::include_scaffolding!("elid");

// ============================================================================
// C-Compatible Basic FFI (for PHP and fallback)
// ============================================================================

/// Encode an embedding into an ELID string
///
/// # Safety
///
/// - `embedding` must be a valid pointer to an array of f32 with `len` elements
/// - Caller must call `elid_free_string` on the returned pointer
/// - Returns NULL on error
#[no_mangle]
pub unsafe extern "C" fn elid_encode(
    embedding: *const f32,
    len: usize,
    profile_type: u8,
) -> *mut c_char {
    if embedding.is_null() || len == 0 {
        return ptr::null_mut();
    }

    // Convert C array to Rust slice
    let embedding_slice = std::slice::from_raw_parts(embedding, len);

    // Create profile (0=Mini128, 1=Morton10x10, 2=Hilbert10x10)
    let profile = match profile_type {
        0 => Profile::Mini128 { seed: 0 },
        1 => Profile::Morton10x10 {
            dims: 10,
            bits_per_dim: 10,
            transform_id: None,
        },
        2 => Profile::Hilbert10x10 {
            dims: 10,
            bits_per_dim: 10,
            transform_id: None,
        },
        _ => return ptr::null_mut(),
    };

    // Encode
    match encode(embedding_slice, &profile) {
        Ok(elid) => {
            // Convert to C string
            match CString::new(elid.as_str()) {
                Ok(c_string) => c_string.into_raw(),
                Err(_) => ptr::null_mut(),
            }
        }
        Err(_) => ptr::null_mut(),
    }
}

/// Decode an ELID string to bytes
///
/// # Safety
///
/// - `elid_str` must be a valid null-terminated C string
/// - Caller must call `elid_free_bytes` on the returned pointer
/// - `out_len` will be set to the number of bytes
/// - Returns NULL on error
#[no_mangle]
pub unsafe extern "C" fn elid_decode(elid_str: *const c_char, out_len: *mut usize) -> *mut u8 {
    if elid_str.is_null() || out_len.is_null() {
        return ptr::null_mut();
    }

    // Convert C string to Rust string
    let c_str = match CStr::from_ptr(elid_str).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    // Create Elid
    let elid = match Elid::from_string(c_str.to_string()) {
        Ok(e) => e,
        Err(_) => return ptr::null_mut(),
    };

    // Decode
    match decode(&elid) {
        Ok(bytes) => {
            *out_len = bytes.len();
            let boxed_slice = bytes.into_boxed_slice();
            Box::into_raw(boxed_slice) as *mut u8
        }
        Err(_) => ptr::null_mut(),
    }
}

/// Compute Hamming distance between two ELIDs
///
/// # Safety
///
/// - Both `elid1` and `elid2` must be valid null-terminated C strings
/// - Returns u32::MAX on error
#[no_mangle]
pub unsafe extern "C" fn elid_hamming_distance(elid1: *const c_char, elid2: *const c_char) -> u32 {
    if elid1.is_null() || elid2.is_null() {
        return u32::MAX;
    }

    // Convert C strings
    let c_str1 = match CStr::from_ptr(elid1).to_str() {
        Ok(s) => s,
        Err(_) => return u32::MAX,
    };
    let c_str2 = match CStr::from_ptr(elid2).to_str() {
        Ok(s) => s,
        Err(_) => return u32::MAX,
    };

    // Create Elids
    let elid_a = match Elid::from_string(c_str1.to_string()) {
        Ok(e) => e,
        Err(_) => return u32::MAX,
    };
    let elid_b = match Elid::from_string(c_str2.to_string()) {
        Ok(e) => e,
        Err(_) => return u32::MAX,
    };

    // Compute distance
    hamming_distance(&elid_a, &elid_b).unwrap_or(u32::MAX)
}

/// Free a string allocated by elid_encode
///
/// # Safety
///
/// - `s` must be a pointer previously returned by elid_encode
/// - Must only be called once per pointer
#[no_mangle]
pub unsafe extern "C" fn elid_free_string(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}

/// Free bytes allocated by elid_decode
///
/// # Safety
///
/// - `bytes` must be a pointer previously returned by elid_decode
/// - `len` must match the length returned by elid_decode
/// - Must only be called once per pointer
#[no_mangle]
pub unsafe extern "C" fn elid_free_bytes(bytes: *mut u8, len: usize) {
    if !bytes.is_null() && len > 0 {
        drop(Box::from_raw(std::slice::from_raw_parts_mut(
            bytes, len,
        )));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_ffi_encode_decode() {
        unsafe {
            // Create test embedding
            let embedding: Vec<f32> = (0..768).map(|i| i as f32 / 768.0).collect();

            // Encode
            let result = elid_encode(embedding.as_ptr(), embedding.len(), 0);
            assert!(!result.is_null());

            // Convert back to Rust string
            let c_str = CStr::from_ptr(result);
            let rust_str = c_str.to_str().unwrap();
            assert_eq!(rust_str.len(), 29); // Mini128 produces 29-char strings

            // Decode
            let mut out_len: usize = 0;
            let bytes_ptr = elid_decode(result, &mut out_len);
            assert!(!bytes_ptr.is_null());
            assert!(out_len > 0);

            // Cleanup
            elid_free_bytes(bytes_ptr, out_len);
            elid_free_string(result);
        }
    }

    #[test]
    fn test_ffi_hamming_distance() {
        unsafe {
            let embedding1: Vec<f32> = vec![0.1; 768];
            let embedding2: Vec<f32> = vec![0.2; 768];

            let elid1 = elid_encode(embedding1.as_ptr(), embedding1.len(), 0);
            let elid2 = elid_encode(embedding2.as_ptr(), embedding2.len(), 0);

            assert!(!elid1.is_null());
            assert!(!elid2.is_null());

            let distance = elid_hamming_distance(elid1, elid2);
            assert!(distance < 128); // Distance should be valid (< 128 bits)

            elid_free_string(elid1);
            elid_free_string(elid2);
        }
    }
}
