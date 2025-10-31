//! Basic usage example demonstrating core ELID functionality
//!
//! This example shows:
//! - Loading and encoding embeddings with all three profiles
//! - Computing Hamming distance for similarity search
//! - Using hamming_neighbors to find similar items
//! - Decoding ELIDs to inspect their contents
//!
//! Run with: cargo run --example basic_usage

use elid_core::{decode, encode, hamming_distance, Profile};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ELID Basic Usage Example ===\n");

    // ========================================================================
    // 1. Create Sample Embeddings
    // ========================================================================
    println!("1. Creating sample embeddings...");

    // In real applications, these come from ML models (BERT, OpenAI, etc.)
    // For demo purposes, we'll create synthetic embeddings with slight variations

    // Base embedding - imagine this is from "The quick brown fox"
    let embedding1: Vec<f32> = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8]
        .into_iter()
        .cycle()
        .take(768) // Typical BERT dimension
        .collect();

    // Similar embedding - imagine this is from "The fast brown fox"
    let mut embedding2 = embedding1.clone();
    embedding2[0] = 0.11; // Slight change in first dimension
    embedding2[1] = 0.21;

    // Different embedding - imagine this is from "Blue sky and clouds"
    let embedding3: Vec<f32> = vec![-0.5, -0.3, 0.2, 0.8, -0.1, 0.4, -0.7, 0.6]
        .into_iter()
        .cycle()
        .take(768)
        .collect();

    println!("Created 3 embeddings (768 dimensions each)");
    println!("- embedding1 and embedding2 are similar");
    println!("- embedding3 is different\n");

    // ========================================================================
    // 2. Encode with Mini128 Profile (Similarity Search)
    // ========================================================================
    println!("2. Encoding with Mini128 profile (for similarity search)...");

    let profile_mini128 = Profile::default(); // Mini128 is the default

    let elid1 = encode(&embedding1, &profile_mini128)?;
    let elid2 = encode(&embedding2, &profile_mini128)?;
    let elid3 = encode(&embedding3, &profile_mini128)?;

    println!("ELID 1: {}", elid1);
    println!("ELID 2: {}", elid2);
    println!("ELID 3: {}", elid3);
    println!("Length: {} characters\n", elid1.as_str().len());

    // ========================================================================
    // 3. Compute Hamming Distance
    // ========================================================================
    println!("3. Computing Hamming distances...");

    let dist_1_2 = hamming_distance(&elid1, &elid2)?;
    let dist_1_3 = hamming_distance(&elid1, &elid3)?;
    let dist_2_3 = hamming_distance(&elid2, &elid3)?;

    println!(
        "Distance (ELID1 <-> ELID2): {}/128 (similar embeddings)",
        dist_1_2
    );
    println!(
        "Distance (ELID1 <-> ELID3): {}/128 (different embeddings)",
        dist_1_3
    );
    println!(
        "Distance (ELID2 <-> ELID3): {}/128 (different embeddings)",
        dist_2_3
    );

    // Interpretation guide
    println!("\nInterpretation:");
    println!("  0-30:   Very similar (cosine similarity > 0.85)");
    println!("  31-64:  Somewhat similar (cosine similarity 0.5-0.85)");
    println!("  65-128: Dissimilar (cosine similarity < 0.5)\n");

    // ========================================================================
    // 4. Find Similar Items with hamming_neighbors
    // ========================================================================
    println!("4. Finding neighbors with Hamming distance search...");

    // For demonstration, let's manually check distances among our ELIDs
    let query_elid = &elid1;
    let candidates = [&elid1, &elid2, &elid3];
    let max_distance = 30;

    println!("Query: {}", query_elid);
    println!("Checking candidates within distance {}:", max_distance);

    let mut neighbors_found = 0;
    for candidate in candidates.iter() {
        let dist = hamming_distance(query_elid, candidate)?;
        if dist <= max_distance {
            neighbors_found += 1;
            println!("  {}. {} (distance: {})", neighbors_found, candidate, dist);
        }
    }

    if neighbors_found == 0 {
        println!("  No neighbors found within threshold");
    }

    println!("\nNote: hamming_neighbors() generates all possible ELIDs within a radius");
    println!("      (useful for database queries, but exponential in radius)");
    println!();

    // ========================================================================
    // 5. Encode with Morton Profile (Fast Database Indexing)
    // ========================================================================
    println!("5. Encoding with Morton10x10 profile (for database indexing)...");

    let profile_morton = Profile::Morton10x10 {
        dims: 10,
        bits_per_dim: 10,
        transform_id: None,
    };

    let morton1 = encode(&embedding1, &profile_morton)?;
    let morton2 = encode(&embedding2, &profile_morton)?;

    println!("Morton ELID 1: {}", morton1);
    println!("Morton ELID 2: {}", morton2);
    println!("Length: {} characters", morton1.as_str().len());
    println!("Note: Morton ELIDs are sortable and cluster nearby embeddings\n");

    // ========================================================================
    // 6. Encode with Hilbert Profile (Best Locality)
    // ========================================================================
    println!("6. Encoding with Hilbert10x10 profile (best locality)...");

    let profile_hilbert = Profile::Hilbert10x10 {
        dims: 10,
        bits_per_dim: 10,
        transform_id: None,
    };

    let hilbert1 = encode(&embedding1, &profile_hilbert)?;
    let hilbert2 = encode(&embedding2, &profile_hilbert)?;

    println!("Hilbert ELID 1: {}", hilbert1);
    println!("Hilbert ELID 2: {}", hilbert2);
    println!("Length: {} characters", hilbert1.as_str().len());
    println!("Note: Hilbert has 5-10% better locality than Morton, but slower encoding\n");

    // ========================================================================
    // 7. Decode ELIDs to Inspect Contents
    // ========================================================================
    println!("7. Decoding ELID to inspect raw bytes...");

    let bytes = decode(&elid1)?;
    println!("ELID:  {}", elid1);
    println!("Bytes: {} total", bytes.len());
    println!("       [{:?}...]", &bytes[..std::cmp::min(8, bytes.len())]);

    // Extract header information
    let profile_info = elid1.profile()?;
    println!("\nProfile Info:");
    println!("  Version:      {}", profile_info.version);
    println!(
        "  Profile Type: 0x{:02x} (Mini128)",
        profile_info.profile_type
    );
    println!();

    // ========================================================================
    // 8. Demonstrate Use Case: Deduplication
    // ========================================================================
    println!("8. Use case example: Deduplication...");

    let test_embeddings = [
        embedding1.clone(),
        embedding2.clone(), // Similar to embedding1
        embedding3.clone(),
        embedding1.clone(), // Duplicate of embedding1
    ];

    let threshold = 15; // Hamming distance threshold for "similar enough"
    let mut unique_elids = Vec::new();

    println!(
        "Processing {} embeddings with threshold {}...",
        test_embeddings.len(),
        threshold
    );

    for (idx, emb) in test_embeddings.iter().enumerate() {
        let elid = encode(emb, &profile_mini128)?;

        // Check if similar ELID already exists
        let is_duplicate = unique_elids.iter().any(|existing: &elid_core::Elid| {
            hamming_distance(&elid, existing).unwrap() < threshold
        });

        if is_duplicate {
            println!("  Embedding {}: DUPLICATE (skipped)", idx + 1);
        } else {
            println!("  Embedding {}: UNIQUE (kept)", idx + 1);
            unique_elids.push(elid);
        }
    }

    println!(
        "\nResult: {} unique embeddings (out of {})\n",
        unique_elids.len(),
        test_embeddings.len()
    );

    // ========================================================================
    // 9. Performance Note
    // ========================================================================
    println!("9. Performance characteristics:");
    println!("  - Encoding:         ~10-50 μs per embedding (Mini128)");
    println!("  - Hamming distance: ~10 ns (single CPU instruction)");
    println!("  - Memory:           Zero allocations for distance computation");
    println!("  - Throughput:       ~20,000-100,000 embeddings/sec (single thread)\n");

    // ========================================================================
    // Summary
    // ========================================================================
    println!("=== Summary ===");
    println!("This example demonstrated:");
    println!("  ✓ Encoding embeddings with three profiles");
    println!("  ✓ Computing Hamming distance for similarity");
    println!("  ✓ Finding neighbors within a distance threshold");
    println!("  ✓ Decoding ELIDs to inspect contents");
    println!("  ✓ Practical deduplication use case");
    println!("\nFor more examples, see:");
    println!("  - batch_encoding.rs: Efficient batch processing");
    println!("  - hamming_neighbors.rs: Advanced neighbor search");
    println!("\nFor API documentation: cargo doc --open");

    Ok(())
}
