//! String Similarity Demonstration
//!
//! This example demonstrates how ELIDs preserve semantic similarity between text strings.
//! It uses a local Sentence-BERT model (all-MiniLM-L6-v2) to generate embeddings,
//! then shows how Hamming distance between ELIDs correlates with semantic similarity.
//!
//! Run with: cargo run --example string_similarity --features fastembed

#[cfg(feature = "fastembed")]
use elid_core::{encode, hamming_distance, Profile};
#[cfg(feature = "fastembed")]
use fastembed::TextEmbedding;

#[cfg(feature = "fastembed")]
fn main() -> anyhow::Result<()> {
    println!("ELID String Similarity Demonstration");
    println!("====================================\n");

    // Initialize the embedding model (downloads ~90MB on first run)
    println!("Loading Sentence-BERT model (all-MiniLM-L6-v2)...");
    let model = TextEmbedding::try_new(Default::default())?;
    println!("Model loaded successfully!\n");

    // Test string pairs with different levels of similarity
    let test_cases = vec![
        // Group 1: Nearly identical (only one word different)
        ("The cat sat on the mat", "The cat sat on the rug"),
        // Group 2: Synonymous phrases
        ("I am very happy", "I am very joyful"),
        (
            "The weather is beautiful today",
            "Today has gorgeous weather",
        ),
        // Group 3: Related but different meaning
        ("I love programming in Rust", "I enjoy writing Python code"),
        (
            "Database indexing is important",
            "Sorting algorithms are crucial",
        ),
        // Group 4: Completely unrelated
        ("The cat sat on the mat", "Quantum physics is fascinating"),
        ("I love chocolate cake", "Database indexing is important"),
        (
            "Neural networks are powerful",
            "The weather is beautiful today",
        ),
    ];

    println!("Generating embeddings and ELIDs...\n");

    let mut results = Vec::new();

    for (text1, text2) in &test_cases {
        // Generate embeddings
        let embeddings1 = model.embed(vec![text1.to_string()], None)?;
        let embeddings2 = model.embed(vec![text2.to_string()], None)?;

        let emb1_vec = &embeddings1[0];
        let emb2_vec = &embeddings2[0];

        // Encode to ELIDs (encode function handles embedding creation and normalization)
        let elid1 = encode(emb1_vec, &Profile::default())?;
        let elid2 = encode(emb2_vec, &Profile::default())?;

        // Calculate Hamming distance
        let hamming_dist = hamming_distance(&elid1, &elid2)?;

        // Calculate cosine similarity (for comparison)
        let cosine_sim = cosine_similarity(emb1_vec, emb2_vec);

        results.push((
            text1.to_string(),
            text2.to_string(),
            elid1.clone(),
            elid2.clone(),
            hamming_dist,
            cosine_sim,
        ));
    }

    // Display results
    println!(
        "{:<50} | {:<50} | Hamming | Cosine | Correlation",
        "Text 1", "Text 2"
    );
    println!("{}", "-".repeat(160));

    for (text1, text2, _elid1, _elid2, hamming_dist, cosine_sim) in &results {
        // Calculate correlation: Hamming distance should roughly correlate with (1 - cosine)
        // Lower cosine similarity → higher Hamming distance
        let expected_hamming = (1.0 - cosine_sim) * 128.0;
        let correlation = format!(
            "{:.1}%",
            (1.0 - (hamming_dist.abs_diff(expected_hamming as u32) as f32 / 128.0)) * 100.0
        );

        println!(
            "{:<50.47}... | {:<50.47}... | {:>7} | {:>6.3} | {:>11}",
            text1, text2, hamming_dist, cosine_sim, correlation
        );
    }

    println!("\n{}", "-".repeat(160));

    // Analysis
    println!("\nAnalysis:");
    println!("--------");
    println!("• Low Hamming distance (<40) = High semantic similarity");
    println!("• High Hamming distance (>80) = Low semantic similarity");
    println!("• Correlation shows how well Hamming distance approximates (1 - cosine_similarity)");
    println!("\nELID Properties:");
    println!("• Lexicographically sortable - similar strings cluster in databases");
    println!("• Constant length (26 chars for Mini128 profile)");
    println!("• Base32hex encoding for filesystem/URL safety");

    // Show example ELIDs
    if let Some((_, _, elid1, elid2, hamming_dist, cosine_sim)) = results.first() {
        println!("\nExample ELIDs (first pair):");
        println!("  Text 1 ELID: {}", elid1);
        println!("  Text 2 ELID: {}", elid2);
        println!("  Hamming Distance: {}", hamming_dist);
        println!("  Cosine Similarity: {:.3}", cosine_sim);

        // Show prefix sharing
        let prefix_len = elid1
            .as_str()
            .chars()
            .zip(elid2.as_str().chars())
            .take_while(|(a, b)| a == b)
            .count();
        println!(
            "  Shared Prefix Length: {} chars ({}%)",
            prefix_len,
            (prefix_len * 100) / 26
        );
    }

    Ok(())
}

/// Calculate cosine similarity between two vectors
#[cfg(feature = "fastembed")]
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let mag_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mag_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    dot / (mag_a * mag_b)
}

#[cfg(not(feature = "fastembed"))]
fn main() {
    eprintln!("This example requires the 'fastembed' feature to be enabled.");
    eprintln!("Run with: cargo run --example string_similarity --features fastembed");
    std::process::exit(1);
}
