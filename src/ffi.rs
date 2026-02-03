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
    crate::levenshtein::levenshtein(a_str, b_str)
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
    crate::levenshtein::normalized_levenshtein(a_str, b_str)
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
    crate::jaro_winkler::jaro(a_str, b_str)
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
    crate::jaro_winkler::jaro_winkler(a_str, b_str)
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
    match crate::hamming::hamming(a_str, b_str) {
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
    crate::osa::osa_distance(a_str, b_str)
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
    crate::simhash::simhash(text_str)
}

/// Compute the Hamming distance between two SimHash values.
///
/// Returns the number of differing bits. Lower values = higher similarity.
#[no_mangle]
pub extern "C" fn elid_simhash_distance(hash1: u64, hash2: u64) -> u32 {
    crate::simhash::simhash_distance(hash1, hash2)
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
    crate::simhash::simhash_similarity(a_str, b_str)
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
        "0.1.0\0".as_ptr() as *const c_char
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
