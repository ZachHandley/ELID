//! Optimal String Alignment (OSA) distance implementation
//!
//! OSA distance is similar to Levenshtein distance but also considers transpositions
//! (swapping of adjacent characters) as a single edit operation.

/// Compute the Optimal String Alignment distance between two strings.
///
/// This is similar to Levenshtein distance but also counts transpositions
/// (swapping two adjacent characters) as a single operation.
///
/// # Example
///
/// ```rust
/// use elid::osa_distance;
///
/// // "ca" requires one transposition to become "ac"
/// let distance = osa_distance("ca", "ac");
/// assert_eq!(distance, 1);
///
/// // Compare with cases where transposition matters
/// let distance = osa_distance("abcd", "acbd");
/// assert_eq!(distance, 1); // One transposition: bc -> cb
/// ```
///
/// # Time Complexity
///
/// O(m * n) where m and n are the lengths of the input strings.
pub fn osa_distance(a: &str, b: &str) -> usize {
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

    // Create distance matrix
    let mut matrix = vec![vec![0; b_len + 1]; a_len + 1];

    // Initialize first row and column
    for i in 0..=a_len {
        matrix[i][0] = i;
    }
    for j in 0..=b_len {
        matrix[0][j] = j;
    }

    // Fill in the matrix
    for i in 1..=a_len {
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };

            let deletion = matrix[i - 1][j] + 1;
            let insertion = matrix[i][j - 1] + 1;
            let substitution = matrix[i - 1][j - 1] + cost;

            let mut min_dist = deletion.min(insertion).min(substitution);

            // Check for transposition
            if i > 1 && j > 1
                && a_chars[i - 1] == b_chars[j - 2]
                && a_chars[i - 2] == b_chars[j - 1]
            {
                let transposition = matrix[i - 2][j - 2] + 1;
                min_dist = min_dist.min(transposition);
            }

            matrix[i][j] = min_dist;
        }
    }

    matrix[a_len][b_len]
}

/// Compute the normalized OSA similarity between two strings.
///
/// Returns a value between 0.0 (completely different) and 1.0 (identical).
///
/// # Example
///
/// ```rust
/// use elid::osa_distance;
///
/// let distance = osa_distance("hello", "hello");
/// assert_eq!(distance, 0);
/// ```
pub fn normalized_osa(a: &str, b: &str) -> f64 {
    if a == b {
        return 1.0;
    }

    let distance = osa_distance(a, b);
    let max_len = a.chars().count().max(b.chars().count());

    if max_len == 0 {
        return 1.0;
    }

    1.0 - (distance as f64 / max_len as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_osa_identical() {
        assert_eq!(osa_distance("hello", "hello"), 0);
    }

    #[test]
    fn test_osa_empty() {
        assert_eq!(osa_distance("", "hello"), 5);
        assert_eq!(osa_distance("hello", ""), 5);
        assert_eq!(osa_distance("", ""), 0);
    }

    #[test]
    fn test_osa_transposition() {
        // Simple transposition
        assert_eq!(osa_distance("ca", "ac"), 1);
        assert_eq!(osa_distance("abcd", "acbd"), 1);

        // Multiple operations
        assert_eq!(osa_distance("abc", "bca"), 2);
    }

    #[test]
    fn test_osa_vs_levenshtein() {
        use crate::levenshtein;

        // For transpositions, OSA should be <= Levenshtein
        let osa = osa_distance("ca", "ac");
        let lev = levenshtein("ca", "ac");
        assert!(osa <= lev);
    }

    #[test]
    fn test_normalized_osa() {
        assert_eq!(normalized_osa("hello", "hello"), 1.0);
        assert!(normalized_osa("abc", "xyz") < 0.5);
    }
}
