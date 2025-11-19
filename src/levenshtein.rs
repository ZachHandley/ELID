//! Levenshtein distance implementation
//!
//! The Levenshtein distance is a string metric for measuring the difference between two sequences.
//! It's defined as the minimum number of single-character edits (insertions, deletions, or substitutions)
//! required to change one string into the other.

use crate::{common, SimilarityOpts};

/// Compute the Levenshtein distance between two strings.
///
/// This is the classic edit distance algorithm. Returns the minimum number of
/// single-character edits needed to transform one string into the other.
///
/// # Example
///
/// ```rust
/// use elid::levenshtein;
///
/// let distance = levenshtein("kitten", "sitting");
/// assert_eq!(distance, 3); // k→s, e→i, insert g
/// ```
///
/// # Time Complexity
///
/// O(m * n) where m and n are the lengths of the input strings.
///
/// # Space Complexity
///
/// O(min(m, n)) - uses optimized single-row algorithm
pub fn levenshtein(a: &str, b: &str) -> usize {
    if a == b {
        return 0;
    }
    if a.is_empty() {
        return b.chars().count();
    }
    if b.is_empty() {
        return a.chars().count();
    }

    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    // Optimize by using the shorter string for the column
    if a_len > b_len {
        return levenshtein(b, a);
    }

    // Use a single row algorithm to save space
    let mut prev_row: Vec<usize> = (0..=a_len).collect();

    for (i, b_char) in b_chars.iter().enumerate() {
        let mut curr_row = vec![i + 1];

        for (j, a_char) in a_chars.iter().enumerate() {
            let cost = if a_char == b_char { 0 } else { 1 };
            let insertion = curr_row[j] + 1;
            let deletion = prev_row[j + 1] + 1;
            let substitution = prev_row[j] + cost;

            curr_row.push(insertion.min(deletion).min(substitution));
        }

        prev_row = curr_row;
    }

    prev_row[a_len]
}

/// Compute the normalized Levenshtein similarity between two strings.
///
/// Returns a value between 0.0 (completely different) and 1.0 (identical).
/// The normalized score is calculated as: 1.0 - (distance / max_length)
///
/// # Example
///
/// ```rust
/// use elid::normalized_levenshtein;
///
/// let similarity = normalized_levenshtein("hello", "hello");
/// assert_eq!(similarity, 1.0);
///
/// let similarity = normalized_levenshtein("kitten", "sitting");
/// assert!(similarity > 0.5 && similarity < 0.7);
/// ```
pub fn normalized_levenshtein(a: &str, b: &str) -> f64 {
    if a == b {
        return 1.0;
    }

    let distance = levenshtein(a, b);
    let max_len = a.chars().count().max(b.chars().count());

    if max_len == 0 {
        return 1.0;
    }

    1.0 - (distance as f64 / max_len as f64)
}

/// Compute Levenshtein distance with configurable options.
///
/// # Example
///
/// ```rust
/// use elid::{levenshtein_with_opts, SimilarityOpts};
///
/// let opts = SimilarityOpts {
///     case_sensitive: false,
///     trim_whitespace: true,
///     ..Default::default()
/// };
///
/// let distance = levenshtein_with_opts("Hello ", " hello", &opts);
/// assert_eq!(distance, 0);
/// ```
pub fn levenshtein_with_opts(a: &str, b: &str, opts: &SimilarityOpts) -> usize {
    let (a_processed, b_processed) = common::preprocess_strings(a, b, opts);
    levenshtein(&a_processed, &b_processed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_identical() {
        assert_eq!(levenshtein("hello", "hello"), 0);
    }

    #[test]
    fn test_levenshtein_empty() {
        assert_eq!(levenshtein("", "hello"), 5);
        assert_eq!(levenshtein("hello", ""), 5);
        assert_eq!(levenshtein("", ""), 0);
    }

    #[test]
    fn test_levenshtein_classic() {
        assert_eq!(levenshtein("kitten", "sitting"), 3);
        assert_eq!(levenshtein("saturday", "sunday"), 3);
        assert_eq!(levenshtein("book", "back"), 2);
    }

    #[test]
    fn test_normalized_levenshtein() {
        assert_eq!(normalized_levenshtein("hello", "hello"), 1.0);
        assert!(normalized_levenshtein("kitten", "sitting") > 0.5);
        assert!(normalized_levenshtein("abc", "xyz") < 0.5);
    }

    #[test]
    fn test_levenshtein_with_opts() {
        let opts = SimilarityOpts {
            case_sensitive: false,
            trim_whitespace: true,
            ..Default::default()
        };

        assert_eq!(levenshtein_with_opts("Hello ", " hello", &opts), 0);
        assert_eq!(levenshtein_with_opts("ABC", "abc", &opts), 0);
    }

    #[test]
    fn test_levenshtein_unicode() {
        assert_eq!(levenshtein("café", "cafe"), 1);
        assert_eq!(levenshtein("你好", "您好"), 1);
    }
}
