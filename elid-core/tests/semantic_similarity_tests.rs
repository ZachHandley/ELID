//! Semantic Similarity Tests
//!
//! Tests demonstrating how ELIDs preserve semantic similarity using synthetic embeddings
//! that simulate real-world text embedding scenarios.

use elid_core::{encode, hamming_distance, Profile};

/// Helper to create synthetic embeddings that simulate semantic similarity
/// Creates a vector that differs from base by rotating it in embedding space
fn create_synthetic_embedding(base: Vec<f32>, perturbation_factor: f32) -> Vec<f32> {
    // Create a different vector to mix with base
    let noise_vec: Vec<f32> = (0..base.len())
        .map(|i| (i as f32 * 1.7).cos() * 0.3 + (i as f32 * 2.3).sin() * 0.7)
        .collect();

    // Mix base with noise vector based on perturbation factor
    // This creates a rotation in embedding space
    base.iter()
        .zip(noise_vec.iter())
        .map(|(&b, &n)| b * (1.0 - perturbation_factor) + n * perturbation_factor)
        .collect()
}

#[test]
fn test_identical_strings_produce_identical_elids() {
    // Simulate encoding the same string twice (identical embeddings)
    let embedding1: Vec<f32> = (0..384).map(|i| (i as f32).sin()).collect();
    let embedding2 = embedding1.clone();

    let elid1 = encode(&embedding1, &Profile::default()).unwrap();
    let elid2 = encode(&embedding2, &Profile::default()).unwrap();

    assert_eq!(
        elid1, elid2,
        "Identical embeddings must produce identical ELIDs"
    );
    assert_eq!(
        hamming_distance(&elid1, &elid2).unwrap(),
        0,
        "Hamming distance between identical ELIDs must be 0"
    );
}

#[test]
fn test_very_similar_strings_have_low_hamming_distance() {
    // Simulate "The cat sat on the mat" vs "The cat sat on the rug"
    // (nearly identical semantic meaning with one word different)
    let base_embedding: Vec<f32> = (0..384).map(|i| (i as f32 * 0.1).sin()).collect();

    // Add tiny perturbation (1%) to simulate one word change
    let similar_embedding = create_synthetic_embedding(base_embedding.clone(), 0.01);

    let elid1 = encode(&base_embedding, &Profile::default()).unwrap();
    let elid2 = encode(&similar_embedding, &Profile::default()).unwrap();

    let hamming_dist = hamming_distance(&elid1, &elid2).unwrap();

    // Very similar embeddings should have low Hamming distance
    assert!(
        hamming_dist < 40,
        "Very similar strings should have Hamming distance < 40, got {}",
        hamming_dist
    );
}

#[test]
fn test_moderately_similar_strings_have_moderate_hamming_distance() {
    // Simulate "I love programming" vs "I enjoy coding"
    // (similar topic, different wording)
    let base_embedding: Vec<f32> = (0..384).map(|i| (i as f32 * 0.1).cos()).collect();

    // Add moderate perturbation (25%) to simulate topic similarity with different wording
    let similar_embedding = create_synthetic_embedding(base_embedding.clone(), 0.25);

    let elid1 = encode(&base_embedding, &Profile::default()).unwrap();
    let elid2 = encode(&similar_embedding, &Profile::default()).unwrap();

    let hamming_dist = hamming_distance(&elid1, &elid2).unwrap();

    // Moderately similar should have moderate Hamming distance
    // Note: SimHash naturally has high locality preservation, so "moderate" is 10-50
    assert!(
        (10..=50).contains(&hamming_dist),
        "Moderately similar strings should have Hamming distance 10-50, got {}",
        hamming_dist
    );
}

#[test]
fn test_dissimilar_strings_have_high_hamming_distance() {
    // Simulate "The cat sat on the mat" vs "Quantum physics is fascinating"
    // (completely unrelated topics)
    let embedding1: Vec<f32> = (0..384).map(|i| (i as f32 * 0.1).sin()).collect();
    let embedding2: Vec<f32> = (0..384).map(|i| (i as f32 * 0.3).cos()).collect();

    let elid1 = encode(&embedding1, &Profile::default()).unwrap();
    let elid2 = encode(&embedding2, &Profile::default()).unwrap();

    let hamming_dist = hamming_distance(&elid1, &elid2).unwrap();

    // Dissimilar embeddings should have higher Hamming distance
    // Note: SimHash naturally clusters around 64 for random vectors (birthday paradox)
    // but dissimilar should still be >35
    assert!(
        hamming_dist > 35,
        "Dissimilar strings should have Hamming distance > 35, got {}",
        hamming_dist
    );
}

#[test]
fn test_hamming_distance_correlates_with_similarity_gradient() {
    // Test a gradient of similarity levels
    let base_embedding: Vec<f32> = (0..384).map(|i| (i as f32 * 0.1).sin()).collect();

    let perturbations = [0.01, 0.05, 0.1, 0.2, 0.4];
    let mut hamming_distances = Vec::new();

    let base_elid = encode(&base_embedding, &Profile::default()).unwrap();

    for perturbation in &perturbations {
        let perturbed = create_synthetic_embedding(base_embedding.clone(), *perturbation);
        let elid = encode(&perturbed, &Profile::default()).unwrap();
        let dist = hamming_distance(&base_elid, &elid).unwrap();
        hamming_distances.push(dist);
    }

    // Verify Hamming distances generally increase with perturbation
    // (allowing for some noise in the middle ranges due to hash collisions)
    assert!(
        hamming_distances[0] < hamming_distances[3],
        "Smaller perturbations should produce smaller Hamming distances: {:?}",
        hamming_distances
    );
    assert!(
        hamming_distances[1] < hamming_distances[4],
        "Gradient should show increasing distance trend: {:?}",
        hamming_distances
    );
}

#[test]
fn test_hamming_distance_symmetric_for_similar_strings() {
    // Verify symmetry property holds for semantic similarity
    let embedding1: Vec<f32> = (0..384).map(|i| (i as f32).sin()).collect();
    let embedding2 = create_synthetic_embedding(embedding1.clone(), 0.05);

    let elid1 = encode(&embedding1, &Profile::default()).unwrap();
    let elid2 = encode(&embedding2, &Profile::default()).unwrap();

    let dist_1_2 = hamming_distance(&elid1, &elid2).unwrap();
    let dist_2_1 = hamming_distance(&elid2, &elid1).unwrap();

    assert_eq!(
        dist_1_2, dist_2_1,
        "Hamming distance must be symmetric: distance(A,B) = distance(B,A)"
    );
}

#[test]
fn test_normalized_vs_unnormalized_similarity_preserved() {
    // Verify that normalization preserves semantic similarity
    // (scaling a vector doesn't change its direction/meaning)
    let embedding1: Vec<f32> = (0..384).map(|i| (i as f32).sin()).collect();
    let embedding2: Vec<f32> = embedding1.iter().map(|&x| x * 2.0).collect(); // Scaled 2x

    let elid1 = encode(&embedding1, &Profile::default()).unwrap();
    let elid2 = encode(&embedding2, &Profile::default()).unwrap();

    // Scaled versions of same vector should produce identical ELIDs
    // (encode function normalizes internally)
    assert_eq!(
        elid1, elid2,
        "Normalized and unnormalized versions of same vector should produce identical ELIDs"
    );
}

#[test]
fn test_hamming_distance_range_for_semantic_similarity() {
    // Statistical test: verify Hamming distance range properties
    let base_embedding: Vec<f32> = (0..384).map(|i| (i as f32 * 0.1).sin()).collect();

    // Test 10 similar embeddings
    let mut similar_distances = Vec::new();
    for i in 0..10 {
        let similar = create_synthetic_embedding(base_embedding.clone(), 0.01 * (i as f32 + 1.0));
        let elid = encode(&similar, &Profile::default()).unwrap();
        let base_elid = encode(&base_embedding, &Profile::default()).unwrap();
        similar_distances.push(hamming_distance(&base_elid, &elid).unwrap());
    }

    // All similar embeddings should cluster in low Hamming distance range
    let max_similar = *similar_distances.iter().max().unwrap();
    assert!(
        max_similar < 50,
        "Similar embeddings should have Hamming distance < 50, max was {}",
        max_similar
    );

    // Test against dissimilar embedding
    let dissimilar: Vec<f32> = (0..384).map(|i| (i as f32 * 0.3).cos()).collect();
    let dissimilar_elid = encode(&dissimilar, &Profile::default()).unwrap();
    let base_elid = encode(&base_embedding, &Profile::default()).unwrap();
    let dissimilar_distance = hamming_distance(&base_elid, &dissimilar_elid).unwrap();

    // Dissimilar should be further than all similar ones
    assert!(
        dissimilar_distance > max_similar,
        "Dissimilar embedding distance ({}) should be greater than max similar distance ({})",
        dissimilar_distance,
        max_similar
    );
}

#[test]
fn test_prefix_sharing_for_similar_strings() {
    // Similar strings should share ELID prefixes due to sortability
    let base_embedding: Vec<f32> = (0..384).map(|i| (i as f32 * 0.1).sin()).collect();
    let similar = create_synthetic_embedding(base_embedding.clone(), 0.01);

    let elid1 = encode(&base_embedding, &Profile::default()).unwrap();
    let elid2 = encode(&similar, &Profile::default()).unwrap();

    // Count shared prefix characters
    let shared_prefix = elid1
        .as_str()
        .chars()
        .zip(elid2.as_str().chars())
        .take_while(|(a, b)| a == b)
        .count();

    // Similar embeddings should share at least some prefix
    // (though not guaranteed due to hash nature, should happen often)
    assert!(
        shared_prefix >= 1,
        "Similar embeddings should typically share at least 1 prefix character, shared: {}",
        shared_prefix
    );
}
