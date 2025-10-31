//! Demonstration of SimHash-128 deterministic output and locality preservation

use elid_core::simhash::{hamming_distance_128, simhash_128};

fn main() {
    println!("=== SimHash-128 Demonstration ===\n");

    // Example 1: Deterministic hashing
    println!("1. Deterministic Hashing:");
    let embedding = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8];
    let embedding = embedding.into_iter().cycle().take(768).collect::<Vec<_>>();
    let seed = 0x454c4944_53494d48; // "ELIDSIMH"

    let hash1 = simhash_128(&embedding, seed);
    let hash2 = simhash_128(&embedding, seed);

    println!("   Embedding: [0.1, 0.2, ..., 0.8] (768 dims, repeating pattern)");
    println!("   Seed: 0x{:016x}", seed);
    println!("   Hash 1: 0x{:032x}", hash1);
    println!("   Hash 2: 0x{:032x}", hash2);
    println!("   Match: {}\n", hash1 == hash2);

    // Example 2: Similar embeddings have low Hamming distance
    println!("2. Locality Preservation:");
    let base = vec![0.5; 256];
    let mut similar = base.clone();
    similar[0] = 0.51; // Slight perturbation

    let hash_base = simhash_128(&base, seed);
    let hash_similar = simhash_128(&similar, seed);
    let dist = hamming_distance_128(hash_base, hash_similar);

    println!("   Base embedding: [0.5; 256]");
    println!("   Similar embedding: [0.51, 0.5, 0.5, ...] (one element changed by 0.01)");
    println!("   Hash base:    0x{:032x}", hash_base);
    println!("   Hash similar: 0x{:032x}", hash_similar);
    println!("   Hamming distance: {} / 128 bits\n", dist);

    // Example 3: Different embeddings have high Hamming distance
    println!("3. Dissimilar Embeddings:");
    let emb_a = vec![1.0, 0.0, 0.0, 0.0]
        .into_iter()
        .cycle()
        .take(256)
        .collect::<Vec<_>>();
    let emb_b = vec![0.0, 1.0, 0.0, 0.0]
        .into_iter()
        .cycle()
        .take(256)
        .collect::<Vec<_>>();

    let hash_a = simhash_128(&emb_a, seed);
    let hash_b = simhash_128(&emb_b, seed);
    let dist_ab = hamming_distance_128(hash_a, hash_b);

    println!("   Embedding A: [1.0, 0.0, 0.0, 0.0, ...] (orthogonal vector 1)");
    println!("   Embedding B: [0.0, 1.0, 0.0, 0.0, ...] (orthogonal vector 2)");
    println!("   Hash A: 0x{:032x}", hash_a);
    println!("   Hash B: 0x{:032x}", hash_b);
    println!("   Hamming distance: {} / 128 bits\n", dist_ab);

    // Example 4: Different seeds produce different hashes
    println!("4. Seed Independence:");
    let test_emb = vec![0.3; 128];
    let seed1 = 0x1111111111111111;
    let seed2 = 0x2222222222222222;

    let hash_s1 = simhash_128(&test_emb, seed1);
    let hash_s2 = simhash_128(&test_emb, seed2);

    println!("   Embedding: [0.3; 128]");
    println!("   Seed 1: 0x{:016x} → Hash: 0x{:032x}", seed1, hash_s1);
    println!("   Seed 2: 0x{:016x} → Hash: 0x{:032x}", seed2, hash_s2);
    println!(
        "   Hamming distance: {} / 128 bits\n",
        hamming_distance_128(hash_s1, hash_s2)
    );

    println!("=== End of Demo ===");
}
