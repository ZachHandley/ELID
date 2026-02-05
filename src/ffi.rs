//! C FFI bindings for ELID
//!
//! This module provides C-compatible function bindings that can be used from C, C++,
//! Swift, Objective-C, Go, Ruby, and many other languages.
//!
//! # Memory Management
//!
//! Strings returned from these functions are allocated by Rust and must be freed
//! using `elid_free_string()`. Failure to do so will cause memory leaks.
//!
//! # Safety
//!
//! All functions check for NULL pointers and return safe defaults if invalid input is provided.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// Represents a match result with index and score
#[repr(C)]
pub struct ElidMatch {
    /// Index of the matched string in the candidates array
    pub index: usize,
    /// Similarity score (0.0 to 1.0)
    pub score: f64,
}

/// Represents an array of match results
#[repr(C)]
pub struct ElidMatchArray {
    /// Pointer to the array of matches
    pub matches: *mut ElidMatch,
    /// Number of matches in the array
    pub length: usize,
}

/// Helper function to convert C string to Rust string
unsafe fn c_str_to_rust(c_str: *const c_char) -> Option<&'static str> {
    if c_str.is_null() {
        return None;
    }
    CStr::from_ptr(c_str).to_str().ok()
}

/// Compute the Levenshtein distance between two strings.
///
/// Returns the minimum number of single-character edits needed to transform one string into another.
///
/// # Safety
///
/// Both `a` and `b` must be valid, null-terminated UTF-8 strings.
/// Returns 0 if either pointer is NULL.
#[no_mangle]
pub unsafe extern "C" fn elid_levenshtein(a: *const c_char, b: *const c_char) -> usize {
    let a_str = match c_str_to_rust(a) {
        Some(s) => s,
        None => return 0,
    };
    let b_str = match c_str_to_rust(b) {
        Some(s) => s,
        None => return 0,
    };
    crate::levenshtein(a_str, b_str)
}

/// Compute the normalized Levenshtein similarity between two strings.
///
/// Returns a value between 0.0 (completely different) and 1.0 (identical).
///
/// # Safety
///
/// Both `a` and `b` must be valid, null-terminated UTF-8 strings.
/// Returns 0.0 if either pointer is NULL.
#[no_mangle]
pub unsafe extern "C" fn elid_normalized_levenshtein(a: *const c_char, b: *const c_char) -> f64 {
    let a_str = match c_str_to_rust(a) {
        Some(s) => s,
        None => return 0.0,
    };
    let b_str = match c_str_to_rust(b) {
        Some(s) => s,
        None => return 0.0,
    };
    crate::normalized_levenshtein(a_str, b_str)
}

/// Compute the Jaro similarity between two strings.
///
/// Returns a value between 0.0 (completely different) and 1.0 (identical).
///
/// # Safety
///
/// Both `a` and `b` must be valid, null-terminated UTF-8 strings.
/// Returns 0.0 if either pointer is NULL.
#[no_mangle]
pub unsafe extern "C" fn elid_jaro(a: *const c_char, b: *const c_char) -> f64 {
    let a_str = match c_str_to_rust(a) {
        Some(s) => s,
        None => return 0.0,
    };
    let b_str = match c_str_to_rust(b) {
        Some(s) => s,
        None => return 0.0,
    };
    crate::jaro(a_str, b_str)
}

/// Compute the Jaro-Winkler similarity between two strings.
///
/// Returns a value between 0.0 (completely different) and 1.0 (identical).
///
/// # Safety
///
/// Both `a` and `b` must be valid, null-terminated UTF-8 strings.
/// Returns 0.0 if either pointer is NULL.
#[no_mangle]
pub unsafe extern "C" fn elid_jaro_winkler(a: *const c_char, b: *const c_char) -> f64 {
    let a_str = match c_str_to_rust(a) {
        Some(s) => s,
        None => return 0.0,
    };
    let b_str = match c_str_to_rust(b) {
        Some(s) => s,
        None => return 0.0,
    };
    crate::jaro_winkler(a_str, b_str)
}

/// Compute the Hamming distance between two strings.
///
/// Returns the number of positions at which the characters differ.
/// Returns -1 if strings have different lengths or if either pointer is NULL.
///
/// # Safety
///
/// Both `a` and `b` must be valid, null-terminated UTF-8 strings.
#[no_mangle]
pub unsafe extern "C" fn elid_hamming(a: *const c_char, b: *const c_char) -> i64 {
    let a_str = match c_str_to_rust(a) {
        Some(s) => s,
        None => return -1,
    };
    let b_str = match c_str_to_rust(b) {
        Some(s) => s,
        None => return -1,
    };
    match crate::hamming(a_str, b_str) {
        Some(dist) => dist as i64,
        None => -1,
    }
}

/// Compute the OSA (Optimal String Alignment) distance between two strings.
///
/// Similar to Levenshtein but also considers transpositions as a single operation.
///
/// # Safety
///
/// Both `a` and `b` must be valid, null-terminated UTF-8 strings.
/// Returns 0 if either pointer is NULL.
#[no_mangle]
pub unsafe extern "C" fn elid_osa_distance(a: *const c_char, b: *const c_char) -> usize {
    let a_str = match c_str_to_rust(a) {
        Some(s) => s,
        None => return 0,
    };
    let b_str = match c_str_to_rust(b) {
        Some(s) => s,
        None => return 0,
    };
    crate::osa_distance(a_str, b_str)
}

/// Compute the best matching similarity between two strings.
///
/// Runs multiple algorithms and returns the highest score.
///
/// # Safety
///
/// Both `a` and `b` must be valid, null-terminated UTF-8 strings.
/// Returns 0.0 if either pointer is NULL.
#[no_mangle]
pub unsafe extern "C" fn elid_best_match(a: *const c_char, b: *const c_char) -> f64 {
    let a_str = match c_str_to_rust(a) {
        Some(s) => s,
        None => return 0.0,
    };
    let b_str = match c_str_to_rust(b) {
        Some(s) => s,
        None => return 0.0,
    };
    crate::best_match(a_str, b_str)
}

/// Compute the SimHash fingerprint of a string.
///
/// Returns a 64-bit hash where similar strings produce similar numbers.
///
/// # Safety
///
/// `text` must be a valid, null-terminated UTF-8 string.
/// Returns 0 if pointer is NULL.
#[no_mangle]
pub unsafe extern "C" fn elid_simhash(text: *const c_char) -> u64 {
    let text_str = match c_str_to_rust(text) {
        Some(s) => s,
        None => return 0,
    };
    crate::simhash(text_str)
}

/// Compute the Hamming distance between two SimHash values.
///
/// Returns the number of differing bits. Lower values = higher similarity.
#[no_mangle]
pub extern "C" fn elid_simhash_distance(hash1: u64, hash2: u64) -> u32 {
    crate::simhash_distance(hash1, hash2)
}

/// Compute the normalized SimHash similarity between two strings.
///
/// Returns a value between 0.0 (completely different) and 1.0 (identical).
///
/// # Safety
///
/// Both `a` and `b` must be valid, null-terminated UTF-8 strings.
/// Returns 0.0 if either pointer is NULL.
#[no_mangle]
pub unsafe extern "C" fn elid_simhash_similarity(a: *const c_char, b: *const c_char) -> f64 {
    let a_str = match c_str_to_rust(a) {
        Some(s) => s,
        None => return 0.0,
    };
    let b_str = match c_str_to_rust(b) {
        Some(s) => s,
        None => return 0.0,
    };
    crate::simhash_similarity(a_str, b_str)
}

/// Free a string allocated by Rust.
///
/// This must be called on all strings returned by ELID functions to prevent memory leaks.
///
/// # Safety
///
/// `s` must be a string previously returned by an ELID function, or NULL.
/// Do not call this function twice on the same pointer.
#[no_mangle]
pub unsafe extern "C" fn elid_free_string(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}

/// Free a match array allocated by Rust.
///
/// This must be called on all match arrays returned by ELID functions to prevent memory leaks.
///
/// # Safety
///
/// `array` must be a match array previously returned by an ELID function.
/// Do not call this function twice on the same pointer.
#[no_mangle]
pub unsafe extern "C" fn elid_free_match_array(array: ElidMatchArray) {
    if !array.matches.is_null() && array.length > 0 {
        drop(Vec::from_raw_parts(
            array.matches,
            array.length,
            array.length,
        ));
    }
}

/// Get the library version as a static string.
///
/// The returned string does not need to be freed.
#[no_mangle]
pub extern "C" fn elid_version() -> *const c_char {
    // cbindgen's syn 1.x can't parse c"" literals, so use allow here
    #[allow(clippy::manual_c_str_literals)]
    {
        "0.0.0-dev\0".as_ptr() as *const c_char
    }
}

// ============================================================================
// Embedding FFI Bindings (feature = "embeddings")
// ============================================================================

#[cfg(feature = "embeddings")]
mod embeddings_ffi {
    use super::*;
    use crate::embeddings::{
        decode_to_embedding, encode, hamming_distance, is_reversible, Profile,
    };

    /// Encode an embedding vector using the lossless profile.
    ///
    /// Creates an ELID string that can be decoded back to the original embedding
    /// with full 32-bit precision.
    ///
    /// # Safety
    ///
    /// - `embedding` must be a valid pointer to an array of `len` f32 values.
    /// - `len` must be between 64 and 2048 (inclusive).
    /// - The returned string must be freed with `elid_free_string()`.
    ///
    /// Returns NULL on error (invalid dimensions, invalid values, encoding failure).
    #[no_mangle]
    pub unsafe extern "C" fn elid_encode_lossless(
        embedding: *const f32,
        len: usize,
    ) -> *mut c_char {
        if embedding.is_null() || !(64..=2048).contains(&len) {
            return std::ptr::null_mut();
        }

        let slice = std::slice::from_raw_parts(embedding, len);
        let profile = Profile::lossless();

        match encode(slice, &profile) {
            Ok(elid) => match CString::new(elid.as_str()) {
                Ok(cs) => cs.into_raw(),
                Err(_) => std::ptr::null_mut(),
            },
            Err(_) => std::ptr::null_mut(),
        }
    }

    /// Encode an embedding vector with compression based on retention percentage.
    ///
    /// The retention percentage (0.0-1.0) controls how much information is preserved:
    /// - 1.0 = lossless (Full32 precision, all dimensions)
    /// - 0.5 = half precision and/or half dimensions
    /// - 0.25 = quarter precision and/or quarter dimensions
    ///
    /// # Safety
    ///
    /// - `embedding` must be a valid pointer to an array of `len` f32 values.
    /// - `len` must be between 64 and 2048 (inclusive).
    /// - `retention_pct` must be between 0.0 and 1.0.
    /// - The returned string must be freed with `elid_free_string()`.
    ///
    /// Returns NULL on error.
    #[no_mangle]
    pub unsafe extern "C" fn elid_encode_compressed(
        embedding: *const f32,
        len: usize,
        retention_pct: f32,
    ) -> *mut c_char {
        if embedding.is_null() || !(64..=2048).contains(&len) {
            return std::ptr::null_mut();
        }

        let slice = std::slice::from_raw_parts(embedding, len);
        let profile = Profile::compressed(retention_pct, len as u16);

        match encode(slice, &profile) {
            Ok(elid) => match CString::new(elid.as_str()) {
                Ok(cs) => cs.into_raw(),
                Err(_) => std::ptr::null_mut(),
            },
            Err(_) => std::ptr::null_mut(),
        }
    }

    /// Encode an embedding vector with a maximum output string length constraint.
    ///
    /// Calculates the optimal precision and dimension settings to fit within
    /// the specified character limit while maximizing fidelity.
    ///
    /// # Safety
    ///
    /// - `embedding` must be a valid pointer to an array of `len` f32 values.
    /// - `len` must be between 64 and 2048 (inclusive).
    /// - `max_chars` should be at least 20 (header overhead).
    /// - The returned string must be freed with `elid_free_string()`.
    ///
    /// Returns NULL on error.
    #[no_mangle]
    pub unsafe extern "C" fn elid_encode_max_length(
        embedding: *const f32,
        len: usize,
        max_chars: usize,
    ) -> *mut c_char {
        if embedding.is_null() || !(64..=2048).contains(&len) {
            return std::ptr::null_mut();
        }

        let slice = std::slice::from_raw_parts(embedding, len);
        let profile = Profile::max_length(max_chars, len as u16);

        match encode(slice, &profile) {
            Ok(elid) => match CString::new(elid.as_str()) {
                Ok(cs) => cs.into_raw(),
                Err(_) => std::ptr::null_mut(),
            },
            Err(_) => std::ptr::null_mut(),
        }
    }

    /// Encode an embedding vector for cross-dimensional comparison.
    ///
    /// Projects the embedding to a common dimension space, allowing comparison
    /// between embeddings of different original dimensions (e.g., 256d vs 768d).
    ///
    /// # Safety
    ///
    /// - `embedding` must be a valid pointer to an array of `len` f32 values.
    /// - `len` must be between 64 and 2048 (inclusive).
    /// - `common_dims` is the target dimension space (vectors will be projected here).
    /// - The returned string must be freed with `elid_free_string()`.
    ///
    /// Returns NULL on error.
    #[no_mangle]
    pub unsafe extern "C" fn elid_encode_cross_dimensional(
        embedding: *const f32,
        len: usize,
        common_dims: u16,
    ) -> *mut c_char {
        if embedding.is_null() || !(64..=2048).contains(&len) {
            return std::ptr::null_mut();
        }

        let slice = std::slice::from_raw_parts(embedding, len);
        let profile = Profile::cross_dimensional(common_dims);

        match encode(slice, &profile) {
            Ok(elid) => match CString::new(elid.as_str()) {
                Ok(cs) => cs.into_raw(),
                Err(_) => std::ptr::null_mut(),
            },
            Err(_) => std::ptr::null_mut(),
        }
    }

    /// Decode an ELID string back to an embedding vector.
    ///
    /// Only works for ELIDs encoded with FullVector profiles (lossless, compressed,
    /// max_length, or cross_dimensional). Other profiles use lossy hashing that
    /// cannot be reversed.
    ///
    /// # Safety
    ///
    /// - `elid` must be a valid, null-terminated UTF-8 string.
    /// - `out_len` must be a valid pointer where the array length will be written.
    /// - The returned array must be freed with `elid_free_embedding()`.
    ///
    /// Returns NULL on error (invalid ELID, non-reversible profile, decoding failure).
    /// On success, `*out_len` will contain the number of f32 elements in the returned array.
    #[no_mangle]
    pub unsafe extern "C" fn elid_decode_to_embedding(
        elid: *const c_char,
        out_len: *mut usize,
    ) -> *mut f32 {
        if elid.is_null() || out_len.is_null() {
            return std::ptr::null_mut();
        }

        let elid_str = match c_str_to_rust(elid) {
            Some(s) => s,
            None => return std::ptr::null_mut(),
        };

        // Create Elid from string
        let elid_obj = match crate::embeddings::Elid::from_string(elid_str.to_string()) {
            Ok(e) => e,
            Err(_) => return std::ptr::null_mut(),
        };

        // Decode to embedding
        match decode_to_embedding(&elid_obj) {
            Ok((embedding, _metadata)) => {
                let len = embedding.len();
                let mut boxed = embedding.into_boxed_slice();
                let ptr = boxed.as_mut_ptr();
                std::mem::forget(boxed);
                *out_len = len;
                ptr
            }
            Err(_) => std::ptr::null_mut(),
        }
    }

    /// Check if an ELID can be decoded back to an embedding.
    ///
    /// Returns 1 if the ELID was encoded with a FullVector profile (reversible),
    /// 0 if not reversible, or -1 on error (NULL pointer, invalid ELID).
    ///
    /// # Safety
    ///
    /// `elid` must be a valid, null-terminated UTF-8 string, or NULL.
    #[no_mangle]
    pub unsafe extern "C" fn elid_is_reversible(elid: *const c_char) -> i32 {
        if elid.is_null() {
            return -1;
        }

        let elid_str = match c_str_to_rust(elid) {
            Some(s) => s,
            None => return -1,
        };

        // Create Elid from string
        let elid_obj = match crate::embeddings::Elid::from_string(elid_str.to_string()) {
            Ok(e) => e,
            Err(_) => return -1,
        };

        if is_reversible(&elid_obj) {
            1
        } else {
            0
        }
    }

    /// Compute the Hamming distance between two Mini128 ELIDs.
    ///
    /// Returns the number of differing bits in the SimHash payloads of two ELIDs.
    /// This distance is proportional to the angular distance between the original
    /// embeddings.
    ///
    /// Both ELIDs must use the Mini128 profile.
    ///
    /// # Safety
    ///
    /// Both `elid1` and `elid2` must be valid, null-terminated UTF-8 strings.
    ///
    /// Returns -1 on error (NULL pointers, invalid ELIDs, profile mismatch).
    /// On success, returns the Hamming distance (0-128).
    #[no_mangle]
    pub unsafe extern "C" fn elid_embedding_hamming_distance(
        elid1: *const c_char,
        elid2: *const c_char,
    ) -> i32 {
        if elid1.is_null() || elid2.is_null() {
            return -1;
        }

        let elid1_str = match c_str_to_rust(elid1) {
            Some(s) => s,
            None => return -1,
        };

        let elid2_str = match c_str_to_rust(elid2) {
            Some(s) => s,
            None => return -1,
        };

        // Create Elid objects
        let elid1_obj = match crate::embeddings::Elid::from_string(elid1_str.to_string()) {
            Ok(e) => e,
            Err(_) => return -1,
        };

        let elid2_obj = match crate::embeddings::Elid::from_string(elid2_str.to_string()) {
            Ok(e) => e,
            Err(_) => return -1,
        };

        // Compute Hamming distance
        match hamming_distance(&elid1_obj, &elid2_obj) {
            Ok(dist) => dist as i32,
            Err(_) => -1,
        }
    }

    /// Free an embedding array allocated by `elid_decode_to_embedding()`.
    ///
    /// # Safety
    ///
    /// - `ptr` must be a pointer previously returned by `elid_decode_to_embedding()`, or NULL.
    /// - `len` must be the length that was written to `out_len` by `elid_decode_to_embedding()`.
    /// - Do not call this function twice on the same pointer.
    /// - Do not use the pointer after freeing.
    #[no_mangle]
    pub unsafe extern "C" fn elid_free_embedding(ptr: *mut f32, len: usize) {
        if !ptr.is_null() && len > 0 {
            drop(Vec::from_raw_parts(ptr, len, len));
        }
    }
}

// Re-export embedding FFI functions at module level
#[cfg(feature = "embeddings")]
pub use embeddings_ffi::*;

// ============================================================================
// Model FFI Bindings (feature = "models-text" and "models-image")
// ============================================================================

/// Embed text using Model2Vec potion-base-8M model
///
/// Returns a 256-dimensional embedding as a float array.
/// Caller must free with `elid_free_embedding(ptr, len)`.
/// Returns NULL on error.
///
/// # Safety
///
/// - `text` must be a valid, null-terminated UTF-8 string.
/// - `out_len` must be a valid pointer where the array length will be written.
/// - The returned array must be freed with `elid_free_embedding()`.
#[cfg(feature = "models-text")]
#[no_mangle]
pub unsafe extern "C" fn elid_embed_text(
    text: *const c_char,
    out_len: *mut usize,
) -> *mut f32 {
    if text.is_null() || out_len.is_null() {
        return std::ptr::null_mut();
    }

    let text_str = match c_str_to_rust(text) {
        Some(s) => s,
        None => return std::ptr::null_mut(),
    };

    match crate::models::embed_text(text_str) {
        Ok(embedding) => {
            let len = embedding.len();
            let mut boxed = embedding.into_boxed_slice();
            let ptr = boxed.as_mut_ptr();
            std::mem::forget(boxed);
            *out_len = len;
            ptr
        }
        Err(_) => std::ptr::null_mut(),
    }
}

/// Embed image using MobileNetV3-Small model
///
/// Takes raw image bytes (JPEG or PNG).
/// Returns a 1024-dimensional embedding as a float array.
/// Caller must free with `elid_free_embedding(ptr, len)`.
/// Returns NULL on error.
///
/// # Safety
///
/// - `image_bytes` must be a valid pointer to an array of `image_len` bytes.
/// - `image_len` must be the actual length of the image data.
/// - `out_len` must be a valid pointer where the array length will be written.
/// - The returned array must be freed with `elid_free_embedding()`.
#[cfg(feature = "models-image")]
#[no_mangle]
pub unsafe extern "C" fn elid_embed_image(
    image_bytes: *const u8,
    image_len: usize,
    out_len: *mut usize,
) -> *mut f32 {
    if image_bytes.is_null() || out_len.is_null() || image_len == 0 {
        return std::ptr::null_mut();
    }

    let bytes = std::slice::from_raw_parts(image_bytes, image_len);

    match crate::models::embed_image(bytes) {
        Ok(embedding) => {
            let len = embedding.len();
            let mut boxed = embedding.into_boxed_slice();
            let ptr = boxed.as_mut_ptr();
            std::mem::forget(boxed);
            *out_len = len;
            ptr
        }
        Err(_) => std::ptr::null_mut(),
    }
}

// ============================================================================
// LSH Bands FFI Binding (feature = "embeddings")
// ============================================================================

/// Generate LSH bands from an embedding
///
/// Computes a 128-bit SimHash of the embedding and splits it into bands for
/// locality-sensitive hashing. This enables efficient approximate nearest
/// neighbor search using database indexes.
///
/// Returns a newline-separated string of bands. The format is:
/// `band0\nband1\nband2\n...\nbandN`
///
/// Each band is a base32hex-encoded string (lowercase alphanumeric characters only).
///
/// Caller must free with `elid_free_string()`.
/// Returns NULL on error (invalid parameters, null pointers).
///
/// # Parameters
///
/// - `embedding`: Pointer to embedding vector (f32 array)
/// - `len`: Number of elements in the embedding
/// - `num_bands`: Number of bands (must be 1, 2, 4, 8, or 16)
/// - `seed`: Master seed for deterministic hashing
///
/// # Safety
///
/// - `embedding` must be a valid pointer to an array of `len` f32 values.
/// - The returned string must be freed with `elid_free_string()`.
#[cfg(feature = "embeddings")]
#[no_mangle]
pub unsafe extern "C" fn elid_embedding_to_bands(
    embedding: *const f32,
    len: usize,
    num_bands: u8,
    seed: u64,
) -> *mut c_char {
    if embedding.is_null() || len == 0 {
        return std::ptr::null_mut();
    }

    // Validate num_bands: must be 1, 2, 4, 8, or 16
    if num_bands == 0 || 16 % num_bands != 0 {
        return std::ptr::null_mut();
    }

    let slice = std::slice::from_raw_parts(embedding, len);
    let bands = crate::embeddings::embedding_to_bands(slice, num_bands, seed);

    if bands.is_empty() {
        return std::ptr::null_mut();
    }

    // Join bands with newline separator
    let result = bands.join("\n");

    match CString::new(result) {
        Ok(cs) => cs.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use std::ptr;

    #[test]
    fn test_ffi_levenshtein() {
        let a = CString::new("kitten").unwrap();
        let b = CString::new("sitting").unwrap();
        unsafe {
            let dist = elid_levenshtein(a.as_ptr(), b.as_ptr());
            assert_eq!(dist, 3);
        }
    }

    #[test]
    fn test_ffi_normalized_levenshtein() {
        let a = CString::new("hello").unwrap();
        let b = CString::new("hello").unwrap();
        unsafe {
            let sim = elid_normalized_levenshtein(a.as_ptr(), b.as_ptr());
            assert_eq!(sim, 1.0);
        }
    }

    #[test]
    fn test_ffi_jaro_winkler() {
        let a = CString::new("martha").unwrap();
        let b = CString::new("marhta").unwrap();
        unsafe {
            let sim = elid_jaro_winkler(a.as_ptr(), b.as_ptr());
            assert!(sim > 0.9);
        }
    }

    #[test]
    fn test_ffi_hamming() {
        let a = CString::new("karolin").unwrap();
        let b = CString::new("kathrin").unwrap();
        unsafe {
            let dist = elid_hamming(a.as_ptr(), b.as_ptr());
            assert_eq!(dist, 3);
        }
    }

    #[test]
    fn test_ffi_simhash() {
        let text = CString::new("iPhone 14").unwrap();
        unsafe {
            let hash = elid_simhash(text.as_ptr());
            assert!(hash > 0);
        }
    }

    #[test]
    fn test_ffi_simhash_distance() {
        let hash1 = 0b1010101010101010u64;
        let hash2 = 0b1010101010101011u64;
        let dist = elid_simhash_distance(hash1, hash2);
        assert_eq!(dist, 1);
    }

    #[test]
    fn test_ffi_null_safety() {
        unsafe {
            // Should not crash with NULL pointers
            assert_eq!(elid_levenshtein(ptr::null(), ptr::null()), 0);
            assert_eq!(elid_normalized_levenshtein(ptr::null(), ptr::null()), 0.0);
            assert_eq!(elid_jaro(ptr::null(), ptr::null()), 0.0);
            assert_eq!(elid_hamming(ptr::null(), ptr::null()), -1);
        }
    }
}

#[cfg(all(test, feature = "embeddings"))]
mod embeddings_tests {
    use super::*;
    use std::ffi::CString;
    use std::ptr;

    #[test]
    fn test_ffi_encode_lossless() {
        let embedding: Vec<f32> = (0..128).map(|i| (i as f32 / 64.0) - 1.0).collect();
        unsafe {
            let elid = elid_encode_lossless(embedding.as_ptr(), embedding.len());
            assert!(!elid.is_null(), "Should encode successfully");

            // Clean up
            elid_free_string(elid);
        }
    }

    #[test]
    fn test_ffi_encode_lossless_null() {
        unsafe {
            let elid = elid_encode_lossless(ptr::null(), 128);
            assert!(elid.is_null(), "Should return NULL for null input");
        }
    }

    #[test]
    fn test_ffi_encode_lossless_invalid_len() {
        let embedding: Vec<f32> = vec![0.1; 32]; // Too small
        unsafe {
            let elid = elid_encode_lossless(embedding.as_ptr(), embedding.len());
            assert!(elid.is_null(), "Should return NULL for invalid length");
        }
    }

    #[test]
    fn test_ffi_encode_compressed() {
        let embedding: Vec<f32> = (0..768).map(|i| (i as f32 / 384.0) - 1.0).collect();
        unsafe {
            let elid = elid_encode_compressed(embedding.as_ptr(), embedding.len(), 0.5);
            assert!(!elid.is_null(), "Should encode successfully");
            elid_free_string(elid);
        }
    }

    #[test]
    fn test_ffi_encode_max_length() {
        let embedding: Vec<f32> = (0..768).map(|i| (i as f32 / 384.0) - 1.0).collect();
        let max_chars = 100;
        unsafe {
            let elid = elid_encode_max_length(embedding.as_ptr(), embedding.len(), max_chars);
            assert!(!elid.is_null(), "Should encode successfully");

            // Verify length constraint
            let elid_str = CStr::from_ptr(elid).to_str().unwrap();
            assert!(
                elid_str.len() <= max_chars,
                "ELID length {} exceeds max {}",
                elid_str.len(),
                max_chars
            );

            elid_free_string(elid);
        }
    }

    #[test]
    fn test_ffi_encode_cross_dimensional() {
        let embedding: Vec<f32> = (0..768).map(|i| (i as f32 / 384.0) - 1.0).collect();
        unsafe {
            let elid = elid_encode_cross_dimensional(embedding.as_ptr(), embedding.len(), 128);
            assert!(!elid.is_null(), "Should encode successfully");
            elid_free_string(elid);
        }
    }

    #[test]
    fn test_ffi_encode_decode_roundtrip() {
        let embedding: Vec<f32> = (0..128).map(|i| (i as f32 / 64.0) - 1.0).collect();
        unsafe {
            // Encode
            let elid = elid_encode_lossless(embedding.as_ptr(), embedding.len());
            assert!(!elid.is_null());

            // Check reversibility
            let is_rev = elid_is_reversible(elid);
            assert_eq!(is_rev, 1, "Lossless encoding should be reversible");

            // Decode
            let mut out_len: usize = 0;
            let decoded = elid_decode_to_embedding(elid, &mut out_len);
            assert!(!decoded.is_null(), "Should decode successfully");
            assert_eq!(out_len, embedding.len());

            // Verify values match
            let decoded_slice = std::slice::from_raw_parts(decoded, out_len);
            for (i, (&orig, &dec)) in embedding.iter().zip(decoded_slice.iter()).enumerate() {
                assert_eq!(orig, dec, "Mismatch at index {}: {} vs {}", i, orig, dec);
            }

            // Clean up
            elid_free_embedding(decoded, out_len);
            elid_free_string(elid);
        }
    }

    #[test]
    fn test_ffi_decode_null_safety() {
        unsafe {
            let mut out_len: usize = 0;

            // NULL elid
            let result = elid_decode_to_embedding(ptr::null(), &mut out_len);
            assert!(result.is_null());

            // NULL out_len
            let elid = CString::new("invalid").unwrap();
            let result = elid_decode_to_embedding(elid.as_ptr(), ptr::null_mut());
            assert!(result.is_null());
        }
    }

    #[test]
    fn test_ffi_is_reversible_null() {
        unsafe {
            let result = elid_is_reversible(ptr::null());
            assert_eq!(result, -1, "Should return -1 for NULL");
        }
    }

    #[test]
    fn test_ffi_is_reversible_invalid() {
        let invalid = CString::new("xyz_invalid").unwrap();
        unsafe {
            let result = elid_is_reversible(invalid.as_ptr());
            assert_eq!(result, -1, "Should return -1 for invalid ELID");
        }
    }

    #[test]
    fn test_ffi_embedding_hamming_distance_null() {
        unsafe {
            assert_eq!(
                elid_embedding_hamming_distance(ptr::null(), ptr::null()),
                -1
            );

            let valid = CString::new("test").unwrap();
            assert_eq!(
                elid_embedding_hamming_distance(valid.as_ptr(), ptr::null()),
                -1
            );
            assert_eq!(
                elid_embedding_hamming_distance(ptr::null(), valid.as_ptr()),
                -1
            );
        }
    }

    #[test]
    fn test_ffi_free_embedding_null() {
        unsafe {
            // Should not crash
            elid_free_embedding(ptr::null_mut(), 0);
            elid_free_embedding(ptr::null_mut(), 100);
        }
    }

    #[test]
    fn test_ffi_embedding_to_bands() {
        let embedding: Vec<f32> = (0..256).map(|i| (i as f32 / 128.0) - 1.0).collect();
        let seed = 0x454c4944_53494d48u64;
        unsafe {
            let bands = elid_embedding_to_bands(embedding.as_ptr(), embedding.len(), 4, seed);
            assert!(!bands.is_null(), "Should generate bands successfully");

            // Parse the newline-separated string
            let bands_str = CStr::from_ptr(bands).to_str().unwrap();
            let band_vec: Vec<&str> = bands_str.split('\n').collect();
            assert_eq!(band_vec.len(), 4, "Should have 4 bands");

            // Verify bands are base32hex encoded (lowercase alphanumeric)
            for band in &band_vec {
                assert!(
                    band.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()),
                    "Band should be base32hex encoded: {}",
                    band
                );
            }

            elid_free_string(bands);
        }
    }

    #[test]
    fn test_ffi_embedding_to_bands_null_safety() {
        unsafe {
            // NULL embedding
            let result = elid_embedding_to_bands(ptr::null(), 128, 4, 0);
            assert!(result.is_null(), "Should return NULL for null embedding");

            // Zero length
            let embedding: Vec<f32> = vec![0.1; 128];
            let result = elid_embedding_to_bands(embedding.as_ptr(), 0, 4, 0);
            assert!(result.is_null(), "Should return NULL for zero length");
        }
    }

    #[test]
    fn test_ffi_embedding_to_bands_invalid_num_bands() {
        let embedding: Vec<f32> = (0..256).map(|i| (i as f32 / 128.0) - 1.0).collect();
        unsafe {
            // Invalid num_bands values: 0, 3, 5, 6, 7, 9, etc.
            let result = elid_embedding_to_bands(embedding.as_ptr(), embedding.len(), 0, 0);
            assert!(result.is_null(), "Should return NULL for num_bands=0");

            let result = elid_embedding_to_bands(embedding.as_ptr(), embedding.len(), 3, 0);
            assert!(result.is_null(), "Should return NULL for num_bands=3");

            let result = elid_embedding_to_bands(embedding.as_ptr(), embedding.len(), 5, 0);
            assert!(result.is_null(), "Should return NULL for num_bands=5");

            // Valid num_bands values: 1, 2, 4, 8, 16
            let result = elid_embedding_to_bands(embedding.as_ptr(), embedding.len(), 1, 0);
            assert!(!result.is_null(), "Should succeed for num_bands=1");
            elid_free_string(result);

            let result = elid_embedding_to_bands(embedding.as_ptr(), embedding.len(), 2, 0);
            assert!(!result.is_null(), "Should succeed for num_bands=2");
            elid_free_string(result);

            let result = elid_embedding_to_bands(embedding.as_ptr(), embedding.len(), 8, 0);
            assert!(!result.is_null(), "Should succeed for num_bands=8");
            elid_free_string(result);

            let result = elid_embedding_to_bands(embedding.as_ptr(), embedding.len(), 16, 0);
            assert!(!result.is_null(), "Should succeed for num_bands=16");
            elid_free_string(result);
        }
    }

    #[test]
    fn test_ffi_embedding_to_bands_deterministic() {
        let embedding: Vec<f32> = (0..256).map(|i| (i as f32 / 128.0) - 1.0).collect();
        let seed = 0x12345678u64;
        unsafe {
            let bands1 = elid_embedding_to_bands(embedding.as_ptr(), embedding.len(), 4, seed);
            let bands2 = elid_embedding_to_bands(embedding.as_ptr(), embedding.len(), 4, seed);

            assert!(!bands1.is_null());
            assert!(!bands2.is_null());

            let str1 = CStr::from_ptr(bands1).to_str().unwrap();
            let str2 = CStr::from_ptr(bands2).to_str().unwrap();
            assert_eq!(str1, str2, "Same embedding and seed should produce same bands");

            elid_free_string(bands1);
            elid_free_string(bands2);
        }
    }
}

#[cfg(all(test, feature = "models-text"))]
mod models_text_tests {
    use super::*;
    use std::ptr;

    #[test]
    fn test_ffi_embed_text_null_safety() {
        unsafe {
            let mut out_len: usize = 0;

            // NULL text
            let result = elid_embed_text(ptr::null(), &mut out_len);
            assert!(result.is_null(), "Should return NULL for null text");

            // NULL out_len
            let text = CString::new("Hello, world!").unwrap();
            let result = elid_embed_text(text.as_ptr(), ptr::null_mut());
            assert!(result.is_null(), "Should return NULL for null out_len");
        }
    }

    #[test]
    fn test_ffi_embed_text_not_implemented() {
        // Currently the model returns an error since it's not implemented yet
        let text = CString::new("Hello, world!").unwrap();
        unsafe {
            let mut out_len: usize = 0;
            let result = elid_embed_text(text.as_ptr(), &mut out_len);
            // Model is not implemented yet, should return NULL
            assert!(result.is_null(), "Should return NULL when model not loaded");
        }
    }
}

#[cfg(all(test, feature = "models-image"))]
mod models_image_tests {
    use super::*;
    use std::ptr;

    #[test]
    fn test_ffi_embed_image_null_safety() {
        unsafe {
            let mut out_len: usize = 0;

            // NULL image_bytes
            let result = elid_embed_image(ptr::null(), 100, &mut out_len);
            assert!(result.is_null(), "Should return NULL for null image_bytes");

            // Zero length
            let bytes = vec![0u8; 100];
            let result = elid_embed_image(bytes.as_ptr(), 0, &mut out_len);
            assert!(result.is_null(), "Should return NULL for zero length");

            // NULL out_len
            let result = elid_embed_image(bytes.as_ptr(), bytes.len(), ptr::null_mut());
            assert!(result.is_null(), "Should return NULL for null out_len");
        }
    }

    #[test]
    fn test_ffi_embed_image_not_implemented() {
        // Currently the model returns an error since it's not implemented yet
        let dummy_bytes = vec![0u8; 100];
        unsafe {
            let mut out_len: usize = 0;
            let result = elid_embed_image(dummy_bytes.as_ptr(), dummy_bytes.len(), &mut out_len);
            // Model is not implemented yet, should return NULL
            assert!(result.is_null(), "Should return NULL when model not loaded");
        }
    }
}
