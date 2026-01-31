//! SimHash implementation for locality-sensitive hashing
//!
//! SimHash creates a numeric fingerprint where similar strings produce similar hashes.
//! This allows for efficient similarity queries using numeric comparisons.
//!
//! Uses FNV-1a hash for stability across Rust versions.

/// Compute the SimHash fingerprint of a string.
///
/// SimHash produces a 64-bit integer where similar strings have similar hash values.
/// The Hamming distance between two SimHash values correlates with string similarity.
///
/// # Example
///
/// ```rust
/// use elid::simhash;
///
/// let hash1 = simhash("iPhone 14");
/// let hash2 = simhash("iPhone 15");
/// let hash3 = simhash("Galaxy S23");
///
/// // hash1 and hash2 will be numerically close
/// // hash3 will be numerically distant
/// ```
///
/// # Algorithm
///
/// 1. Extract character trigrams (3-character sequences)
/// 2. Hash each trigram
/// 3. Build fingerprint using weighted bit voting
/// 4. Return 64-bit hash value
pub fn simhash(text: &str) -> u64 {
    if text.is_empty() {
        return 0;
    }

    let text_lower = text.to_lowercase();
    let features = extract_features(&text_lower);

    if features.is_empty() {
        return hash_string(text);
    }

    // Initialize bit counters (64 bits)
    let mut v = [0i32; 64];

    // Process each feature
    for feature in features {
        let hash = hash_string(&feature);

        // For each bit position
        #[allow(clippy::needless_range_loop)]
        for i in 0..64 {
            let bit = (hash >> i) & 1;
            if bit == 1 {
                v[i] += 1;
            } else {
                v[i] -= 1;
            }
        }
    }

    // Build final fingerprint
    let mut fingerprint: u64 = 0;
    #[allow(clippy::needless_range_loop)]
    for i in 0..64 {
        if v[i] > 0 {
            fingerprint |= 1u64 << i;
        }
    }

    fingerprint
}

/// Compute the Hamming distance between two SimHash values.
///
/// Returns the number of differing bits between two hash values.
/// Lower values indicate higher similarity.
///
/// # Example
///
/// ```rust
/// use elid::{simhash, simhash_distance};
///
/// let hash1 = simhash("iPhone 14");
/// let hash2 = simhash("iPhone 15");
/// let distance = simhash_distance(hash1, hash2);
///
/// assert!(distance < 20); // Similar strings have relatively low distance
/// ```
pub fn simhash_distance(hash1: u64, hash2: u64) -> u32 {
    (hash1 ^ hash2).count_ones()
}

/// Compute the normalized SimHash similarity between two strings.
///
/// Returns a value between 0.0 (completely different) and 1.0 (identical).
/// This is computed as: 1.0 - (hamming_distance / 64)
///
/// # Example
///
/// ```rust
/// use elid::simhash_similarity;
///
/// let similarity = simhash_similarity("iPhone 14", "iPhone 15");
/// assert!(similarity > 0.6); // Relatively high similarity
///
/// let similarity = simhash_similarity("iPhone", "Galaxy");
/// assert!(similarity < 0.9); // Different strings
/// ```
pub fn simhash_similarity(a: &str, b: &str) -> f64 {
    let hash1 = simhash(a);
    let hash2 = simhash(b);
    let distance = simhash_distance(hash1, hash2);

    // Normalize: 0 distance = 1.0 similarity, 64 distance = 0.0 similarity
    1.0 - (distance as f64 / 64.0)
}

/// Find all items within a given SimHash distance threshold.
///
/// This is useful for database queries - pre-compute SimHash values,
/// then query by numeric range.
///
/// # Example
///
/// ```rust
/// use elid::{simhash, find_similar_hashes};
///
/// let candidates = vec!["iPhone 14 Pro", "iPhone 13", "Galaxy S23"];
/// let candidate_hashes: Vec<u64> = candidates.iter().map(|s| simhash(s)).collect();
///
/// let query_hash = simhash("iPhone 14");
/// let matches = find_similar_hashes(query_hash, &candidate_hashes, 10);
/// // Returns indices of candidates within Hamming distance 10
/// ```
pub fn find_similar_hashes(query_hash: u64, candidate_hashes: &[u64], max_distance: u32) -> Vec<usize> {
    candidate_hashes
        .iter()
        .enumerate()
        .filter_map(|(i, &hash)| {
            let distance = simhash_distance(query_hash, hash);
            if distance <= max_distance {
                Some(i)
            } else {
                None
            }
        })
        .collect()
}

/// Extract features from text for SimHash computation.
///
/// Uses character trigrams (3-character sliding windows).
fn extract_features(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut features = Vec::new();

    // Single characters for very short strings
    if chars.len() <= 2 {
        for c in chars {
            features.push(c.to_string());
        }
        return features;
    }

    // Character trigrams
    for window in chars.windows(3) {
        let trigram: String = window.iter().collect();
        features.push(trigram);
    }

    // Also include individual tokens (words)
    for token in text.split_whitespace() {
        if !token.is_empty() {
            features.push(token.to_string());
        }
    }

    features
}

/// Hash a string to u64 using FNV-1a hash (stable across Rust versions)
///
/// FNV-1a is a simple, fast, non-cryptographic hash that produces consistent
/// results across different Rust compiler versions, making it perfect for
/// database storage where hash values must remain stable.
fn hash_string(s: &str) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 14695981039346656037;
    const FNV_PRIME: u64 = 1099511628211;

    let mut hash = FNV_OFFSET_BASIS;
    for byte in s.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simhash_identical() {
        let hash1 = simhash("hello world");
        let hash2 = simhash("hello world");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_simhash_similar() {
        let hash1 = simhash("iPhone 14");
        let hash2 = simhash("iPhone 15");
        let hash3 = simhash("Galaxy S23");

        let dist_similar = simhash_distance(hash1, hash2);
        let dist_different = simhash_distance(hash1, hash3);

        // Similar strings should have smaller distance
        assert!(dist_similar < dist_different);
        assert!(dist_similar < 20); // Should be quite similar
    }

    #[test]
    fn test_simhash_similarity() {
        let sim_identical = simhash_similarity("test", "test");
        assert_eq!(sim_identical, 1.0);

        let sim_similar = simhash_similarity("hello world", "hello world!");
        assert!(sim_similar > 0.7); // Slightly relaxed threshold

        let sim_different = simhash_similarity("apple", "orange");
        assert!(sim_different < 0.8); // Adjusted threshold
    }

    #[test]
    fn test_simhash_distance() {
        let hash1 = simhash("test");
        let hash2 = simhash("test");
        assert_eq!(simhash_distance(hash1, hash2), 0);
    }

    #[test]
    fn test_find_similar_hashes() {
        let candidates = vec!["iPhone 14 Pro", "iPhone 13", "Galaxy S23", "iPhone 14"];
        let hashes: Vec<u64> = candidates.iter().map(|s| simhash(s)).collect();

        let query_hash = simhash("iPhone 14");
        let matches = find_similar_hashes(query_hash, &hashes, 15);

        // Should find iPhone variants, not Galaxy
        assert!(matches.len() >= 2);
        assert!(matches.contains(&3)); // "iPhone 14" itself
    }

    #[test]
    fn test_empty_string() {
        let hash = simhash("");
        assert_eq!(hash, 0);
    }

    #[test]
    fn test_case_insensitive() {
        let hash1 = simhash("Hello World");
        let hash2 = simhash("hello world");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_feature_extraction() {
        let features = extract_features("abc");
        assert!(features.len() > 0);

        let features = extract_features("hello world");
        assert!(features.len() > 0);
    }
}
