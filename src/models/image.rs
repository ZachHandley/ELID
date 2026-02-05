//! Image embedding using MobileNetV3
//!
//! Uses MobileNetV3-Small for fast, lightweight image feature extraction.
//! This model produces 1024-dimensional embeddings and is optimized for
//! mobile and edge deployment, making it suitable for WASM.
//!
//! ## Model Details
//!
//! - **Model**: MobileNetV3-Small
//! - **Input**: 224x224 RGB images
//! - **Dimensions**: 1024
//! - **License**: Apache 2.0
//!
//! ## Supported Formats
//!
//! - JPEG
//! - PNG
//!
//! ## Example
//!
//! ```rust,ignore
//! use elid::models::embed_image;
//!
//! let bytes = std::fs::read("image.jpg")?;
//! let embedding = embed_image(&bytes)?;
//! assert_eq!(embedding.len(), 1024);
//! ```

use super::ModelError;

/// Embed an image into a vector representation
///
/// Uses MobileNetV3-Small to generate a 1024-dimensional feature vector
/// from the input image. The image is automatically resized to 224x224
/// and normalized before inference.
///
/// # Arguments
///
/// * `image_bytes` - Raw image bytes (JPEG or PNG format)
///
/// # Returns
///
/// A 1024-dimensional feature vector as `Vec<f32>`
///
/// # Errors
///
/// Returns `ModelError::ModelLoad` if the model file is not found or cannot be loaded.
/// Returns `ModelError::Preprocessing` if image decoding or resizing fails.
/// Returns `ModelError::Inference` if model inference fails.
///
/// # Example
///
/// ```rust,ignore
/// use elid::models::embed_image;
///
/// let bytes = std::fs::read("photo.jpg")?;
/// let embedding = embed_image(&bytes)?;
/// assert_eq!(embedding.len(), 1024);
///
/// // Similar images should produce similar embeddings
/// let emb1 = embed_image(&std::fs::read("cat1.jpg")?)?;
/// let emb2 = embed_image(&std::fs::read("cat2.jpg")?)?;
/// // emb1 and emb2 should be close in vector space
/// ```
pub fn embed_image(_image_bytes: &[u8]) -> Result<Vec<f32>, ModelError> {
    // TODO: Implement with tract-onnx + image crate
    // For now, return a placeholder that indicates the model needs to be downloaded
    Err(ModelError::ModelLoad(
        "Image model not yet implemented. Run: python scripts/download_models.py".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embed_image_returns_error_without_model() {
        let dummy_bytes = vec![0u8; 100];
        let result = embed_image(&dummy_bytes);
        assert!(result.is_err());
        if let Err(ModelError::ModelLoad(msg)) = result {
            assert!(msg.contains("not yet implemented"));
        } else {
            panic!("Expected ModelLoad error");
        }
    }
}
