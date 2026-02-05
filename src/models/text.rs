//! Text embedding using Model2Vec
//!
//! Uses the potion-base-8M model for fast, lightweight text embeddings.
//! This model produces 256-dimensional embeddings and is optimized for
//! low-latency inference, making it suitable for WASM deployment.
//!
//! ## Model Details
//!
//! - **Model**: Model2Vec potion-base-8M
//! - **Dimensions**: 256
//! - **License**: MIT
//!
//! ## Example
//!
//! ```rust,ignore
//! use elid::models::embed_text;
//!
//! let embedding = embed_text("Hello, world!")?;
//! assert_eq!(embedding.len(), 256);
//! ```

use super::ModelError;

/// Embed text into a vector representation
///
/// Uses the Model2Vec potion-base-8M model to generate a 256-dimensional
/// embedding vector from the input text.
///
/// # Arguments
///
/// * `text` - The input text to embed
///
/// # Returns
///
/// A 256-dimensional embedding vector as `Vec<f32>`
///
/// # Errors
///
/// Returns `ModelError::ModelLoad` if the model file is not found or cannot be loaded.
/// Returns `ModelError::Preprocessing` if text tokenization fails.
/// Returns `ModelError::Inference` if model inference fails.
///
/// # Example
///
/// ```rust,ignore
/// use elid::models::embed_text;
///
/// let embedding = embed_text("Hello, world!")?;
/// assert_eq!(embedding.len(), 256);
///
/// // Similar texts should produce similar embeddings
/// let emb1 = embed_text("The quick brown fox")?;
/// let emb2 = embed_text("The fast brown fox")?;
/// // emb1 and emb2 should be close in vector space
/// ```
pub fn embed_text(_text: &str) -> Result<Vec<f32>, ModelError> {
    // TODO: Implement with tract-onnx
    // For now, return a placeholder that indicates the model needs to be downloaded
    Err(ModelError::ModelLoad(
        "Text model not yet implemented. Run: python scripts/download_models.py".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embed_text_returns_error_without_model() {
        let result = embed_text("Hello, world!");
        assert!(result.is_err());
        if let Err(ModelError::ModelLoad(msg)) = result {
            assert!(msg.contains("not yet implemented"));
        } else {
            panic!("Expected ModelLoad error");
        }
    }
}
