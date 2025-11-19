//! Integration tests for the ELID library

use elid::*;

#[test]
fn test_all_algorithms_on_identical_strings() {
    let a = "hello";
    let b = "hello";

    assert_eq!(levenshtein(a, b), 0);
    assert_eq!(normalized_levenshtein(a, b), 1.0);
    assert_eq!(jaro(a, b), 1.0);
    assert_eq!(jaro_winkler(a, b), 1.0);
    assert_eq!(hamming(a, b), Some(0));
    assert_eq!(osa_distance(a, b), 0);
}

#[test]
fn test_all_algorithms_on_empty_strings() {
    let a = "";
    let b = "";

    assert_eq!(levenshtein(a, b), 0);
    assert_eq!(normalized_levenshtein(a, b), 1.0);
    assert_eq!(jaro(a, b), 1.0);
    assert_eq!(jaro_winkler(a, b), 1.0);
    assert_eq!(hamming(a, b), Some(0));
    assert_eq!(osa_distance(a, b), 0);
}

#[test]
fn test_levenshtein_comprehensive() {
    // Classic examples
    assert_eq!(levenshtein("kitten", "sitting"), 3);
    assert_eq!(levenshtein("saturday", "sunday"), 3);
    assert_eq!(levenshtein("book", "back"), 2);

    // Edge cases
    assert_eq!(levenshtein("", "abc"), 3);
    assert_eq!(levenshtein("abc", ""), 3);

    // Unicode support
    assert_eq!(levenshtein("café", "cafe"), 1);
    assert_eq!(levenshtein("你好", "您好"), 1);
}

#[test]
fn test_normalized_levenshtein_range() {
    // Should always be between 0.0 and 1.0
    let test_cases = vec![
        ("hello", "world"),
        ("abc", "xyz"),
        ("test", "testing"),
        ("", "something"),
        ("same", "same"),
    ];

    for (a, b) in test_cases {
        let similarity = normalized_levenshtein(a, b);
        assert!(
            (0.0..=1.0).contains(&similarity),
            "Similarity {} is out of range for '{}' and '{}'",
            similarity,
            a,
            b
        );
    }
}

#[test]
fn test_jaro_comprehensive() {
    // Classic examples
    let sim = jaro("martha", "marhta");
    assert!(sim > 0.9, "Jaro similarity for martha/marhta should be > 0.9");

    let sim = jaro("DIXON", "DICKSON");
    assert!(sim > 0.7, "Jaro similarity for DIXON/DICKSON should be > 0.7");

    // Completely different strings
    let sim = jaro("abc", "xyz");
    assert!(sim < 0.5, "Jaro similarity for abc/xyz should be < 0.5");
}

#[test]
fn test_jaro_winkler_prefix_advantage() {
    // Strings with common prefixes should score higher with Jaro-Winkler
    let test_cases = vec![
        ("DIXON", "DICKSON"),
        ("hello", "help"),
        ("prefix", "preface"),
    ];

    for (a, b) in test_cases {
        let jaro_sim = jaro(a, b);
        let jw_sim = jaro_winkler(a, b);
        assert!(
            jw_sim >= jaro_sim,
            "Jaro-Winkler ({}) should be >= Jaro ({}) for '{}' and '{}'",
            jw_sim,
            jaro_sim,
            a,
            b
        );
    }
}

#[test]
fn test_hamming_distance() {
    // Valid cases (same length)
    assert_eq!(hamming("karolin", "kathrin"), Some(3));
    assert_eq!(hamming("1011101", "1001001"), Some(2));
    assert_eq!(hamming("hello", "hallo"), Some(1));

    // Invalid cases (different lengths)
    assert_eq!(hamming("hello", "world!"), None);
    assert_eq!(hamming("hi", "hello"), None);
}

#[test]
fn test_osa_transposition_handling() {
    // OSA should count transpositions as single operations
    assert_eq!(osa_distance("ca", "ac"), 1);
    assert_eq!(osa_distance("abcd", "acbd"), 1);

    // Compare with Levenshtein to ensure OSA is handling transpositions
    let osa = osa_distance("ca", "ac");
    let lev = levenshtein("ca", "ac");
    assert!(
        osa <= lev,
        "OSA distance ({}) should be <= Levenshtein ({}) for transpositions",
        osa,
        lev
    );
}

#[test]
fn test_similarity_opts_case_insensitive() {
    let opts = SimilarityOpts {
        case_sensitive: false,
        trim_whitespace: false,
        ..Default::default()
    };

    let dist = levenshtein_with_opts("HELLO", "hello", &opts);
    assert_eq!(dist, 0, "Case-insensitive comparison should match");

    let dist = levenshtein_with_opts("ABC", "abc", &opts);
    assert_eq!(dist, 0, "Case-insensitive comparison should match");
}

#[test]
fn test_similarity_opts_trim_whitespace() {
    let opts = SimilarityOpts {
        case_sensitive: true,
        trim_whitespace: true,
        ..Default::default()
    };

    let dist = levenshtein_with_opts("  hello  ", "hello", &opts);
    assert_eq!(dist, 0, "Trimmed strings should match");

    let dist = levenshtein_with_opts("\thello\n", "hello", &opts);
    assert_eq!(dist, 0, "Trimmed strings should match");
}

#[test]
fn test_similarity_opts_combined() {
    let opts = SimilarityOpts {
        case_sensitive: false,
        trim_whitespace: true,
        ..Default::default()
    };

    let dist = levenshtein_with_opts("  HELLO  ", "hello", &opts);
    assert_eq!(dist, 0, "Case-insensitive and trimmed strings should match");
}

#[test]
fn test_best_match_function() {
    // best_match should return the highest score from multiple algorithms
    let score = best_match("hello", "hallo");
    assert!(
        score > 0.7,
        "best_match should find good similarity for hello/hallo"
    );

    let score = best_match("identical", "identical");
    assert_eq!(score, 1.0, "Identical strings should have score of 1.0");

    let score = best_match("abc", "xyz");
    assert!(
        score < 0.5,
        "Completely different strings should have low score"
    );
}

#[test]
fn test_find_best_match_in_candidates() {
    let candidates = vec!["apple", "application", "apply", "banana"];

    let (idx, score) = find_best_match("app", &candidates);
    assert!(
        score > 0.5,
        "Should find a good match for 'app' in candidates"
    );
    assert!(
        candidates[idx].starts_with("app"),
        "Best match should start with 'app'"
    );
}

#[test]
fn test_find_matches_above_threshold() {
    let candidates = vec!["apple", "application", "apply", "apricot", "banana"];

    let matches = find_matches_above_threshold("app", &candidates, 0.5);
    assert!(
        matches.len() >= 2,
        "Should find at least 2 matches above threshold"
    );

    // All matches should have scores above the threshold
    for (_, score) in &matches {
        assert!(
            *score >= 0.5,
            "All matches should have scores >= threshold"
        );
    }

    // Test with high threshold
    let matches = find_matches_above_threshold("app", &candidates, 0.95);
    assert!(
        matches.len() <= 2,
        "Should find few matches with very high threshold"
    );
}

#[test]
fn test_unicode_support() {
    // Test various Unicode strings
    assert_eq!(levenshtein("café", "cafe"), 1);
    assert_eq!(levenshtein("你好", "您好"), 1);
    assert_eq!(levenshtein("🎉🎊", "🎉"), 1);

    // Jaro with Unicode
    let sim = jaro("café", "cafe");
    assert!(sim > 0.7, "Jaro should handle Unicode");

    // Hamming with Unicode (same length in chars)
    assert_eq!(hamming("你好", "您好"), Some(1));
}

#[test]
fn test_real_world_scenarios() {
    // Name matching
    let sim = jaro_winkler("John Smith", "Jon Smith");
    assert!(sim > 0.8, "Should match similar names");

    // Typo detection
    let dist = levenshtein("receive", "recieve");
    assert_eq!(dist, 2, "Should detect common typos");

    // DNA sequence comparison (Hamming)
    let dist = hamming("ACGTACGT", "ACGTACCT");
    assert_eq!(dist, Some(1), "Should compare DNA sequences");

    // Product search
    let candidates = vec![
        "iPhone 14 Pro",
        "iPhone 14",
        "iPhone 13 Pro",
        "Samsung Galaxy",
    ];
    let (idx, _) = find_best_match("iphone 14 pro", &candidates);
    assert!(
        candidates[idx].to_lowercase().contains("iphone 14 pro"),
        "Should find the right product"
    );
}

#[test]
fn test_performance_on_long_strings() {
    // Test that the algorithms work on reasonably long strings
    let long_a = "a".repeat(100);
    let long_b = "a".repeat(99) + "b";

    let dist = levenshtein(&long_a, &long_b);
    assert_eq!(dist, 1, "Should handle long strings");

    let sim = jaro(&long_a, &long_b);
    assert!(sim > 0.95, "Jaro should handle long strings");
}

#[test]
fn test_symmetry() {
    // Test that distance metrics are symmetric
    let test_pairs = vec![
        ("hello", "world"),
        ("abc", "xyz"),
        ("test", "testing"),
    ];

    for (a, b) in test_pairs {
        assert_eq!(
            levenshtein(a, b),
            levenshtein(b, a),
            "Levenshtein should be symmetric"
        );
        assert_eq!(jaro(a, b), jaro(b, a), "Jaro should be symmetric");
        assert_eq!(
            jaro_winkler(a, b),
            jaro_winkler(b, a),
            "Jaro-Winkler should be symmetric"
        );
        assert_eq!(
            osa_distance(a, b),
            osa_distance(b, a),
            "OSA should be symmetric"
        );
        assert_eq!(
            hamming(a, b),
            hamming(b, a),
            "Hamming should be symmetric"
        );
    }
}
