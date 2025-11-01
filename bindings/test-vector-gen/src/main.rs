#!/usr/bin/env rust-script
//! Generate reference test vectors for cross-language validation
//!
//! This program generates a set of test embeddings and their corresponding
//! ELID encodings for all three profiles. Output is in JSON format for
//! consumption by language binding tests.

use elid_core::{encode, Profile};
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
struct TestVector {
    #[serde(rename = "embedding")]
    embedding: Vec<f32>,
    #[serde(rename = "profile")]
    profile: String,
    #[serde(rename = "elid")]
    elid: String,
    #[serde(rename = "dimensions")]
    dimensions: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct TestVectors {
    #[serde(rename = "version")]
    version: String,
    #[serde(rename = "vectors")]
    vectors: Vec<TestVector>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut vectors = Vec::new();

    // Test vector 1: 384D (common for small models)
    let embedding_384: Vec<f32> = (0..384).map(|i| (i as f32) / 384.0).collect();

    // Test vector 2: 768D (BERT, sentence-transformers)
    let embedding_768: Vec<f32> = (0..768).map(|i| (i as f32) / 768.0).collect();

    // Test vector 3: 1536D (OpenAI text-embedding-3-small)
    let embedding_1536: Vec<f32> = (0..1536).map(|i| (i as f32) / 1536.0).collect();

    // Test vector 4: All same values (edge case)
    let embedding_same: Vec<f32> = vec![0.5; 768];

    // Test vector 5: Alternating values
    let embedding_alt: Vec<f32> = (0..768).map(|i| if i % 2 == 0 { 0.1 } else { 0.9 }).collect();

    let test_embeddings = vec![
        (embedding_384, "384D uniform"),
        (embedding_768, "768D uniform"),
        (embedding_1536, "1536D uniform"),
        (embedding_same, "768D same values"),
        (embedding_alt, "768D alternating"),
    ];

    // Generate vectors for all profiles
    let profiles = vec![
        (Profile::Mini128 { seed: 0 }, "Mini128"),
        (
            Profile::Morton10x10 {
                dims: 10,
                bits_per_dim: 10,
                transform_id: None,
            },
            "Morton10x10",
        ),
        (
            Profile::Hilbert10x10 {
                dims: 10,
                bits_per_dim: 10,
                transform_id: None,
            },
            "Hilbert10x10",
        ),
    ];

    for (embedding, desc) in test_embeddings {
        for (profile, profile_name) in &profiles {
            let elid = encode(&embedding, profile)?;
            vectors.push(TestVector {
                embedding: embedding.clone(),
                profile: format!("{} ({})", profile_name, desc),
                elid: elid.as_str().to_string(),
                dimensions: embedding.len(),
            });
        }
    }

    let output = TestVectors {
        version: "1.0.0".to_string(),
        vectors,
    };

    // Output as pretty JSON
    println!("{}", serde_json::to_string_pretty(&output)?);

    Ok(())
}
