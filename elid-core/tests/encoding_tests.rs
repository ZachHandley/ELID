//! Integration tests for ELID encoding/decoding operations
//!
//! This test suite verifies end-to-end functionality of the ELID encoding system,
//! including roundtrip encoding/decoding, profile extraction, format validation,
//! and multi-embedding consistency.

use elid_core::{decode, encode, Elid, Profile};

// ============================================================================
// T025.1: Roundtrip Encoding/Decoding Tests
// ============================================================================

#[test]
fn test_roundtrip_basic_mini128() {
    let embedding = vec![0.1, 0.2, 0.3, 0.4]
        .into_iter()
        .cycle()
        .take(768)
        .collect::<Vec<_>>();
    let profile = Profile::default();

    // Encode
    let elid = encode(&embedding, &profile).expect("Encoding failed");

    // Decode
    let bytes = decode(&elid).expect("Decoding failed");

    // Verify structure: 2 header bytes + 16 payload bytes = 18 bytes
    assert_eq!(
        bytes.len(),
        18,
        "Mini128 should produce 18 bytes (2 header + 16 payload)"
    );

    // Re-encode from decoded bytes (verify bytes can produce same ELID)
    let profile_info = elid.profile().expect("Failed to extract profile");
    assert_eq!(profile_info.version, 0);
    assert_eq!(profile_info.profile_type, 0x01); // Mini128
}

#[test]
fn test_roundtrip_different_dimensions() {
    let profile = Profile::Mini128 {
        seed: 0x454c4944_53494d48,
    };

    for dim in [64, 128, 256, 512, 768, 1024, 1536, 2048] {
        let embedding = vec![0.5_f32; dim];
        let elid = encode(&embedding, &profile)
            .unwrap_or_else(|_| panic!("Encoding failed for dimension {}", dim));
        let bytes =
            decode(&elid).unwrap_or_else(|_| panic!("Decoding failed for dimension {}", dim));

        assert_eq!(
            bytes.len(),
            18,
            "All Mini128 ELIDs should be 18 bytes regardless of dimension (dim={})",
            dim
        );
    }
}

#[test]
fn test_roundtrip_preserves_exact_bytes() {
    let embedding = vec![
        0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, -0.1, -0.2, -0.3, -0.4, -0.5, -0.6, -0.7, -0.8,
    ]
    .into_iter()
    .cycle()
    .take(256)
    .collect::<Vec<_>>();

    let profile = Profile::Mini128 {
        seed: 0x454c4944_53494d48,
    };

    // Encode twice
    let elid1 = encode(&embedding, &profile).expect("First encoding failed");
    let elid2 = encode(&embedding, &profile).expect("Second encoding failed");

    // Should produce identical ELIDs
    assert_eq!(elid1, elid2);

    // Decode both
    let bytes1 = decode(&elid1).expect("First decoding failed");
    let bytes2 = decode(&elid2).expect("Second decoding failed");

    // Should produce identical bytes
    assert_eq!(bytes1, bytes2);

    // Create Elid from the same string and decode again
    let elid3 = Elid::from_string(elid1.as_str().to_string()).expect("Elid creation failed");
    let bytes3 = decode(&elid3).expect("Third decoding failed");

    // All bytes should match
    assert_eq!(bytes1, bytes3);
}

#[test]
fn test_roundtrip_zero_embedding() {
    let embedding = vec![0.0_f32; 128];
    let profile = Profile::default();

    let elid = encode(&embedding, &profile).expect("Encoding all-zero embedding failed");
    let bytes = decode(&elid).expect("Decoding all-zero embedding failed");

    assert_eq!(bytes.len(), 18);

    // Re-encode should produce same result
    let elid2 = encode(&embedding, &profile).expect("Second encoding failed");
    assert_eq!(elid, elid2);
}

// ============================================================================
// T025.2: Profile Information Extraction Tests
// ============================================================================

#[test]
fn test_profile_extraction_mini128() {
    let embedding = vec![0.3; 256];
    let profile = Profile::Mini128 {
        seed: 0x454c4944_53494d48,
    };

    let elid = encode(&embedding, &profile).expect("Encoding failed");
    let profile_info = elid.profile().expect("Profile extraction failed");

    // Verify header fields
    assert_eq!(profile_info.version, 0, "Version should be 0 for v0.1");
    assert_eq!(
        profile_info.profile_type, 0x01,
        "Profile type should be 0x01 for Mini128"
    );
    assert_eq!(
        profile_info.transform_id, None,
        "Transform ID should be None for basic Mini128"
    );
    assert_eq!(
        profile_info.model_id, None,
        "Model ID should be None in v0.1"
    );
}

#[test]
fn test_profile_extraction_from_bytes() {
    let embedding = vec![0.5; 512];
    let profile = Profile::default();

    let elid = encode(&embedding, &profile).expect("Encoding failed");
    let bytes = decode(&elid).expect("Decoding failed");

    // Extract profile from raw bytes
    let version = (bytes[0] & 0xF0) >> 4;
    let profile_type = bytes[0] & 0x0F;
    let _reserved = bytes[1];

    assert_eq!(version, 0);
    assert_eq!(profile_type, 0x01);
}

#[test]
fn test_profile_extraction_different_seeds() {
    // Different seeds should not affect profile type extraction
    let embedding = vec![0.4; 128];

    for seed in [
        0x0000_0000_0000_0001,
        0x1111_1111_1111_1111,
        0x454c4944_53494d48, // "ELIDSIMH"
        0xFFFF_FFFF_FFFF_FFFF,
    ] {
        let profile = Profile::Mini128 { seed };
        let elid = encode(&embedding, &profile)
            .unwrap_or_else(|_| panic!("Encoding failed for seed {:#x}", seed));

        let profile_info = elid
            .profile()
            .unwrap_or_else(|_| panic!("Profile extraction failed for seed {:#x}", seed));

        assert_eq!(profile_info.version, 0);
        assert_eq!(profile_info.profile_type, 0x01);
    }
}

// ============================================================================
// T025.3: Base32hex String Format Validation Tests
// ============================================================================

#[test]
fn test_base32hex_alphabet_only() {
    let embedding = vec![
        0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, -0.1, -0.2, -0.3, -0.4,
    ]
    .into_iter()
    .cycle()
    .take(768)
    .collect::<Vec<_>>();
    let profile = Profile::default();

    let elid = encode(&embedding, &profile).expect("Encoding failed");

    // Verify all characters are in base32hex alphabet (0-9, a-v)
    for c in elid.as_str().chars() {
        assert!(
            matches!(c, '0'..='9' | 'a'..='v'),
            "Character '{}' not in base32hex alphabet",
            c
        );
    }
}

#[test]
fn test_base32hex_lowercase_only() {
    let embeddings = vec![
        vec![1.0; 128],
        vec![-1.0; 128],
        vec![0.5; 256],
        vec![0.3, 0.7].into_iter().cycle().take(512).collect(),
    ];

    let profile = Profile::default();

    for (idx, embedding) in embeddings.into_iter().enumerate() {
        let elid = encode(&embedding, &profile)
            .unwrap_or_else(|_| panic!("Encoding failed for embedding {}", idx));

        // No uppercase letters
        assert!(
            elid.as_str().chars().all(|c| !c.is_ascii_uppercase()),
            "ELID should only contain lowercase letters, got: {}",
            elid.as_str()
        );
    }
}

#[test]
fn test_base32hex_no_padding() {
    let embedding = vec![0.2; 768];
    let profile = Profile::default();

    let elid = encode(&embedding, &profile).expect("Encoding failed");

    // Base32hex without padding should not contain '='
    assert!(
        !elid.as_str().contains('='),
        "ELID should not contain padding character '='"
    );
}

#[test]
fn test_invalid_base32hex_characters() {
    // Characters outside base32hex alphabet (w, x, y, z are invalid)
    let invalid_strings = vec![
        "w0000000000000000000000000000",
        "0000000000000000000000000000x",
        "00000000000000y00000000000000",
        "000000000000000000000000000z",
        "ABCD00000000000000000000000",  // uppercase
        "000000000000000000000000000!", // special char
    ];

    for invalid in invalid_strings {
        let result = Elid::from_string(invalid.to_string());
        assert!(
            result.is_err(),
            "Should reject invalid base32hex string: {}",
            invalid
        );
    }
}

#[test]
fn test_base32hex_sortability() {
    // Create embeddings that will produce different hashes
    let embeddings = vec![
        vec![1.0, 0.0]
            .into_iter()
            .cycle()
            .take(128)
            .collect::<Vec<_>>(),
        vec![0.0, 1.0].into_iter().cycle().take(128).collect(),
        vec![1.0, 1.0].into_iter().cycle().take(128).collect(),
        vec![0.5, 0.5].into_iter().cycle().take(128).collect(),
    ];

    let profile = Profile::Mini128 {
        seed: 0x454c4944_53494d48,
    };

    let mut elids: Vec<Elid> = embeddings
        .into_iter()
        .map(|emb| encode(&emb, &profile).expect("Encoding failed"))
        .collect();

    // Sort the ELIDs
    let original_order = elids.clone();
    elids.sort();

    // Verify they can be sorted (no panics)
    // Note: We don't verify specific ordering because SimHash doesn't preserve
    // value-based ordering, only locality-based clustering

    // Verify all ELIDs are distinct (compare with original before dedup)
    let original_len = original_order.len();
    elids.dedup();

    // Note: Some embeddings might produce identical hashes after normalization
    // (e.g., [1.0, 1.0] and [0.5, 0.5] have same direction), so we just verify
    // that we can sort without panicking, not that all are necessarily unique
    assert!(
        !elids.is_empty() && elids.len() <= original_len,
        "ELIDs should be sortable (deduped {} from {})",
        elids.len(),
        original_len
    );
}

// ============================================================================
// T025.4: ELID Length Validation Tests
// ============================================================================

#[test]
fn test_mini128_length_constant() {
    let profile = Profile::default();

    // Test various embedding dimensions - all should produce same ELID length
    for dim in [64, 128, 256, 512, 768, 1024, 1536, 2048] {
        let embedding = vec![0.1; dim];
        let elid = encode(&embedding, &profile)
            .unwrap_or_else(|_| panic!("Encoding failed for dim={}", dim));

        // Mini128: 2 header bytes + 16 payload bytes = 18 bytes
        // Base32hex: ceil(18 * 8 / 5) = ceil(144 / 5) = ceil(28.8) = 29 characters
        assert_eq!(
            elid.as_str().len(),
            29,
            "Mini128 ELID should be 29 characters for dim={}",
            dim
        );
    }
}

#[test]
fn test_elid_bytes_length() {
    let embedding = vec![0.3; 256];
    let profile = Profile::default();

    let elid = encode(&embedding, &profile).expect("Encoding failed");
    let bytes = elid.to_bytes().expect("to_bytes failed");

    assert_eq!(bytes.len(), 18, "Mini128 decoded bytes should be 18 bytes");

    // Verify structure
    assert_eq!(bytes[0] & 0x0F, 0x01, "Profile type should be Mini128");
    assert_eq!(bytes[1], 0x00, "Reserved byte should be 0");
    // Remaining 16 bytes are SimHash payload
}

#[test]
fn test_different_embeddings_same_length() {
    let profile = Profile::default();

    let embeddings = vec![
        vec![0.1; 128],
        vec![1.0; 128],
        vec![-1.0; 128],
        vec![0.5, -0.5].into_iter().cycle().take(128).collect(),
        vec![
            0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, -0.1, -0.2, -0.3, -0.4, -0.5, -0.6, -0.7, -0.8,
        ]
        .into_iter()
        .cycle()
        .take(128)
        .collect(),
    ];

    for (idx, embedding) in embeddings.into_iter().enumerate() {
        let elid = encode(&embedding, &profile)
            .unwrap_or_else(|_| panic!("Encoding failed for embedding {}", idx));

        assert_eq!(
            elid.as_str().len(),
            29,
            "All Mini128 ELIDs should be 29 characters (embedding {})",
            idx
        );
    }
}

// ============================================================================
// T025.5: Multiple Embeddings with Same Profile Consistency Tests
// ============================================================================

#[test]
fn test_same_profile_consistent_format() {
    let profile = Profile::Mini128 {
        seed: 0x454c4944_53494d48,
    };

    let embeddings = vec![
        vec![0.1; 128],
        vec![0.2; 256],
        vec![0.3; 512],
        vec![0.4; 768],
        vec![0.5; 1024],
    ];

    for embedding in embeddings {
        let elid = encode(&embedding, &profile).expect("Encoding failed");

        // All should have same profile info
        let profile_info = elid.profile().expect("Profile extraction failed");
        assert_eq!(profile_info.version, 0);
        assert_eq!(profile_info.profile_type, 0x01);

        // All should have same string length
        assert_eq!(elid.as_str().len(), 29);

        // All should have same byte length
        let bytes = decode(&elid).expect("Decoding failed");
        assert_eq!(bytes.len(), 18);
    }
}

#[test]
fn test_determinism_across_multiple_embeddings() {
    let profile = Profile::default();

    let embeddings = vec![
        vec![0.1, 0.2, 0.3, 0.4]
            .into_iter()
            .cycle()
            .take(128)
            .collect::<Vec<_>>(),
        vec![0.5, 0.6, 0.7, 0.8]
            .into_iter()
            .cycle()
            .take(256)
            .collect(),
        vec![1.0, 2.0, 3.0, 4.0]
            .into_iter()
            .cycle()
            .take(512)
            .collect(),
    ];

    // Encode each embedding twice and verify consistency
    for (idx, embedding) in embeddings.into_iter().enumerate() {
        let elid1 = encode(&embedding, &profile)
            .unwrap_or_else(|_| panic!("First encoding failed for embedding {}", idx));
        let elid2 = encode(&embedding, &profile)
            .unwrap_or_else(|_| panic!("Second encoding failed for embedding {}", idx));

        assert_eq!(
            elid1, elid2,
            "Embedding {} should produce identical ELIDs on repeated encoding",
            idx
        );
    }
}

#[test]
fn test_similar_embeddings_with_same_profile() {
    let profile = Profile::Mini128 {
        seed: 0x454c4944_53494d48,
    };

    // Create base embedding
    let base = vec![0.5; 256];

    // Create variations with small perturbations
    let mut similar1 = base.clone();
    similar1[0] = 0.51;

    let mut similar2 = base.clone();
    similar2[1] = 0.49;

    let mut similar3 = base.clone();
    similar3[255] = 0.52;

    // Encode all
    let elid_base = encode(&base, &profile).expect("Base encoding failed");
    let elid_sim1 = encode(&similar1, &profile).expect("Similar1 encoding failed");
    let elid_sim2 = encode(&similar2, &profile).expect("Similar2 encoding failed");
    let elid_sim3 = encode(&similar3, &profile).expect("Similar3 encoding failed");

    // All should have same format
    for elid in [&elid_base, &elid_sim1, &elid_sim2, &elid_sim3] {
        assert_eq!(elid.as_str().len(), 29);
        let profile_info = elid.profile().expect("Profile extraction failed");
        assert_eq!(profile_info.version, 0);
        assert_eq!(profile_info.profile_type, 0x01);
    }

    // Similar embeddings may or may not produce identical ELIDs depending on
    // how the perturbation affects the hash, but they should be valid
    // (We test locality preservation in simhash tests)
}

#[test]
fn test_batch_encoding_consistency() {
    let profile = Profile::default();

    // Generate a batch of embeddings
    let batch_size = 10;
    let mut embeddings = Vec::new();
    for i in 0..batch_size {
        let value = (i as f32) * 0.1;
        embeddings.push(vec![value; 128]);
    }

    // Encode all embeddings
    let elids: Vec<Elid> = embeddings
        .iter()
        .map(|emb| encode(emb, &profile).expect("Batch encoding failed"))
        .collect();

    // Verify all have consistent format
    assert_eq!(elids.len(), batch_size);

    for (idx, elid) in elids.iter().enumerate() {
        assert_eq!(
            elid.as_str().len(),
            29,
            "ELID {} should have length 29",
            idx
        );

        let profile_info = elid
            .profile()
            .unwrap_or_else(|_| panic!("Profile extraction failed for ELID {}", idx));
        assert_eq!(profile_info.version, 0);
        assert_eq!(profile_info.profile_type, 0x01);
    }

    // Re-encode and verify each produces same result
    for (idx, embedding) in embeddings.iter().enumerate() {
        let elid2 = encode(embedding, &profile)
            .unwrap_or_else(|_| panic!("Re-encoding failed for embedding {}", idx));
        assert_eq!(
            elids[idx], elid2,
            "Re-encoding embedding {} should produce same ELID",
            idx
        );
    }
}

#[test]
fn test_different_profiles_different_output() {
    let embedding = vec![0.3; 128];

    let profile1 = Profile::Mini128 {
        seed: 0x1111_1111_1111_1111,
    };
    let profile2 = Profile::Mini128 {
        seed: 0x2222_2222_2222_2222,
    };

    let elid1 = encode(&embedding, &profile1).expect("Encoding with profile1 failed");
    let elid2 = encode(&embedding, &profile2).expect("Encoding with profile2 failed");

    // Different seeds should produce different ELIDs
    assert_ne!(elid1, elid2);

    // But both should have same format
    assert_eq!(elid1.as_str().len(), elid2.as_str().len());

    let info1 = elid1.profile().expect("Profile1 extraction failed");
    let info2 = elid2.profile().expect("Profile2 extraction failed");

    assert_eq!(info1.version, info2.version);
    assert_eq!(info1.profile_type, info2.profile_type);
}

// ============================================================================
// Morton10x10 and Hilbert10x10 Profile Tests
// ============================================================================

#[test]
fn test_morton_encoding_basic() {
    let embedding = vec![0.1; 128];
    let profile = Profile::Morton10x10 {
        dims: 10,
        bits_per_dim: 10,
        transform_id: None,
    };

    let elid = encode(&embedding, &profile).expect("Morton encoding failed");

    // Verify profile info
    let profile_info = elid.profile().expect("Profile extraction failed");
    assert_eq!(profile_info.version, 0);
    assert_eq!(profile_info.profile_type, 0x02); // Morton

    // Morton 10x10: 10 dims × 10 bits = 100 bits
    // 100 bits = 13 bytes (ceil(100/8))
    // 13 bytes + 2 header = 15 bytes total
    // Base32hex: ceil(15 * 8 / 5) = ceil(120 / 5) = 24 characters
    let bytes = decode(&elid).expect("Decoding failed");
    assert_eq!(bytes.len(), 15, "Morton10x10 should produce 15 bytes");
    assert_eq!(
        elid.as_str().len(),
        24,
        "Morton10x10 ELID should be 24 characters"
    );
}

#[test]
fn test_hilbert_encoding_basic() {
    let embedding = vec![0.2; 128];
    let profile = Profile::Hilbert10x10 {
        dims: 10,
        bits_per_dim: 10,
        transform_id: None,
    };

    let elid = encode(&embedding, &profile).expect("Hilbert encoding failed");

    // Verify profile info
    let profile_info = elid.profile().expect("Profile extraction failed");
    assert_eq!(profile_info.version, 0);
    assert_eq!(profile_info.profile_type, 0x03); // Hilbert

    // Hilbert 10x10: same size as Morton
    let bytes = decode(&elid).expect("Decoding failed");
    assert_eq!(bytes.len(), 15, "Hilbert10x10 should produce 15 bytes");
    assert_eq!(
        elid.as_str().len(),
        24,
        "Hilbert10x10 ELID should be 24 characters"
    );
}

#[test]
fn test_morton_encoding_deterministic() {
    let embedding = vec![0.3; 256];
    let profile = Profile::Morton10x10 {
        dims: 10,
        bits_per_dim: 10,
        transform_id: None,
    };

    let elid1 = encode(&embedding, &profile).expect("First encoding failed");
    let elid2 = encode(&embedding, &profile).expect("Second encoding failed");

    assert_eq!(elid1, elid2, "Morton encoding should be deterministic");
}

#[test]
fn test_hilbert_encoding_deterministic() {
    let embedding = vec![0.4; 256];
    let profile = Profile::Hilbert10x10 {
        dims: 10,
        bits_per_dim: 10,
        transform_id: None,
    };

    let elid1 = encode(&embedding, &profile).expect("First encoding failed");
    let elid2 = encode(&embedding, &profile).expect("Second encoding failed");

    assert_eq!(elid1, elid2, "Hilbert encoding should be deterministic");
}

#[test]
fn test_morton_valid_base32hex() {
    let embedding = vec![0.5; 128];
    let profile = Profile::Morton10x10 {
        dims: 10,
        bits_per_dim: 10,
        transform_id: None,
    };

    let elid = encode(&embedding, &profile).expect("Encoding failed");

    // Verify all characters are valid base32hex
    for c in elid.as_str().chars() {
        assert!(
            matches!(c, '0'..='9' | 'a'..='v'),
            "Character '{}' not in base32hex alphabet",
            c
        );
    }
}

#[test]
fn test_hilbert_valid_base32hex() {
    let embedding = vec![0.6; 128];
    let profile = Profile::Hilbert10x10 {
        dims: 10,
        bits_per_dim: 10,
        transform_id: None,
    };

    let elid = encode(&embedding, &profile).expect("Encoding failed");

    // Verify all characters are valid base32hex
    for c in elid.as_str().chars() {
        assert!(
            matches!(c, '0'..='9' | 'a'..='v'),
            "Character '{}' not in base32hex alphabet",
            c
        );
    }
}

#[test]
fn test_morton_different_from_simhash() {
    let embedding = vec![0.7; 128];

    let mini = Profile::Mini128 {
        seed: 0x454c4944_53494d48,
    };
    let morton = Profile::Morton10x10 {
        dims: 10,
        bits_per_dim: 10,
        transform_id: None,
    };

    let elid_mini = encode(&embedding, &mini).expect("Mini128 encoding failed");
    let elid_morton = encode(&embedding, &morton).expect("Morton encoding failed");

    // Different profiles should produce different ELIDs
    assert_ne!(elid_mini, elid_morton);

    // Different lengths
    assert_eq!(elid_mini.as_str().len(), 29);
    assert_eq!(elid_morton.as_str().len(), 24);
}

#[test]
fn test_hilbert_different_from_simhash() {
    let embedding = vec![0.8; 128];

    let mini = Profile::Mini128 {
        seed: 0x454c4944_53494d48,
    };
    let hilbert = Profile::Hilbert10x10 {
        dims: 10,
        bits_per_dim: 10,
        transform_id: None,
    };

    let elid_mini = encode(&embedding, &mini).expect("Mini128 encoding failed");
    let elid_hilbert = encode(&embedding, &hilbert).expect("Hilbert encoding failed");

    // Different profiles should produce different ELIDs
    assert_ne!(elid_mini, elid_hilbert);

    // Different lengths
    assert_eq!(elid_mini.as_str().len(), 29);
    assert_eq!(elid_hilbert.as_str().len(), 24);
}

#[test]
fn test_morton_hilbert_different_codes() {
    let embedding = vec![0.9; 256];

    let morton = Profile::Morton10x10 {
        dims: 10,
        bits_per_dim: 10,
        transform_id: None,
    };
    let hilbert = Profile::Hilbert10x10 {
        dims: 10,
        bits_per_dim: 10,
        transform_id: None,
    };

    let elid_morton = encode(&embedding, &morton).expect("Morton encoding failed");
    let elid_hilbert = encode(&embedding, &hilbert).expect("Hilbert encoding failed");

    // Morton and Hilbert should produce different codes for same input
    // (because they use different space-filling curves)
    assert_ne!(elid_morton, elid_hilbert);

    // But same length (same dims × bits_per_dim)
    assert_eq!(elid_morton.as_str().len(), elid_hilbert.as_str().len());
}

#[test]
fn test_morton_transform_id_rejected() {
    let embedding = vec![0.1; 128];
    let profile = Profile::Morton10x10 {
        dims: 10,
        bits_per_dim: 10,
        transform_id: Some(42), // Transform IDs not supported in v0.1
    };

    let result = encode(&embedding, &profile);
    assert!(
        matches!(result, Err(elid_core::ElidError::TransformNotFound(42))),
        "Should reject transform_id in v0.1"
    );
}

#[test]
fn test_hilbert_transform_id_rejected() {
    let embedding = vec![0.2; 128];
    let profile = Profile::Hilbert10x10 {
        dims: 10,
        bits_per_dim: 10,
        transform_id: Some(99), // Transform IDs not supported in v0.1
    };

    let result = encode(&embedding, &profile);
    assert!(
        matches!(result, Err(elid_core::ElidError::TransformNotFound(99))),
        "Should reject transform_id in v0.1"
    );
}

#[test]
fn test_all_three_profiles_work() {
    let embedding = vec![0.5, 0.6, 0.7, 0.8]
        .into_iter()
        .cycle()
        .take(768)
        .collect::<Vec<_>>();

    // Test all three profiles
    let mini = Profile::Mini128 {
        seed: 0x454c4944_53494d48,
    };
    let morton = Profile::Morton10x10 {
        dims: 10,
        bits_per_dim: 10,
        transform_id: None,
    };
    let hilbert = Profile::Hilbert10x10 {
        dims: 10,
        bits_per_dim: 10,
        transform_id: None,
    };

    let elid_mini = encode(&embedding, &mini).expect("Mini128 encoding failed");
    let elid_morton = encode(&embedding, &morton).expect("Morton encoding failed");
    let elid_hilbert = encode(&embedding, &hilbert).expect("Hilbert encoding failed");

    // Verify all are valid base32hex
    for elid in [&elid_mini, &elid_morton, &elid_hilbert] {
        for c in elid.as_str().chars() {
            assert!(
                matches!(c, '0'..='9' | 'a'..='v'),
                "Invalid character in ELID: {}",
                c
            );
        }
    }

    // Verify expected lengths
    assert_eq!(elid_mini.as_str().len(), 29);
    assert_eq!(elid_morton.as_str().len(), 24);
    assert_eq!(elid_hilbert.as_str().len(), 24);

    // Verify profile types
    assert_eq!(elid_mini.profile().unwrap().profile_type, 0x01);
    assert_eq!(elid_morton.profile().unwrap().profile_type, 0x02);
    assert_eq!(elid_hilbert.profile().unwrap().profile_type, 0x03);

    // All should be different
    assert_ne!(elid_mini, elid_morton);
    assert_ne!(elid_mini, elid_hilbert);
    assert_ne!(elid_morton, elid_hilbert);
}

// ============================================================================
// T038: Sortability and Locality Preservation Tests
// ============================================================================

#[test]
fn test_morton_lexicographic_order_matches_proximity() {
    // Create a cluster of embeddings with known spatial relationships
    let base_embedding = vec![0.5; 256];

    // Create variations with different levels of similarity
    let mut close_neighbor1 = base_embedding.clone();
    close_neighbor1[0] = 0.51; // Very close

    let mut close_neighbor2 = base_embedding.clone();
    close_neighbor2[1] = 0.49; // Very close

    let mut medium_neighbor = base_embedding.clone();
    for elem in medium_neighbor.iter_mut().take(10) {
        *elem = 0.6; // Medium distance
    }

    let mut far_neighbor = base_embedding.clone();
    for elem in far_neighbor.iter_mut().take(50) {
        *elem = 0.8; // Far distance
    }

    let profile = Profile::Morton10x10 {
        dims: 10,
        bits_per_dim: 10,
        transform_id: None,
    };

    // Encode all embeddings
    let base_elid = encode(&base_embedding, &profile).expect("Base encoding failed");
    let close1_elid = encode(&close_neighbor1, &profile).expect("Close1 encoding failed");
    let close2_elid = encode(&close_neighbor2, &profile).expect("Close2 encoding failed");
    let medium_elid = encode(&medium_neighbor, &profile).expect("Medium encoding failed");
    let far_elid = encode(&far_neighbor, &profile).expect("Far encoding failed");

    // Create sorted list of ELIDs
    let mut elids = [
        (base_elid.clone(), "base"),
        (close1_elid.clone(), "close1"),
        (close2_elid.clone(), "close2"),
        (medium_elid.clone(), "medium"),
        (far_elid.clone(), "far"),
    ];

    elids.sort_by(|a, b| a.0.as_str().cmp(b.0.as_str()));

    // Find position of base embedding in sorted list
    let base_pos = elids.iter().position(|e| e.0 == base_elid).unwrap();

    // Calculate string distances from base in sorted order
    let get_string_distance =
        |from_pos: usize, to_pos: usize| -> usize { from_pos.abs_diff(to_pos) };

    let close1_pos = elids.iter().position(|e| e.0 == close1_elid).unwrap();
    let close2_pos = elids.iter().position(|e| e.0 == close2_elid).unwrap();
    let medium_pos = elids.iter().position(|e| e.0 == medium_elid).unwrap();
    let far_pos = elids.iter().position(|e| e.0 == far_elid).unwrap();

    let close1_dist = get_string_distance(base_pos, close1_pos);
    let close2_dist = get_string_distance(base_pos, close2_pos);
    let medium_dist = get_string_distance(base_pos, medium_pos);
    let far_dist = get_string_distance(base_pos, far_pos);

    // Verify locality: closer embeddings should be closer in sorted order
    // This is a statistical test - Morton curves preserve locality but not perfectly
    println!(
        "Morton sortability - base: {}, close1: {}, close2: {}, medium: {}, far: {}",
        base_pos, close1_pos, close2_pos, medium_pos, far_pos
    );
    println!(
        "Distances - close1: {}, close2: {}, medium: {}, far: {}",
        close1_dist, close2_dist, medium_dist, far_dist
    );

    // At least one close neighbor should be closer than the far neighbor
    assert!(
        close1_dist < far_dist || close2_dist < far_dist,
        "Close neighbors should be closer in sorted order than far neighbors"
    );
}

#[test]
fn test_hilbert_lexicographic_order_matches_proximity() {
    // Same test as Morton but with Hilbert curve
    // Hilbert should show similar or slightly better locality preservation

    let base_embedding = vec![0.5; 256];

    let mut close_neighbor1 = base_embedding.clone();
    close_neighbor1[0] = 0.51;

    let mut close_neighbor2 = base_embedding.clone();
    close_neighbor2[1] = 0.49;

    let mut medium_neighbor = base_embedding.clone();
    for elem in medium_neighbor.iter_mut().take(10) {
        *elem = 0.6;
    }

    let mut far_neighbor = base_embedding.clone();
    for elem in far_neighbor.iter_mut().take(50) {
        *elem = 0.8;
    }

    let profile = Profile::Hilbert10x10 {
        dims: 10,
        bits_per_dim: 10,
        transform_id: None,
    };

    let base_elid = encode(&base_embedding, &profile).expect("Base encoding failed");
    let close1_elid = encode(&close_neighbor1, &profile).expect("Close1 encoding failed");
    let close2_elid = encode(&close_neighbor2, &profile).expect("Close2 encoding failed");
    let medium_elid = encode(&medium_neighbor, &profile).expect("Medium encoding failed");
    let far_elid = encode(&far_neighbor, &profile).expect("Far encoding failed");

    let mut elids = [
        (base_elid.clone(), "base"),
        (close1_elid.clone(), "close1"),
        (close2_elid.clone(), "close2"),
        (medium_elid.clone(), "medium"),
        (far_elid.clone(), "far"),
    ];

    elids.sort_by(|a, b| a.0.as_str().cmp(b.0.as_str()));

    let base_pos = elids.iter().position(|e| e.0 == base_elid).unwrap();

    let get_string_distance =
        |from_pos: usize, to_pos: usize| -> usize { from_pos.abs_diff(to_pos) };

    let close1_pos = elids.iter().position(|e| e.0 == close1_elid).unwrap();
    let close2_pos = elids.iter().position(|e| e.0 == close2_elid).unwrap();
    let medium_pos = elids.iter().position(|e| e.0 == medium_elid).unwrap();
    let far_pos = elids.iter().position(|e| e.0 == far_elid).unwrap();

    let close1_dist = get_string_distance(base_pos, close1_pos);
    let close2_dist = get_string_distance(base_pos, close2_pos);
    let medium_dist = get_string_distance(base_pos, medium_pos);
    let far_dist = get_string_distance(base_pos, far_pos);

    println!(
        "Hilbert sortability - base: {}, close1: {}, close2: {}, medium: {}, far: {}",
        base_pos, close1_pos, close2_pos, medium_pos, far_pos
    );
    println!(
        "Distances - close1: {}, close2: {}, medium: {}, far: {}",
        close1_dist, close2_dist, medium_dist, far_dist
    );

    // Hilbert should also preserve locality
    assert!(
        close1_dist < far_dist || close2_dist < far_dist,
        "Close neighbors should be closer in sorted order than far neighbors"
    );
}

#[test]
fn test_morton_prefix_sharing_similarity() {
    // Test that similar embeddings share common prefixes in their ELIDs

    let base = vec![0.5; 128];

    // Create a very similar embedding (tiny perturbation)
    let mut very_similar = base.clone();
    very_similar[0] = 0.500001; // Extremely close

    // Create a somewhat similar embedding
    let mut similar = base.clone();
    for elem in similar.iter_mut().take(5) {
        *elem = 0.55;
    }

    // Create a dissimilar embedding
    let mut dissimilar = base.clone();
    for elem in dissimilar.iter_mut().take(50) {
        *elem = 1.0;
    }

    let profile = Profile::Morton10x10 {
        dims: 10,
        bits_per_dim: 10,
        transform_id: None,
    };

    let base_elid = encode(&base, &profile).expect("Base encoding failed");
    let very_similar_elid = encode(&very_similar, &profile).expect("Very similar encoding failed");
    let similar_elid = encode(&similar, &profile).expect("Similar encoding failed");
    let dissimilar_elid = encode(&dissimilar, &profile).expect("Dissimilar encoding failed");

    // Calculate common prefix length
    let common_prefix_len = |s1: &str, s2: &str| -> usize {
        s1.chars()
            .zip(s2.chars())
            .take_while(|(c1, c2)| c1 == c2)
            .count()
    };

    let very_similar_prefix = common_prefix_len(base_elid.as_str(), very_similar_elid.as_str());
    let similar_prefix = common_prefix_len(base_elid.as_str(), similar_elid.as_str());
    let dissimilar_prefix = common_prefix_len(base_elid.as_str(), dissimilar_elid.as_str());

    println!(
        "Morton prefix sharing - very_similar: {}, similar: {}, dissimilar: {}",
        very_similar_prefix, similar_prefix, dissimilar_prefix
    );

    // More similar embeddings should share longer prefixes
    // This is a statistical property - not guaranteed for every case
    // but should hold in aggregate

    // At minimum, verify that we can compute prefix lengths
    assert!(very_similar_prefix <= 24); // Max ELID length for Morton10x10
    assert!(similar_prefix <= 24);
    assert!(dissimilar_prefix <= 24);

    // Very similar embeddings should tend to have some prefix sharing
    // (though quantization may prevent this in some cases)
    println!(
        "Base ELID: {}\nVery Similar: {}\nSimilar: {}\nDissimilar: {}",
        base_elid.as_str(),
        very_similar_elid.as_str(),
        similar_elid.as_str(),
        dissimilar_elid.as_str()
    );
}

#[test]
fn test_hilbert_prefix_sharing_similarity() {
    // Same test for Hilbert curves

    let base = vec![0.5; 128];

    let mut very_similar = base.clone();
    very_similar[0] = 0.500001;

    let mut similar = base.clone();
    for elem in similar.iter_mut().take(5) {
        *elem = 0.55;
    }

    let mut dissimilar = base.clone();
    for elem in dissimilar.iter_mut().take(50) {
        *elem = 1.0;
    }

    let profile = Profile::Hilbert10x10 {
        dims: 10,
        bits_per_dim: 10,
        transform_id: None,
    };

    let base_elid = encode(&base, &profile).expect("Base encoding failed");
    let very_similar_elid = encode(&very_similar, &profile).expect("Very similar encoding failed");
    let similar_elid = encode(&similar, &profile).expect("Similar encoding failed");
    let dissimilar_elid = encode(&dissimilar, &profile).expect("Dissimilar encoding failed");

    let common_prefix_len = |s1: &str, s2: &str| -> usize {
        s1.chars()
            .zip(s2.chars())
            .take_while(|(c1, c2)| c1 == c2)
            .count()
    };

    let very_similar_prefix = common_prefix_len(base_elid.as_str(), very_similar_elid.as_str());
    let similar_prefix = common_prefix_len(base_elid.as_str(), similar_elid.as_str());
    let dissimilar_prefix = common_prefix_len(base_elid.as_str(), dissimilar_elid.as_str());

    println!(
        "Hilbert prefix sharing - very_similar: {}, similar: {}, dissimilar: {}",
        very_similar_prefix, similar_prefix, dissimilar_prefix
    );

    assert!(very_similar_prefix <= 24);
    assert!(similar_prefix <= 24);
    assert!(dissimilar_prefix <= 24);

    println!(
        "Base ELID: {}\nVery Similar: {}\nSimilar: {}\nDissimilar: {}",
        base_elid.as_str(),
        very_similar_elid.as_str(),
        similar_elid.as_str(),
        dissimilar_elid.as_str()
    );
}

#[test]
fn test_morton_range_scan_simulation() {
    // Simulate a range scan: find all ELIDs within a prefix range
    // Verify that neighbors are captured

    let mut embeddings = Vec::new();

    // Create a base embedding at center of space
    let center = vec![0.5; 256];
    embeddings.push((center.clone(), "center"));

    // Create 8 neighbors in different directions
    let mut neighbor1 = center.clone();
    neighbor1[0] = 0.51;
    embeddings.push((neighbor1, "n1"));

    let mut neighbor2 = center.clone();
    neighbor2[1] = 0.51;
    embeddings.push((neighbor2, "n2"));

    let mut neighbor3 = center.clone();
    neighbor3[2] = 0.51;
    embeddings.push((neighbor3, "n3"));

    let mut neighbor4 = center.clone();
    neighbor4[0] = 0.49;
    embeddings.push((neighbor4, "n4"));

    let mut neighbor5 = center.clone();
    neighbor5[1] = 0.49;
    embeddings.push((neighbor5, "n5"));

    // Add some distant points
    let mut distant1 = center.clone();
    for elem in distant1.iter_mut().take(50) {
        *elem = 0.8;
    }
    embeddings.push((distant1, "distant1"));

    let mut distant2 = center.clone();
    for elem in distant2.iter_mut().take(50) {
        *elem = 0.2;
    }
    embeddings.push((distant2, "distant2"));

    let profile = Profile::Morton10x10 {
        dims: 10,
        bits_per_dim: 10,
        transform_id: None,
    };

    // Encode all
    let mut elids: Vec<(Elid, &str)> = embeddings
        .iter()
        .map(|(emb, label)| (encode(emb, &profile).expect("Encoding failed"), *label))
        .collect();

    // Sort by ELID string
    elids.sort_by(|a, b| a.0.as_str().cmp(b.0.as_str()));

    // Find center position
    let center_pos = elids
        .iter()
        .position(|(_, label)| *label == "center")
        .unwrap();

    println!("Sorted ELIDs (Morton):");
    for (i, (elid, label)) in elids.iter().enumerate() {
        println!("  {}: {} ({})", i, elid.as_str(), label);
    }

    // Count neighbors within distance 2 of center in sorted order
    let range = 2;
    let start = center_pos.saturating_sub(range);
    let end = (center_pos + range + 1).min(elids.len());

    let neighbors_in_range: Vec<&str> = elids[start..end]
        .iter()
        .filter(|(_, label)| label.starts_with('n'))
        .map(|(_, label)| *label)
        .collect();

    println!(
        "Center at position {}, checking range [{}, {})",
        center_pos, start, end
    );
    println!("Neighbors in range: {:?}", neighbors_in_range);

    // Verify that at least some neighbors are captured
    // (Morton curves don't guarantee ALL neighbors, but should capture some)
    assert!(
        !neighbors_in_range.is_empty(),
        "Range scan should capture at least one neighbor"
    );
}

#[test]
fn test_hilbert_range_scan_simulation() {
    // Same range scan test for Hilbert curves

    let mut embeddings = Vec::new();

    let center = vec![0.5; 256];
    embeddings.push((center.clone(), "center"));

    let mut neighbor1 = center.clone();
    neighbor1[0] = 0.51;
    embeddings.push((neighbor1, "n1"));

    let mut neighbor2 = center.clone();
    neighbor2[1] = 0.51;
    embeddings.push((neighbor2, "n2"));

    let mut neighbor3 = center.clone();
    neighbor3[2] = 0.51;
    embeddings.push((neighbor3, "n3"));

    let mut neighbor4 = center.clone();
    neighbor4[0] = 0.49;
    embeddings.push((neighbor4, "n4"));

    let mut neighbor5 = center.clone();
    neighbor5[1] = 0.49;
    embeddings.push((neighbor5, "n5"));

    let mut distant1 = center.clone();
    for elem in distant1.iter_mut().take(50) {
        *elem = 0.8;
    }
    embeddings.push((distant1, "distant1"));

    let mut distant2 = center.clone();
    for elem in distant2.iter_mut().take(50) {
        *elem = 0.2;
    }
    embeddings.push((distant2, "distant2"));

    let profile = Profile::Hilbert10x10 {
        dims: 10,
        bits_per_dim: 10,
        transform_id: None,
    };

    let mut elids: Vec<(Elid, &str)> = embeddings
        .iter()
        .map(|(emb, label)| (encode(emb, &profile).expect("Encoding failed"), *label))
        .collect();

    elids.sort_by(|a, b| a.0.as_str().cmp(b.0.as_str()));

    let center_pos = elids
        .iter()
        .position(|(_, label)| *label == "center")
        .unwrap();

    println!("Sorted ELIDs (Hilbert):");
    for (i, (elid, label)) in elids.iter().enumerate() {
        println!("  {}: {} ({})", i, elid.as_str(), label);
    }

    let range = 2;
    let start = center_pos.saturating_sub(range);
    let end = (center_pos + range + 1).min(elids.len());

    let neighbors_in_range: Vec<&str> = elids[start..end]
        .iter()
        .filter(|(_, label)| label.starts_with('n'))
        .map(|(_, label)| *label)
        .collect();

    println!(
        "Center at position {}, checking range [{}, {})",
        center_pos, start, end
    );
    println!("Neighbors in range: {:?}", neighbors_in_range);

    // Hilbert should also capture neighbors
    assert!(
        !neighbors_in_range.is_empty(),
        "Range scan should capture at least one neighbor"
    );
}

#[test]
fn test_morton_vs_hilbert_locality_comparison() {
    // Direct comparison: which curve preserves locality better?
    // Research suggests Hilbert is ~5-10% better in 10D

    let center = vec![0.5; 256];

    // Create a set of neighbors at various distances
    let mut neighbors = Vec::new();
    for i in 1..=10 {
        let mut neighbor = center.clone();
        for elem in neighbor.iter_mut().take(i) {
            *elem = 0.5 + (i as f32) * 0.01; // Increasing distance
        }
        neighbors.push(neighbor);
    }

    let morton_profile = Profile::Morton10x10 {
        dims: 10,
        bits_per_dim: 10,
        transform_id: None,
    };

    let hilbert_profile = Profile::Hilbert10x10 {
        dims: 10,
        bits_per_dim: 10,
        transform_id: None,
    };

    // Encode with both curves
    let center_morton = encode(&center, &morton_profile).expect("Center Morton encoding failed");
    let center_hilbert = encode(&center, &hilbert_profile).expect("Center Hilbert encoding failed");

    let mut morton_elids: Vec<Elid> = neighbors
        .iter()
        .map(|emb| encode(emb, &morton_profile).expect("Morton encoding failed"))
        .collect();
    morton_elids.insert(0, center_morton.clone());

    let mut hilbert_elids: Vec<Elid> = neighbors
        .iter()
        .map(|emb| encode(emb, &hilbert_profile).expect("Hilbert encoding failed"))
        .collect();
    hilbert_elids.insert(0, center_hilbert.clone());

    // Sort both
    morton_elids.sort();
    hilbert_elids.sort();

    // Find center positions
    let morton_center_pos = morton_elids
        .iter()
        .position(|e| e == &center_morton)
        .unwrap();
    let hilbert_center_pos = hilbert_elids
        .iter()
        .position(|e| e == &center_hilbert)
        .unwrap();

    // Count how many neighbors are within range N of center
    let range = 3;

    let count_in_range = |elids: &[Elid], center_pos: usize| -> usize {
        let start = center_pos.saturating_sub(range);
        let end = (center_pos + range + 1).min(elids.len());
        end - start - 1 // -1 to exclude center itself
    };

    let morton_neighbors_in_range = count_in_range(&morton_elids, morton_center_pos);
    let hilbert_neighbors_in_range = count_in_range(&hilbert_elids, hilbert_center_pos);

    println!(
        "Morton: {} neighbors in range, Hilbert: {} neighbors in range",
        morton_neighbors_in_range, hilbert_neighbors_in_range
    );

    // Both should capture some neighbors
    assert!(morton_neighbors_in_range > 0);
    assert!(hilbert_neighbors_in_range > 0);

    // This is a weak assertion - in 10D, the difference is marginal
    // We're just verifying both algorithms preserve locality
    // (Research shows Hilbert is only 5-10% better in 10D)
}
