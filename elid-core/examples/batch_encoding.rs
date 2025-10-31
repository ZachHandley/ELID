//! Batch encoding example showing efficient processing of multiple embeddings
//!
//! This example demonstrates:
//! - Sequential batch processing with progress tracking
//! - Efficient memory usage patterns
//! - Error handling for batch operations
//! - Performance measurement and reporting
//!
//! Note: Parallel encoding with Rayon is noted as optional for MVP.
//! This example focuses on sequential processing with best practices.
//!
//! Run with: cargo run --example batch_encoding

use elid_core::{encode, hamming_distance, Profile};
use std::time::Instant;

/// Simple progress tracker for batch operations
struct ProgressTracker {
    total: usize,
    processed: usize,
    start_time: Instant,
    last_report: usize,
}

impl ProgressTracker {
    fn new(total: usize) -> Self {
        Self {
            total,
            processed: 0,
            start_time: Instant::now(),
            last_report: 0,
        }
    }

    fn increment(&mut self) {
        self.processed += 1;

        // Report progress every 10% or 1000 items, whichever is smaller
        let report_interval = std::cmp::min(self.total / 10, 1000).max(1);

        if self.processed - self.last_report >= report_interval || self.processed == self.total {
            self.report();
            self.last_report = self.processed;
        }
    }

    fn report(&self) {
        let elapsed = self.start_time.elapsed();
        let rate = self.processed as f64 / elapsed.as_secs_f64();
        let percent = (self.processed as f64 / self.total as f64) * 100.0;

        eprintln!(
            "Progress: {}/{} ({:.1}%) - {:.0} embeddings/sec",
            self.processed, self.total, percent, rate
        );
    }

    fn finish(&self) {
        let elapsed = self.start_time.elapsed();
        let rate = self.total as f64 / elapsed.as_secs_f64();

        eprintln!(
            "\nCompleted {} embeddings in {:.2}s ({:.0} embeddings/sec)",
            self.total,
            elapsed.as_secs_f64(),
            rate
        );
    }
}

/// Generate synthetic embeddings for demonstration
///
/// In real applications, embeddings come from ML models.
/// For this example, we generate embeddings with controlled variation.
fn generate_embeddings(count: usize, dimensions: usize) -> Vec<Vec<f32>> {
    println!(
        "Generating {} embeddings with {} dimensions...",
        count, dimensions
    );

    let mut embeddings = Vec::with_capacity(count);

    for i in 0..count {
        // Create embedding with slight variation based on index
        let base_value = (i as f32 / count as f32) - 0.5; // Range: -0.5 to 0.5
        let embedding: Vec<f32> = (0..dimensions)
            .map(|j| {
                let variation = (j as f32 / dimensions as f32) * 0.1;
                base_value + variation
            })
            .collect();

        embeddings.push(embedding);
    }

    println!("Generated {} embeddings\n", embeddings.len());
    embeddings
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ELID Batch Encoding Example ===\n");

    // ========================================================================
    // 1. Configuration
    // ========================================================================
    let batch_size = 5000; // Number of embeddings to process
    let dimensions = 768; // Typical BERT dimension

    println!("Configuration:");
    println!("  Batch size:  {} embeddings", batch_size);
    println!("  Dimensions:  {}", dimensions);
    println!();

    // ========================================================================
    // 2. Generate Test Data
    // ========================================================================
    let embeddings = generate_embeddings(batch_size, dimensions);

    // ========================================================================
    // 3. Sequential Batch Encoding with Mini128
    // ========================================================================
    println!("3. Encoding with Mini128 profile...");

    let profile = Profile::default(); // Mini128
    let mut progress = ProgressTracker::new(embeddings.len());
    let mut elids = Vec::with_capacity(embeddings.len());
    let mut errors = 0;

    let start_time = Instant::now();

    for embedding in &embeddings {
        match encode(embedding, &profile) {
            Ok(elid) => {
                elids.push(elid);
            }
            Err(e) => {
                errors += 1;
                eprintln!("Warning: Failed to encode embedding: {}", e);
            }
        }
        progress.increment();
    }

    progress.finish();

    if errors > 0 {
        println!("Warning: {} embeddings failed to encode\n", errors);
    }

    // ========================================================================
    // 4. Memory Usage Analysis
    // ========================================================================
    println!("4. Memory usage analysis:");

    let embedding_size = embeddings.len() * dimensions * std::mem::size_of::<f32>();
    let elid_size = elids.iter().map(|e| e.as_str().len()).sum::<usize>();

    println!(
        "  Original embeddings: {:.2} MB",
        embedding_size as f64 / 1_048_576.0
    );
    println!("  Encoded ELIDs:       {:.2} KB", elid_size as f64 / 1024.0);
    println!(
        "  Compression ratio:   {:.1}x",
        embedding_size as f64 / elid_size as f64
    );
    println!();

    // ========================================================================
    // 5. Quality Check: Similarity Preservation
    // ========================================================================
    println!("5. Quality check: Verifying similarity preservation...");

    // Check that nearby embeddings have low Hamming distance
    let sample_indices = [0, 1, 2, 100, 101, 102];
    let mut similarity_preserved = true;

    println!("Checking sample pairs:");
    for i in 0..sample_indices.len() - 1 {
        let idx1 = sample_indices[i];
        let idx2 = sample_indices[i + 1];

        if idx1 >= elids.len() || idx2 >= elids.len() {
            continue;
        }

        let distance = hamming_distance(&elids[idx1], &elids[idx2])?;

        // Adjacent embeddings should have relatively low distance
        let expected_low = (idx2 - idx1) <= 2;
        let is_low = distance < 64;

        println!(
            "  Pair ({}, {}): distance = {}/128 {}",
            idx1,
            idx2,
            distance,
            if expected_low == is_low {
                "✓"
            } else {
                "✗ unexpected"
            }
        );

        if expected_low != is_low {
            similarity_preserved = false;
        }
    }

    if similarity_preserved {
        println!("✓ Similarity preservation verified\n");
    } else {
        println!("Note: Some distances unexpected (expected for synthetic data)\n");
    }

    // ========================================================================
    // 6. Batch Encoding with Different Profiles
    // ========================================================================
    println!("6. Comparing encoding profiles...");

    let profiles = vec![
        ("Mini128", Profile::default()),
        (
            "Morton10x10",
            Profile::Morton10x10 {
                dims: 10,
                bits_per_dim: 10,
                transform_id: None,
            },
        ),
        (
            "Hilbert10x10",
            Profile::Hilbert10x10 {
                dims: 10,
                bits_per_dim: 10,
                transform_id: None,
            },
        ),
    ];

    // Use smaller sample for profile comparison
    let sample_size = 1000;
    let sample_embeddings = &embeddings[..sample_size];

    println!("Encoding {} embeddings with each profile...\n", sample_size);

    for (name, profile) in profiles {
        let start = Instant::now();
        let mut count = 0;

        for embedding in sample_embeddings {
            if encode(embedding, &profile).is_ok() {
                count += 1;
            }
        }

        let elapsed = start.elapsed();
        let rate = count as f64 / elapsed.as_secs_f64();

        println!(
            "  {:12} {:.2}s ({:.0} embeddings/sec)",
            name,
            elapsed.as_secs_f64(),
            rate
        );
    }
    println!();

    // ========================================================================
    // 7. Efficient Batch Pattern: Pre-allocated Vector
    // ========================================================================
    println!("7. Demonstrating efficient batch pattern...");

    let batch_start = Instant::now();

    // Pre-allocate output vector
    let mut batch_elids = Vec::with_capacity(sample_size);

    // Process batch
    for embedding in sample_embeddings {
        if let Ok(elid) = encode(embedding, &Profile::default()) {
            batch_elids.push(elid);
        }
    }

    let batch_elapsed = batch_start.elapsed();

    println!(
        "Encoded {} embeddings using pre-allocated vector",
        batch_elids.len()
    );
    println!("Time: {:.2}s", batch_elapsed.as_secs_f64());
    println!("Pattern: Vec::with_capacity() + push in loop");
    println!("Benefit: Avoids reallocation during growth\n");

    // ========================================================================
    // 8. Error Handling Pattern
    // ========================================================================
    println!("8. Error handling for batch operations...");

    // Create some invalid embeddings
    let test_batch = [
        vec![0.1; 768],           // Valid
        vec![f32::NAN; 768],      // Invalid: contains NaN
        vec![0.2; 768],           // Valid
        vec![f32::INFINITY; 768], // Invalid: contains Inf
        vec![0.3; 32],            // Invalid: too few dimensions
    ];

    let mut successful = Vec::new();
    let mut failed = Vec::new();

    for (idx, embedding) in test_batch.iter().enumerate() {
        match encode(embedding, &Profile::default()) {
            Ok(elid) => {
                successful.push((idx, elid));
            }
            Err(e) => {
                failed.push((idx, e.to_string()));
            }
        }
    }

    println!("Processed {} embeddings:", test_batch.len());
    println!("  Successful: {}", successful.len());
    println!("  Failed:     {}", failed.len());

    if !failed.is_empty() {
        println!("\nFailure details:");
        for (idx, error) in failed {
            println!("  Embedding {}: {}", idx, error);
        }
    }
    println!();

    // ========================================================================
    // 9. Best Practices Summary
    // ========================================================================
    println!("=== Best Practices for Batch Encoding ===");
    println!();
    println!("1. Pre-allocate vectors:");
    println!("   let mut elids = Vec::with_capacity(batch_size);");
    println!();
    println!("2. Track progress for large batches:");
    println!("   Report every 10% or 1000 items");
    println!();
    println!("3. Handle errors gracefully:");
    println!("   Continue processing on failure, collect errors");
    println!();
    println!("4. Choose appropriate profile:");
    println!("   - Mini128:      Similarity search (default)");
    println!("   - Morton10x10:  Fast database indexing");
    println!("   - Hilbert10x10: Best locality (slower)");
    println!();
    println!("5. Measure performance:");
    println!("   Track encoding rate (embeddings/sec)");
    println!();
    println!("6. Optional optimization (not in MVP):");
    println!("   For very large batches (>100k), consider parallel processing with Rayon");
    println!();

    // ========================================================================
    // Summary
    // ========================================================================
    println!("=== Summary ===");
    println!("Successfully processed {} embeddings", batch_size);
    println!(
        "Average encoding time: {:.2} μs/embedding",
        start_time.elapsed().as_micros() as f64 / batch_size as f64
    );
    println!("\nFor more examples, see:");
    println!("  - basic_usage.rs: Core ELID functionality");
    println!("  - hamming_neighbors.rs: Similarity search");

    Ok(())
}
