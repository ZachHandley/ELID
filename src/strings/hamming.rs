//! Hamming distance implementation
//!
//! The Hamming distance is the number of positions at which the corresponding
//! characters are different. It's only defined for strings of equal length.

/// Compute the Hamming distance between two strings.
///
/// Returns the number of positions at which the characters differ.
/// The strings must be of equal length, otherwise returns None.
///
/// # Example
///
/// ```rust
/// use elid::hamming;
///
/// let distance = hamming("karolin", "kathrin").unwrap();
/// assert_eq!(distance, 3);
///
/// // Different lengths return None
/// assert_eq!(hamming("hello", "world!"), None);
/// ```
///
/// # Time Complexity
///
/// O(n) where n is the length of the strings.
pub fn hamming(a: &str, b: &str) -> Option<usize> {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    if a_chars.len() != b_chars.len() {
        return None;
    }

    let distance = a_chars
        .iter()
        .zip(b_chars.iter())
        .filter(|(a_char, b_char)| a_char != b_char)
        .count();

    Some(distance)
}

/// Compute the normalized Hamming similarity between two strings.
///
/// Returns a value between 0.0 (completely different) and 1.0 (identical).
/// Returns None if the strings have different lengths.
///
/// # Example
///
/// ```rust
/// use elid::hamming;
///
/// let distance = hamming("1011101", "1001001").unwrap();
/// assert_eq!(distance, 2);
/// ```
pub fn normalized_hamming(a: &str, b: &str) -> Option<f64> {
    let distance = hamming(a, b)?;
    let len = a.chars().count();

    if len == 0 {
        return Some(1.0);
    }

    Some(1.0 - (distance as f64 / len as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hamming_identical() {
        assert_eq!(hamming("hello", "hello"), Some(0));
    }

    #[test]
    fn test_hamming_different_length() {
        assert_eq!(hamming("hello", "world!"), None);
        assert_eq!(hamming("hi", "hello"), None);
    }

    #[test]
    fn test_hamming_classic() {
        assert_eq!(hamming("karolin", "kathrin"), Some(3));
        assert_eq!(hamming("1011101", "1001001"), Some(2));
        assert_eq!(hamming("2173896", "2233796"), Some(3));
    }

    #[test]
    fn test_hamming_empty() {
        assert_eq!(hamming("", ""), Some(0));
    }

    #[test]
    fn test_normalized_hamming() {
        assert_eq!(normalized_hamming("hello", "hello"), Some(1.0));
        // Use approximate comparison for floats
        let sim = normalized_hamming("hello", "world").unwrap();
        assert!((sim - 0.2).abs() < 1e-10); // 1 match out of 5
        assert_eq!(normalized_hamming("hi", "hello"), None);
    }

    #[test]
    fn test_hamming_unicode() {
        // "café" has 4 chars (c, a, f, é), "cafe" has 4 chars (c, a, f, e)
        assert_eq!(hamming("café", "cafe"), Some(1)); // é != e
        assert_eq!(hamming("你好", "您好"), Some(1));
    }
}
