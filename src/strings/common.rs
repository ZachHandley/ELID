//! Common utilities for string processing

/// Options for configuring string similarity algorithms
#[derive(Debug, Clone, Copy)]
pub struct SimilarityOpts {
    /// Case-sensitive comparison (default: true)
    pub case_sensitive: bool,
    /// Trim whitespace before comparison (default: false)
    pub trim_whitespace: bool,
    /// Prefix scale for Jaro-Winkler (default: 0.1, max: 0.25)
    pub prefix_scale: f64,
}

impl Default for SimilarityOpts {
    fn default() -> Self {
        Self {
            case_sensitive: true,
            trim_whitespace: false,
            prefix_scale: 0.1,
        }
    }
}

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
