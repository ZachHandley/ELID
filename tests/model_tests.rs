//! Integration tests for embedding models (text + image).
//!
//! These tests verify end-to-end embedding pipelines including:
//! - Model loading from local files
//! - Tokenization and ONNX inference
//! - Output dimensions and normalization
//! - Similarity properties (similar inputs → similar embeddings)
//! - Integration with ELID encoding pipeline
//!
//! Run with: `cargo test --test model_tests --features "models-text,models-image"`
//! Models must be downloaded first: `uv run --script scripts/download_models.py`

// ============================================================================
// Text Embedding Tests (Model2Vec potion-base-8M)
// ============================================================================

#[cfg(feature = "models-text")]
mod text {
    use elid::models::{embed_text, embed_text_from_bytes, TEXT_EMBEDDING_DIM};

    #[test]
    fn text_embedding_correct_dimensions() {
        let emb = embed_text("Hello, world!").unwrap();
        assert_eq!(emb.len(), TEXT_EMBEDDING_DIM);
        assert_eq!(emb.len(), 256);
    }

    #[test]
    fn text_embedding_is_l2_normalized() {
        let emb = embed_text("The quick brown fox jumps over the lazy dog").unwrap();
        let norm: f32 = emb.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!(
            (norm - 1.0).abs() < 0.01,
            "Expected L2 norm ~1.0, got {norm}"
        );
    }

    #[test]
    fn text_embedding_deterministic() {
        let emb1 = embed_text("determinism test").unwrap();
        let emb2 = embed_text("determinism test").unwrap();
        assert_eq!(emb1, emb2, "Same input should produce identical embeddings");
    }

    #[test]
    fn text_similar_inputs_produce_similar_embeddings() {
        let e1 = embed_text("The quick brown fox").unwrap();
        let e2 = embed_text("The fast brown fox").unwrap();
        let cosine: f32 = e1.iter().zip(e2.iter()).map(|(a, b)| a * b).sum();
        assert!(
            cosine > 0.7,
            "Similar texts should have high cosine similarity, got {cosine}"
        );
    }

    #[test]
    fn text_different_inputs_produce_different_embeddings() {
        let e1 = embed_text("Machine learning algorithms").unwrap();
        let e2 = embed_text("Italian pasta recipes").unwrap();
        let cosine: f32 = e1.iter().zip(e2.iter()).map(|(a, b)| a * b).sum();
        assert!(
            cosine < 0.7,
            "Unrelated texts should have low cosine similarity, got {cosine}"
        );
    }

    #[test]
    fn text_embedding_handles_unicode() {
        let emb = embed_text("café résumé naïve").unwrap();
        assert_eq!(emb.len(), TEXT_EMBEDDING_DIM);
    }

    #[test]
    fn text_embedding_handles_long_input() {
        let long_text = "word ".repeat(500);
        let emb = embed_text(&long_text).unwrap();
        assert_eq!(emb.len(), TEXT_EMBEDDING_DIM);
    }

    #[test]
    fn text_from_bytes_matches_file_loading() {
        let dir = std::path::Path::new("models");
        let safetensors = std::fs::read(dir.join("potion-base-8m.safetensors")).unwrap();
        let tokenizer = std::fs::read(dir.join("potion-base-8m-tokenizer.json")).unwrap();
        let config = std::fs::read(dir.join("potion-base-8m-config.json")).unwrap();

        let emb_file = embed_text("consistency check").unwrap();
        let emb_bytes =
            embed_text_from_bytes("consistency check", &safetensors, &tokenizer, &config).unwrap();

        assert_eq!(emb_file.len(), emb_bytes.len());
        for (a, b) in emb_file.iter().zip(emb_bytes.iter()) {
            assert!(
                (a - b).abs() < 1e-6,
                "File and bytes loading should produce identical results"
            );
        }
    }

    /// End-to-end: embed_text → encode to ELID → compare distances
    #[test]
    fn text_embedding_to_elid_pipeline() {
        use elid::embeddings::{encode, hamming_distance, Profile};

        let e1 = embed_text("The cat sat on the mat").unwrap();
        let e2 = embed_text("The cat sat on the rug").unwrap();
        let e3 = embed_text("Quantum computing advances").unwrap();

        let profile = Profile::default(); // Mini128
        let elid1 = encode(&e1, &profile).unwrap();
        let elid2 = encode(&e2, &profile).unwrap();
        let elid3 = encode(&e3, &profile).unwrap();

        let dist_similar = hamming_distance(&elid1, &elid2).unwrap();
        let dist_different = hamming_distance(&elid1, &elid3).unwrap();

        assert!(
            dist_similar < dist_different,
            "Similar texts should have smaller ELID hamming distance: similar={dist_similar}, different={dist_different}"
        );
    }
}

// ============================================================================
// Image Embedding Tests (MobileNetV3-Small ONNX via tract)
// ============================================================================

#[cfg(feature = "models-image")]
mod image {
    use elid::models::{embed_image, IMAGE_EMBEDDING_DIM};

    fn test_image() -> Vec<u8> {
        std::fs::read("tests/fixtures/test_image.jpg")
            .expect("Test image not found. Create one at tests/fixtures/test_image.jpg")
    }

    #[test]
    fn image_embedding_correct_dimensions() {
        let emb = embed_image(&test_image()).unwrap();
        assert_eq!(emb.len(), IMAGE_EMBEDDING_DIM);
        assert_eq!(emb.len(), 1000);
    }

    #[test]
    fn image_embedding_is_l2_normalized() {
        let emb = embed_image(&test_image()).unwrap();
        let norm: f32 = emb.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!(
            (norm - 1.0).abs() < 0.01,
            "Expected L2 norm ~1.0, got {norm}"
        );
    }

    #[test]
    fn image_embedding_deterministic() {
        let img = test_image();
        let emb1 = embed_image(&img).unwrap();
        let emb2 = embed_image(&img).unwrap();
        assert_eq!(emb1, emb2, "Same image should produce identical embeddings");
    }

    #[test]
    fn image_embedding_rejects_invalid_input() {
        let result = embed_image(&[0u8; 100]);
        assert!(result.is_err(), "Random bytes should fail as image input");
    }

    /// End-to-end: embed_image → encode to ELID
    #[test]
    fn image_embedding_to_elid_pipeline() {
        use elid::embeddings::{encode, Profile};

        let emb = embed_image(&test_image()).unwrap();
        let profile = Profile::default(); // Mini128
        let elid = encode(&emb, &profile).unwrap();

        // ELID should be a non-empty base32hex string
        let s = elid.as_str();
        assert!(!s.is_empty());
        assert!(
            s.chars().all(|c| c.is_ascii_alphanumeric()),
            "ELID should be alphanumeric, got: {s}"
        );
    }
}

// ============================================================================
// Cross-feature tests (text + image together)
// ============================================================================

#[cfg(all(feature = "models-text", feature = "models-image"))]
mod cross {
    use elid::embeddings::{encode, hamming_distance, Profile};
    use elid::models::{embed_image, embed_text};

    #[test]
    fn text_and_image_embeddings_have_different_dimensions() {
        let text_emb = embed_text("A photo of a cat").unwrap();
        let img = std::fs::read("tests/fixtures/test_image.jpg").unwrap();
        let img_emb = embed_image(&img).unwrap();

        assert_eq!(text_emb.len(), 256);
        assert_eq!(img_emb.len(), 1000);
    }

    #[test]
    fn text_and_image_elids_are_comparable_via_mini128() {
        let text_emb = embed_text("Hello world").unwrap();
        let img = std::fs::read("tests/fixtures/test_image.jpg").unwrap();
        let img_emb = embed_image(&img).unwrap();

        let profile = Profile::default(); // Mini128
        let text_elid = encode(&text_emb, &profile).unwrap();
        let img_elid = encode(&img_emb, &profile).unwrap();

        // Both should produce valid ELIDs of the same length (Mini128 → same bit count)
        assert_eq!(text_elid.as_str().len(), img_elid.as_str().len());

        // Should be able to compute hamming distance
        let dist = hamming_distance(&text_elid, &img_elid).unwrap();
        // Just verify it computes without error (dist is u32, always >= 0)
        let _ = dist;
    }
}
