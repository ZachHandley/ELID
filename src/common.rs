//! Common utilities for string processing

use crate::SimilarityOpts;

/// Preprocess strings according to the given options.
///
/// This handles case normalization and whitespace trimming.
pub fn preprocess_strings(a: &str, b: &str, opts: &SimilarityOpts) -> (String, String) {
    let mut a_processed = a.to_string();
    let mut b_processed = b.to_string();

    if opts.trim_whitespace {
        a_processed = a_processed.trim().to_string();
        b_processed = b_processed.trim().to_string();
    }

    if !opts.case_sensitive {
        a_processed = a_processed.to_lowercase();
        b_processed = b_processed.to_lowercase();
    }

    (a_processed, b_processed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preprocess_case_insensitive() {
        let opts = SimilarityOpts {
            case_sensitive: false,
            ..Default::default()
        };

        let (a, b) = preprocess_strings("HELLO", "hello", &opts);
        assert_eq!(a, "hello");
        assert_eq!(b, "hello");
    }

    #[test]
    fn test_preprocess_trim() {
        let opts = SimilarityOpts {
            trim_whitespace: true,
            ..Default::default()
        };

        let (a, b) = preprocess_strings("  hello  ", "  world  ", &opts);
        assert_eq!(a, "hello");
        assert_eq!(b, "world");
    }

    #[test]
    fn test_preprocess_both() {
        let opts = SimilarityOpts {
            case_sensitive: false,
            trim_whitespace: true,
            ..Default::default()
        };

        let (a, b) = preprocess_strings("  HELLO  ", "  world  ", &opts);
        assert_eq!(a, "hello");
        assert_eq!(b, "world");
    }
}
