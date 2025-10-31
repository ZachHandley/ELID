//! Performance smoke tests to verify FR-023 requirements
//!
//! FR-023 states: "10K embeddings (768-dim) in <30s"
//!
//! These tests verify that encoding performance meets the required thresholds.
//! They are marked with `#[ignore]` because they are slow tests.
//!
//! Run with: `cargo test --release -- --ignored`

use elid_core::{encode, Profile};

/// Generate a random f32 embedding vector
fn random_embedding(dims: usize) -> Vec<f32> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..dims).map(|_| rng.gen::<f32>()).collect()
}

#[test]
#[ignore]
fn test_encoding_performance_mini128() {
    use std::time::Instant;

    // Create 10,000 random 768-dim embeddings
    let embeddings: Vec<Vec<f32>> = (0..10_000).map(|_| random_embedding(768)).collect();

    let profile = Profile::default();
    let start = Instant::now();

    for emb in &embeddings {
        let _ = encode(emb, &profile).expect("Encoding failed");
    }

    let duration = start.elapsed();
    println!("Encoded 10,000 embeddings (Mini128) in {:?}", duration);

    // Verify FR-023: Must complete in <30 seconds
    assert!(
        duration.as_secs() < 30,
        "Performance requirement failed: took {:?}, expected <30s",
        duration
    );
}

#[test]
#[ignore]
fn test_encoding_performance_morton() {
    use std::time::Instant;

    // Create 10,000 random 768-dim embeddings (reduced to 100 dims for Morton10x10)
    let embeddings: Vec<Vec<f32>> = (0..10_000).map(|_| random_embedding(100)).collect();

    let profile = Profile::Morton10x10 {
        dims: 10,
        bits_per_dim: 10,
        transform_id: None,
    };
    let start = Instant::now();

    for emb in &embeddings {
        let _ = encode(emb, &profile).expect("Encoding failed");
    }

    let duration = start.elapsed();
    println!("Encoded 10,000 embeddings (Morton10x10) in {:?}", duration);

    // Verify FR-023: Must complete in <30 seconds
    assert!(
        duration.as_secs() < 30,
        "Performance requirement failed: took {:?}, expected <30s",
        duration
    );
}

#[test]
#[ignore]
fn test_encoding_performance_hilbert() {
    use std::time::Instant;

    // Create 10,000 random 768-dim embeddings (reduced to 100 dims for Hilbert10x10)
    let embeddings: Vec<Vec<f32>> = (0..10_000).map(|_| random_embedding(100)).collect();

    let profile = Profile::Hilbert10x10 {
        dims: 10,
        bits_per_dim: 10,
        transform_id: None,
    };
    let start = Instant::now();

    for emb in &embeddings {
        let _ = encode(emb, &profile).expect("Encoding failed");
    }

    let duration = start.elapsed();
    println!("Encoded 10,000 embeddings (Hilbert10x10) in {:?}", duration);

    // Verify FR-023: Must complete in <30 seconds
    assert!(
        duration.as_secs() < 30,
        "Performance requirement failed: took {:?}, expected <30s",
        duration
    );
}

#[test]
#[ignore]
fn test_encoding_performance_full_768dim() {
    use std::time::Instant;

    // Create 10,000 random 768-dim embeddings
    // Use a profile that can handle full dimensionality
    let embeddings: Vec<Vec<f32>> = (0..10_000).map(|_| random_embedding(768)).collect();

    let profile = Profile::Mini128 { seed: 42 };
    let start = Instant::now();

    for emb in &embeddings {
        let _ = encode(emb, &profile).expect("Encoding failed");
    }

    let duration = start.elapsed();
    println!("Encoded 10,000 embeddings (full 768-dim) in {:?}", duration);

    // Verify FR-023: Must complete in <30 seconds
    assert!(
        duration.as_secs() < 30,
        "Performance requirement failed: took {:?}, expected <30s",
        duration
    );
}

#[test]
fn test_encoding_single_performance_baseline() {
    use std::time::Instant;

    // Quick sanity check: single encoding should be very fast
    let embedding = random_embedding(768);
    let profile = Profile::default();

    let start = Instant::now();
    let _ = encode(&embedding, &profile).expect("Encoding failed");
    let duration = start.elapsed();

    println!("Single embedding encode time: {:?}", duration);

    // Should be well under 10ms in release mode, but much slower in debug
    // This is a smoke test, not a strict performance requirement
    #[cfg(debug_assertions)]
    assert!(
        duration.as_millis() < 100,
        "Single encoding too slow (debug): {:?}",
        duration
    );

    #[cfg(not(debug_assertions))]
    assert!(
        duration.as_millis() < 10,
        "Single encoding too slow (release): {:?}",
        duration
    );
}

#[test]
fn test_encoding_batch_100_performance() {
    use std::time::Instant;

    // Test 100 embeddings as a quick smoke test
    let embeddings: Vec<Vec<f32>> = (0..100).map(|_| random_embedding(768)).collect();

    let profile = Profile::default();
    let start = Instant::now();

    for emb in &embeddings {
        let _ = encode(emb, &profile).expect("Encoding failed");
    }

    let duration = start.elapsed();
    println!("Encoded 100 embeddings in {:?}", duration);

    // Should complete in under 1 second in release, under 10s in debug
    #[cfg(debug_assertions)]
    assert!(
        duration.as_secs() < 10,
        "Batch encoding too slow (debug): {:?}",
        duration
    );

    #[cfg(not(debug_assertions))]
    assert!(
        duration.as_millis() < 1000,
        "Batch encoding too slow (release): {:?}",
        duration
    );
}
