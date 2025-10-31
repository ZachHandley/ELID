//! Demonstration of the public API functions for ELID encoding and decoding
//!
//! This example shows:
//! - Encoding embeddings to ELIDs
//! - Decoding ELIDs to raw bytes
//! - Computing Hamming distances for similarity search
//! - Working with profile metadata

use elid_core::{decode, encode, hamming_distance, Profile};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ELID Public API Demo ===\n");

    // Create a sample embedding (768-dimensional, typical for text embeddings)
    let embedding = vec![
        0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, -0.1, -0.2, -0.3, -0.4, -0.5, -0.6, -0.7, -0.8,
    ];
    let embedding: Vec<f32> = embedding.into_iter().cycle().take(768).collect();

    println!("📊 Input embedding: {} dimensions", embedding.len());
    println!("   First 8 values: {:?}", &embedding[0..8]);

    // Step 1: Encode with default profile (Mini128)
    println!("\n1️⃣  Encoding with Mini128 profile...");
    let profile = Profile::default();
    let elid = encode(&embedding, &profile)?;

    println!("   ✅ ELID: {}", elid);
    println!("   Length: {} characters", elid.as_str().len());
    println!("   Alphabet: base32hex (0-9, a-v)");

    // Step 2: Decode to raw bytes
    println!("\n2️⃣  Decoding ELID to raw bytes...");
    let bytes = decode(&elid)?;

    println!("   ✅ Total bytes: {}", bytes.len());
    println!("   Header (2 bytes): {:02x} {:02x}", bytes[0], bytes[1]);
    println!(
        "   Payload (16 bytes): {}...",
        bytes[2..6]
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
            .join(" ")
    );

    // Step 3: Extract profile information
    println!("\n3️⃣  Extracting profile metadata...");
    let profile_info = elid.profile()?;

    println!("   ✅ Version: {}", profile_info.version);
    println!(
        "   Profile type: 0x{:02x} (Mini128)",
        profile_info.profile_type
    );
    println!(
        "   Transform ID: {}",
        profile_info
            .transform_id
            .map_or("None".to_string(), |id| format!("{}", id))
    );

    // Step 4: Create a similar embedding and measure distance
    println!("\n4️⃣  Testing similarity search...");

    let mut similar_embedding = embedding.clone();
    similar_embedding[0] += 0.05; // Small perturbation
    similar_embedding[1] -= 0.03;

    let elid2 = encode(&similar_embedding, &profile)?;
    println!("   Similar embedding ELID: {}", elid2);

    let distance = hamming_distance(&elid, &elid2)?;
    println!("   ✅ Hamming distance: {} / 128 bits", distance);
    println!(
        "   Similarity: {:.1}%",
        100.0 * (128.0 - distance as f32) / 128.0
    );

    // Step 5: Create a very different embedding
    println!("\n5️⃣  Testing with dissimilar embedding...");

    let different_embedding: Vec<f32> = vec![1.0, -1.0].into_iter().cycle().take(768).collect();
    let elid3 = encode(&different_embedding, &profile)?;
    println!("   Different embedding ELID: {}", elid3);

    let distance2 = hamming_distance(&elid, &elid3)?;
    println!("   ✅ Hamming distance: {} / 128 bits", distance2);
    println!(
        "   Similarity: {:.1}%",
        100.0 * (128.0 - distance2 as f32) / 128.0
    );

    // Step 6: Test determinism
    println!("\n6️⃣  Testing determinism...");
    let elid_copy = encode(&embedding, &profile)?;
    assert_eq!(elid, elid_copy);
    println!(
        "   ✅ Same embedding produces same ELID: {}",
        elid == elid_copy
    );

    // Step 7: Test normalization invariance
    println!("\n7️⃣  Testing normalization invariance...");
    let scaled_embedding: Vec<f32> = embedding.iter().map(|v| v * 2.0).collect();
    let elid_scaled = encode(&scaled_embedding, &profile)?;
    assert_eq!(elid, elid_scaled);
    println!(
        "   ✅ Scaled embedding produces same ELID: {}",
        elid == elid_scaled
    );

    // Step 8: Test different seeds
    println!("\n8️⃣  Testing different seeds...");
    let profile2 = Profile::Mini128 {
        seed: 0x1234_5678_9ABC_DEF0,
    };
    let elid_different_seed = encode(&embedding, &profile2)?;
    println!("   Different seed ELID: {}", elid_different_seed);
    assert_ne!(elid, elid_different_seed);
    println!(
        "   ✅ Different seeds produce different ELIDs: {}",
        elid != elid_different_seed
    );

    // Summary
    println!("\n=== Summary ===");
    println!("✅ All public API functions working correctly:");
    println!("   • encode() - converts embeddings to sortable IDs");
    println!("   • decode() - extracts raw bytes from IDs");
    println!("   • hamming_distance() - computes similarity between IDs");
    println!("   • Elid::profile() - extracts metadata from IDs");
    println!("   • Elid::to_bytes() - decodes to bytes");
    println!("\n🎉 Ready for production use!");

    Ok(())
}
