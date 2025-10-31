//! Example demonstrating SimHash utility functions
//!
//! Run with: cargo run --example simhash_utilities

use elid_core::simhash::{
    cosine_similarity_approx, hamming_distance_128, simhash_128, simhash_from_bytes,
    simhash_to_bytes,
};

fn main() {
    println!("=== SimHash Utility Functions Demo ===\n");

    // Create sample embeddings
    let seed = 0x454c4944_53494d48; // "ELIDSIMH"

    let emb1 = vec![0.1, 0.2, 0.3, 0.4, 0.5]
        .into_iter()
        .cycle()
        .take(768)
        .collect::<Vec<_>>();

    let emb2 = vec![0.15, 0.25, 0.35, 0.45, 0.55] // Similar to emb1
        .into_iter()
        .cycle()
        .take(768)
        .collect::<Vec<_>>();

    let emb3 = vec![-0.8, -0.6, -0.4, -0.2, 0.0] // Very different from emb1
        .into_iter()
        .cycle()
        .take(768)
        .collect::<Vec<_>>();

    // Generate hashes
    let hash1 = simhash_128(&emb1, seed);
    let hash2 = simhash_128(&emb2, seed);
    let hash3 = simhash_128(&emb3, seed);

    println!("Generated SimHashes:");
    println!("  hash1: 0x{:032x}", hash1);
    println!("  hash2: 0x{:032x}", hash2);
    println!("  hash3: 0x{:032x}\n", hash3);

    // T018: Hamming distance function
    println!("=== T018: Hamming Distance ===");
    let dist_12 = hamming_distance_128(hash1, hash2);
    let dist_13 = hamming_distance_128(hash1, hash3);
    let dist_11 = hamming_distance_128(hash1, hash1);

    println!("  hamming_distance(hash1, hash2) = {} bits", dist_12);
    println!("  hamming_distance(hash1, hash3) = {} bits", dist_13);
    println!("  hamming_distance(hash1, hash1) = {} bits", dist_11);
    println!("  Note: Uses hardware popcnt instruction (~3-5 CPU cycles)\n");

    // T019: SimHash to/from bytes conversion
    println!("=== T019: Byte Conversion ===");
    let bytes1 = simhash_to_bytes(hash1);
    println!(
        "  simhash_to_bytes(hash1) = [{:02x} {:02x} {:02x} {:02x} ... {:02x} {:02x}]",
        bytes1[0], bytes1[1], bytes1[2], bytes1[3], bytes1[14], bytes1[15]
    );
    println!("  Byte array length: {} bytes", bytes1.len());

    let recovered = simhash_from_bytes(&bytes1).expect("Valid bytes");
    println!("  simhash_from_bytes(bytes1) = 0x{:032x}", recovered);
    println!("  Round-trip successful: {}\n", hash1 == recovered);

    // Test invalid byte array
    let invalid_bytes = [0u8; 8]; // Wrong size
    match simhash_from_bytes(&invalid_bytes) {
        Ok(_) => println!("  ERROR: Should have failed with invalid length"),
        Err(e) => println!("  simhash_from_bytes([0; 8]) = Err({:?})", e),
    }
    println!();

    // T020: Cosine similarity approximation
    println!("=== T020: Cosine Similarity Approximation ===");
    let sim_12 = cosine_similarity_approx(hash1, hash2);
    let sim_13 = cosine_similarity_approx(hash1, hash3);
    let sim_11 = cosine_similarity_approx(hash1, hash1);

    println!("  cosine_similarity_approx(hash1, hash2) = {:.4}", sim_12);
    println!("    (Hamming distance: {} → Similar embeddings)", dist_12);

    println!("  cosine_similarity_approx(hash1, hash3) = {:.4}", sim_13);
    println!(
        "    (Hamming distance: {} → Dissimilar embeddings)",
        dist_13
    );

    println!("  cosine_similarity_approx(hash1, hash1) = {:.4}", sim_11);
    println!("    (Hamming distance: {} → Identical)", dist_11);

    println!("\n  Formula: cos_sim ≈ 1 - (hamming_dist / 128) × π");
    println!("  Based on Charikar 2002 theorem");

    // Demonstrate the formula
    println!("\n=== Formula Verification ===");
    for (label, hash_a, hash_b) in [
        ("hash1 vs hash2", hash1, hash2),
        ("hash1 vs hash3", hash1, hash3),
    ] {
        let dist = hamming_distance_128(hash_a, hash_b);
        let sim = cosine_similarity_approx(hash_a, hash_b);
        let expected = 1.0 - (dist as f32 / 128.0) * std::f32::consts::PI;

        println!("  {}: ", label);
        println!("    Distance: {}", dist);
        println!("    Expected: {:.4}", expected);
        println!("    Actual:   {:.4}", sim);
        println!("    Match:    {}", (sim - expected).abs() < 0.0001);
    }

    println!("\n=== Performance Characteristics ===");
    println!("  hamming_distance_128: O(1) - single popcnt instruction");
    println!("  simhash_to_bytes:     O(1) - simple endian conversion");
    println!("  simhash_from_bytes:   O(1) - validation + conversion");
    println!("  cosine_similarity:    O(1) - popcnt + arithmetic");
}
