//! Basic usage examples for the ELID library

use elid::*;

fn main() {
    println!("=== ELID String Similarity Examples ===\n");

    // Example 1: Levenshtein Distance
    println!("1. Levenshtein Distance:");
    let dist = levenshtein("kitten", "sitting");
    println!("   Distance between 'kitten' and 'sitting': {}", dist);

    let similarity = normalized_levenshtein("kitten", "sitting");
    println!("   Normalized similarity: {:.2}", similarity);
    println!();

    // Example 2: Jaro-Winkler Similarity (good for names)
    println!("2. Jaro-Winkler Similarity (best for names):");
    let names = vec![
        ("Martha", "Marhta"),
        ("John Smith", "Jon Smith"),
        ("DIXON", "DICKSON"),
    ];

    for (name1, name2) in names {
        let jaro_sim = jaro(name1, name2);
        let jw_sim = jaro_winkler(name1, name2);
        println!("   '{}' vs '{}':", name1, name2);
        println!("     Jaro: {:.3}, Jaro-Winkler: {:.3}", jaro_sim, jw_sim);
    }
    println!();

    // Example 3: Hamming Distance (equal-length strings)
    println!("3. Hamming Distance (for equal-length strings):");
    let dna_sequences = vec![
        ("ACGTACGT", "ACGTACCT"),
        ("AAAAAAAA", "AAAAAAAA"),
        ("11111111", "00000000"),
    ];

    for (seq1, seq2) in dna_sequences {
        if let Some(dist) = hamming(seq1, seq2) {
            println!("   '{}' vs '{}': {} differences", seq1, seq2, dist);
        }
    }
    println!();

    // Example 4: OSA Distance (with transpositions)
    println!("4. OSA Distance (handles transpositions):");
    let pairs = vec![
        ("ca", "ac"),
        ("abcd", "acbd"),
        ("hello", "hallo"),
    ];

    for (str1, str2) in pairs {
        let osa = osa_distance(str1, str2);
        let lev = levenshtein(str1, str2);
        println!("   '{}' vs '{}':", str1, str2);
        println!("     OSA: {}, Levenshtein: {}", osa, lev);
    }
    println!();

    // Example 5: Finding best match
    println!("5. Finding Best Match:");
    let candidates = vec![
        "apple",
        "application",
        "apply",
        "apricot",
        "banana",
    ];

    let query = "app";
    let (idx, score) = find_best_match(query, &candidates);
    println!("   Query: '{}'", query);
    println!("   Best match: '{}' (score: {:.3})", candidates[idx], score);
    println!();

    // Example 6: Finding all matches above threshold
    println!("6. Finding All Matches Above Threshold:");
    let matches = find_matches_above_threshold(query, &candidates, 0.5);
    println!("   Query: '{}', Threshold: 0.5", query);
    println!("   Matches:");
    for (idx, score) in matches {
        println!("     - '{}' (score: {:.3})", candidates[idx], score);
    }
    println!();

    // Example 7: Using SimilarityOpts
    println!("7. Using SimilarityOpts (case-insensitive, trimmed):");
    let opts = SimilarityOpts {
        case_sensitive: false,
        trim_whitespace: true,
        ..Default::default()
    };

    let pairs = vec![
        ("  HELLO  ", "hello"),
        ("World", "WORLD"),
        ("\tTest\n", "test"),
    ];

    for (str1, str2) in pairs {
        let dist = levenshtein_with_opts(str1, str2, &opts);
        println!("   '{}' vs '{}': distance = {}", str1, str2, dist);
    }
    println!();

    // Example 8: Unicode support
    println!("8. Unicode Support:");
    let unicode_pairs = vec![
        ("café", "cafe"),
        ("你好", "您好"),
        ("🎉🎊", "🎉"),
    ];

    for (str1, str2) in unicode_pairs {
        let dist = levenshtein(str1, str2);
        let sim = jaro(str1, str2);
        println!("   '{}' vs '{}':", str1, str2);
        println!("     Levenshtein: {}, Jaro: {:.3}", dist, sim);
    }
    println!();

    // Example 9: Real-world scenario - Product search
    println!("9. Real-world Example - Product Search:");
    let products = vec![
        "iPhone 14 Pro Max",
        "iPhone 14 Pro",
        "iPhone 14",
        "iPhone 13 Pro",
        "Samsung Galaxy S23",
        "Google Pixel 7",
    ];

    let searches = vec!["iphone 14 pro", "galaxy", "pixel"];

    for search in searches {
        // Find matches with normalized comparison (case-insensitive)
        let mut scored_products: Vec<_> = products
            .iter()
            .enumerate()
            .map(|(i, product)| {
                let lower_product = product.to_lowercase();
                let score = best_match(search, &lower_product);
                (i, score)
            })
            .collect();

        scored_products.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        println!("   Search: '{}'", search);
        println!("   Top 3 matches:");
        for (idx, score) in scored_products.iter().take(3) {
            println!("     - {} (score: {:.3})", products[*idx], score);
        }
        println!();
    }

    println!("=== End of Examples ===");
}
