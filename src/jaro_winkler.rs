//! Jaro and Jaro-Winkler similarity implementations
//!
//! The Jaro similarity is a measure of similarity between two strings.
//! The Jaro-Winkler similarity is a variant that gives more favorable ratings
//! to strings with common prefixes.

/// Compute the Jaro similarity between two strings.
///
/// Returns a value between 0.0 (completely different) and 1.0 (identical).
/// The Jaro similarity is particularly effective for short strings like names.
///
/// # Example
///
/// ```rust
/// use elid::jaro;
///
/// let similarity = jaro("martha", "marhta");
/// assert!(similarity > 0.9);
/// ```
///
/// # Algorithm
///
/// The Jaro similarity is calculated as:
/// - If both strings are empty: 1.0
/// - Otherwise: (matches/len1 + matches/len2 + (matches-transpositions)/matches) / 3
pub fn jaro(a: &str, b: &str) -> f64 {
    if a == b {
        return 1.0;
    }
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }

    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    // Calculate the match window
    let match_window = (a_len.max(b_len) / 2).saturating_sub(1);

    let mut a_matches = vec![false; a_len];
    let mut b_matches = vec![false; b_len];

    let mut matches = 0;
    let mut transpositions = 0;

    // Find matches
    for (i, a_char) in a_chars.iter().enumerate() {
        let start = i.saturating_sub(match_window);
        let end = (i + match_window + 1).min(b_len);

        for j in start..end {
            if b_matches[j] || a_char != &b_chars[j] {
                continue;
            }
            a_matches[i] = true;
            b_matches[j] = true;
            matches += 1;
            break;
        }
    }

    if matches == 0 {
        return 0.0;
    }

    // Count transpositions
    let mut k = 0;
    for (i, is_match) in a_matches.iter().enumerate() {
        if !is_match {
            continue;
        }
        while !b_matches[k] {
            k += 1;
        }
        if a_chars[i] != b_chars[k] {
            transpositions += 1;
        }
        k += 1;
    }

    let matches_f = matches as f64;
    let transpositions_f = (transpositions / 2) as f64;

    (matches_f / a_len as f64
        + matches_f / b_len as f64
        + (matches_f - transpositions_f) / matches_f)
        / 3.0
}

/// Compute the Jaro-Winkler similarity between two strings.
///
/// Returns a value between 0.0 (completely different) and 1.0 (identical).
/// This variant gives more favorable ratings to strings with common prefixes.
///
/// # Example
///
/// ```rust
/// use elid::{jaro_winkler, jaro};
///
/// let similarity = jaro_winkler("martha", "marhta");
/// assert!(similarity > 0.9);
///
/// // Strings with common prefixes get higher scores
/// let sim1 = jaro_winkler("DIXON", "DICKSON");
/// let sim2 = jaro("DIXON", "DICKSON");
/// assert!(sim1 > sim2);
/// ```
pub fn jaro_winkler(a: &str, b: &str) -> f64 {
    jaro_winkler_with_prefix(a, b, 0.1)
}

/// Compute the Jaro-Winkler similarity with a custom prefix scale.
///
/// The prefix scale should be between 0.0 and 0.25 (the standard is 0.1).
/// Higher values give more weight to common prefixes.
///
/// # Example
///
/// ```rust
/// use elid::jaro_winkler_with_prefix;
///
/// // With higher prefix scale, common prefixes matter more
/// let similarity = jaro_winkler_with_prefix("martha", "martha", 0.2);
/// assert_eq!(similarity, 1.0);
/// ```
pub fn jaro_winkler_with_prefix(a: &str, b: &str, prefix_scale: f64) -> f64 {
    let jaro_sim = jaro(a, b);

    if jaro_sim < 0.7 {
        // Only apply prefix bonus if Jaro similarity is high enough
        return jaro_sim;
    }

    // Find common prefix (up to 4 characters)
    let prefix_len = a
        .chars()
        .zip(b.chars())
        .take(4)
        .take_while(|(a_char, b_char)| a_char == b_char)
        .count();

    let prefix_scale = prefix_scale.min(0.25); // Max prefix scale is 0.25

    jaro_sim + (prefix_len as f64 * prefix_scale * (1.0 - jaro_sim))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jaro_identical() {
        assert_eq!(jaro("hello", "hello"), 1.0);
    }

    #[test]
    fn test_jaro_empty() {
        assert_eq!(jaro("", ""), 1.0);
        assert_eq!(jaro("hello", ""), 0.0);
        assert_eq!(jaro("", "hello"), 0.0);
    }

    #[test]
    fn test_jaro_classic() {
        let sim = jaro("martha", "marhta");
        assert!(sim > 0.9);

        let sim = jaro("DIXON", "DICKSON");
        assert!(sim > 0.7);
    }

    #[test]
    fn test_jaro_winkler_prefix_bonus() {
        // Strings with common prefixes should score higher with Jaro-Winkler
        let jaro_sim = jaro("DIXON", "DICKSON");
        let jw_sim = jaro_winkler("DIXON", "DICKSON");
        assert!(jw_sim > jaro_sim);
    }

    #[test]
    fn test_jaro_winkler_identical() {
        assert_eq!(jaro_winkler("hello", "hello"), 1.0);
    }

    #[test]
    fn test_jaro_winkler_with_custom_prefix() {
        let sim1 = jaro_winkler_with_prefix("hello", "help", 0.1);
        let sim2 = jaro_winkler_with_prefix("hello", "help", 0.2);
        // Higher prefix scale should give more weight to common prefix
        assert!(sim2 >= sim1);
    }

    #[test]
    fn test_jaro_no_matches() {
        let sim = jaro("abc", "xyz");
        assert!(sim < 0.5);
    }
}
