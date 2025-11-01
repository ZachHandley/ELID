use anyhow::{anyhow, Context, Result};
use elid_core::{decode as core_decode, encode as core_encode, hamming_distance as core_hamming_distance, Elid as CoreElid, Profile as CoreProfile};
use flutter_rust_bridge::frb;

/// ELID encoding profile
#[frb(dart_metadata=("freezed"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    /// 128-bit SimHash (29 characters) - Best for similarity search
    Mini128,
    /// 10D Morton curve (16-24 characters) - Fast indexing
    Morton10x10,
    /// 10D Hilbert curve (16-24 characters) - Maximum locality
    Hilbert10x10,
}

impl From<Profile> for CoreProfile {
    fn from(profile: Profile) -> Self {
        match profile {
            Profile::Mini128 => CoreProfile::Mini128 { seed: 0 },
            Profile::Morton10x10 => CoreProfile::Morton10x10 {
                dims: 10,
                bits_per_dim: 10,
                transform_id: None,
            },
            Profile::Hilbert10x10 => CoreProfile::Hilbert10x10 {
                dims: 10,
                bits_per_dim: 10,
                transform_id: None,
            },
        }
    }
}

/// Initialize the ELID library (placeholder for future initialization needs)
#[frb]
pub fn init() -> Result<()> {
    // Currently no initialization needed, but keeping for API compatibility
    Ok(())
}

/// Encode a single embedding to ELID (synchronous)
///
/// # Arguments
/// * `embedding` - List of doubles with length 64-2048
/// * `profile` - Encoding profile (Mini128, Morton10x10, or Hilbert10x10)
///
/// # Returns
/// Sortable string identifier (29 chars for Mini128, 16-24 for others)
///
/// # Errors
/// Returns error if embedding dimensions are invalid (not in 64-2048 range)
#[frb(sync)]
pub fn encode(embedding: Vec<f64>, profile: Profile) -> Result<String> {
    let core_profile: CoreProfile = profile.into();
    // Convert f64 to f32
    let embedding_f32: Vec<f32> = embedding.iter().map(|&x| x as f32).collect();
    let elid = core_encode(&embedding_f32, &core_profile)
        .context("Failed to encode embedding")?;
    Ok(elid.to_string())
}

/// Decode ELID string to raw bytes (synchronous)
///
/// # Arguments
/// * `elid` - ELID string identifier (base32hex encoded)
///
/// # Returns
/// List of bytes (16 bytes for Mini128, 10 for Morton/Hilbert10x10)
///
/// # Errors
/// Returns error if ELID string is malformed
#[frb(sync)]
pub fn decode(elid: String) -> Result<Vec<u8>> {
    let elid_obj = CoreElid::from_string(elid)
        .context("Failed to parse ELID string")?;
    core_decode(&elid_obj)
        .context("Failed to decode ELID")
}

/// Calculate Hamming distance between two ELIDs (synchronous)
///
/// # Arguments
/// * `elid1` - First ELID string
/// * `elid2` - Second ELID string (must use same profile as elid1)
///
/// # Returns
/// Hamming distance (0-128 for Mini128, 0-80 for Morton/Hilbert10x10)
/// Lower distance indicates higher similarity.
///
/// # Errors
/// Returns error if ELIDs use different profiles or are malformed
#[frb(sync)]
pub fn hamming_distance(elid1: String, elid2: String) -> Result<u32> {
    let e1 = CoreElid::from_string(elid1)
        .context("Failed to parse first ELID")?;
    let e2 = CoreElid::from_string(elid2)
        .context("Failed to parse second ELID")?;

    core_hamming_distance(&e1, &e2)
        .context("Failed to compute hamming distance")
}

/// Encode multiple embeddings asynchronously (does not block UI thread)
///
/// # Arguments
/// * `embeddings` - List of embedding vectors, each length 64-2048
/// * `profile` - Encoding profile for all embeddings
///
/// # Returns
/// Future resolving to list of ELID strings, same order as input
///
/// # Errors
/// Returns error if any embedding has invalid dimensions
#[frb]
pub async fn encode_batch(embeddings: Vec<Vec<f64>>, profile: Profile) -> Result<Vec<String>> {
    let core_profile: CoreProfile = profile.into();

    // Use tokio spawn_blocking to avoid blocking the thread pool
    tokio::task::spawn_blocking(move || {
        let mut results = Vec::with_capacity(embeddings.len());

        for emb_data in embeddings {
            // Convert f64 to f32
            let embedding_f32: Vec<f32> = emb_data.iter().map(|&x| x as f32).collect();
            let elid = core_encode(&embedding_f32, &core_profile)
                .context("Failed to encode embedding")?;
            results.push(elid.to_string());
        }

        Ok(results)
    })
    .await
    .context("Task join error")?
}

/// Stream-based encoding for real-time UI updates
/// Yields ELID strings progressively as embeddings are encoded
///
/// # Arguments
/// * `embeddings` - List of embedding vectors, each length 64-2048
/// * `profile` - Encoding profile for all embeddings
///
/// # Returns
/// Stream emitting ELID strings as they are encoded
#[frb]
pub fn encode_stream(
    embeddings: Vec<Vec<f64>>,
    profile: Profile,
) -> impl futures::Stream<Item = String> {
    use futures::stream;

    let core_profile: CoreProfile = profile.into();

    stream::iter(embeddings.into_iter().filter_map(move |emb_data| {
        // Convert f64 to f32
        let embedding_f32: Vec<f32> = emb_data.iter().map(|&x| x as f32).collect();
        match core_encode(&embedding_f32, &core_profile) {
            Ok(elid) => Some(elid.to_string()),
            Err(_) => None,
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode() {
        let embedding = vec![0.5; 768];
        let elid_str = encode(embedding, Profile::Mini128).unwrap();
        assert_eq!(elid_str.len(), 29);

        let bytes = decode(elid_str).unwrap();
        assert_eq!(bytes.len(), 18); // 2 header + 16 payload
    }

    #[test]
    fn test_hamming_distance() {
        let embedding = vec![0.5; 768];
        let elid1 = encode(embedding.clone(), Profile::Mini128).unwrap();
        let elid2 = encode(embedding, Profile::Mini128).unwrap();

        let distance = hamming_distance(elid1, elid2).unwrap();
        assert_eq!(distance, 0);
    }

    #[tokio::test]
    async fn test_encode_batch() {
        let embeddings: Vec<Vec<f64>> = vec![
            vec![0.1; 768],
            vec![0.2; 768],
            vec![0.3; 768],
        ];

        let results = encode_batch(embeddings, Profile::Mini128).await.unwrap();
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|s| s.len() == 29));
    }
}
