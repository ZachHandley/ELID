//! Text embedding using Model2Vec
//!
//! Uses the potion-base-8M model for fast, lightweight text embeddings.
//! Unlike traditional transformer models, Model2Vec is a static embedding
//! model: it tokenizes, looks up token embeddings, and mean-pools them.
//! No ONNX inference needed — just a matrix lookup.
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

use super::error::ModelError;
use super::models_dir;
use safetensors::SafeTensors;
use std::sync::OnceLock;
use tokenizers::Tokenizer;

/// Cached text model for reuse across calls (file-based loading)
static TEXT_MODEL: OnceLock<Result<TextModel, String>> = OnceLock::new();

/// Cached text model loaded via remote fetch (init_text_model)
static REMOTE_TEXT_MODEL: OnceLock<TextModel> = OnceLock::new();

/// Model2Vec text embedding model
///
/// Holds the embedding matrix and tokenizer. Inference is a simple
/// lookup + mean pool — no neural network forward pass.
pub(crate) struct TextModel {
    /// Flattened embedding matrix [vocab_size * hidden_dim]
    embeddings: Vec<f32>,
    /// Number of vocabulary entries
    vocab_size: usize,
    /// Embedding dimensionality (256 for potion-base-8M)
    hidden_dim: usize,
    /// Whether to L2-normalize output embeddings
    normalize: bool,
    /// HuggingFace tokenizer
    tokenizer: Tokenizer,
}

/// Model2Vec config parsed from config.json
#[derive(serde::Deserialize)]
struct Model2VecConfig {
    hidden_dim: usize,
    #[serde(default)]
    normalize: bool,
}

impl TextModel {
    /// Load model from a directory containing the safetensors, tokenizer, and config files
    fn load_from_dir(dir: &std::path::Path) -> Result<Self, ModelError> {
        let safetensors_path = dir.join("potion-base-8m.safetensors");
        let tokenizer_path = dir.join("potion-base-8m-tokenizer.json");
        let config_path = dir.join("potion-base-8m-config.json");

        if !safetensors_path.exists() {
            return Err(ModelError::ModelLoad(format!(
                "Model file not found: {}. Run: python scripts/download_models.py --text-only",
                safetensors_path.display()
            )));
        }

        Self::load_from_files(&safetensors_path, &tokenizer_path, &config_path)
    }

    /// Load from specific file paths
    fn load_from_files(
        safetensors_path: &std::path::Path,
        tokenizer_path: &std::path::Path,
        config_path: &std::path::Path,
    ) -> Result<Self, ModelError> {
        // Load config
        let config_bytes = std::fs::read(config_path)?;
        let config: Model2VecConfig = serde_json::from_slice(&config_bytes)?;

        // Load tokenizer
        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| ModelError::ModelLoad(format!("Tokenizer load failed: {e}")))?;

        // Load safetensors embedding matrix
        let safetensors_bytes = std::fs::read(safetensors_path)?;
        let (embeddings, vocab_size) =
            Self::parse_embeddings(&safetensors_bytes, config.hidden_dim)?;

        Ok(Self {
            embeddings,
            vocab_size,
            hidden_dim: config.hidden_dim,
            normalize: config.normalize,
            tokenizer,
        })
    }

    /// Load model from raw bytes (for WASM — no filesystem)
    pub(crate) fn load_from_bytes(
        safetensors_bytes: &[u8],
        tokenizer_json: &[u8],
        config_json: &[u8],
    ) -> Result<Self, ModelError> {
        let config: Model2VecConfig = serde_json::from_slice(config_json)?;

        let tokenizer = Tokenizer::from_bytes(tokenizer_json)
            .map_err(|e| ModelError::ModelLoad(format!("Tokenizer load failed: {e}")))?;

        let (embeddings, vocab_size) =
            Self::parse_embeddings(safetensors_bytes, config.hidden_dim)?;

        Ok(Self {
            embeddings,
            vocab_size,
            hidden_dim: config.hidden_dim,
            normalize: config.normalize,
            tokenizer,
        })
    }

    /// Parse the embedding matrix from safetensors bytes
    fn parse_embeddings(bytes: &[u8], hidden_dim: usize) -> Result<(Vec<f32>, usize), ModelError> {
        let tensors = SafeTensors::deserialize(bytes)?;

        // Model2Vec stores embeddings under "embeddings" key
        let tensor = tensors
            .tensor("embeddings")
            .map_err(|e| ModelError::ModelLoad(format!("Missing 'embeddings' tensor: {e}")))?;

        let shape = tensor.shape();
        if shape.len() != 2 || shape[1] != hidden_dim {
            return Err(ModelError::ModelLoad(format!(
                "Expected embeddings shape [vocab_size, {hidden_dim}], got {shape:?}"
            )));
        }

        let vocab_size = shape[0];

        // Convert raw bytes to f32 (safetensors stores as little-endian)
        let data = tensor.data();
        if data.len() != vocab_size * hidden_dim * 4 {
            return Err(ModelError::ModelLoad(format!(
                "Embedding data size mismatch: expected {} bytes, got {}",
                vocab_size * hidden_dim * 4,
                data.len()
            )));
        }

        let embeddings: Vec<f32> = data
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();

        Ok((embeddings, vocab_size))
    }

    /// Embed text into a vector
    pub(crate) fn embed(&self, text: &str) -> Result<Vec<f32>, ModelError> {
        // Tokenize without special tokens (matching model2vec-rs behavior)
        let encoding = self
            .tokenizer
            .encode(text, false)
            .map_err(|e| ModelError::Preprocessing(format!("Tokenization failed: {e}")))?;

        let ids = encoding.get_ids();

        if ids.is_empty() {
            return Err(ModelError::Preprocessing(
                "Tokenization produced no tokens".to_string(),
            ));
        }

        // Filter out UNK tokens (typically id=0 for WordPiece) and out-of-range
        let valid_ids: Vec<u32> = ids
            .iter()
            .copied()
            .filter(|&id| (id as usize) < self.vocab_size)
            .collect();

        if valid_ids.is_empty() {
            return Err(ModelError::Preprocessing(
                "No valid tokens after filtering".to_string(),
            ));
        }

        // Lookup embeddings and mean pool
        let mut result = vec![0.0f32; self.hidden_dim];
        let count = valid_ids.len() as f32;

        for id in &valid_ids {
            let offset = (*id as usize) * self.hidden_dim;
            for (i, val) in result.iter_mut().enumerate() {
                *val += self.embeddings[offset + i];
            }
        }

        // Mean pool
        for val in result.iter_mut() {
            *val /= count;
        }

        // L2 normalize if configured
        if self.normalize {
            let norm: f32 = result.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm > 0.0 {
                for val in result.iter_mut() {
                    *val /= norm;
                }
            }
        }

        Ok(result)
    }
}

/// Embed text into a 256-dimensional vector representation
///
/// Uses the Model2Vec potion-base-8M model to generate embeddings.
/// The model is loaded from the `models/` directory (or `ELID_MODELS_DIR`)
/// on first call and cached for subsequent calls.
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
/// Returns `ModelError::ModelLoad` if model files are not found.
/// Returns `ModelError::Preprocessing` if tokenization fails.
///
/// # Example
///
/// ```rust,ignore
/// use elid::models::embed_text;
///
/// let embedding = embed_text("Hello, world!")?;
/// assert_eq!(embedding.len(), 256);
/// ```
pub fn embed_text(text: &str) -> Result<Vec<f32>, ModelError> {
    let model = TEXT_MODEL
        .get_or_init(|| TextModel::load_from_dir(&models_dir()).map_err(|e| e.to_string()));

    match model {
        Ok(m) => m.embed(text),
        Err(e) => Err(ModelError::ModelLoad(e.clone())),
    }
}

/// Embed text using model data provided as raw bytes
///
/// This is the WASM-compatible variant — pass the model files as byte slices
/// instead of relying on filesystem access.
///
/// # Arguments
///
/// * `text` - The input text to embed
/// * `safetensors_bytes` - Contents of `potion-base-8m.safetensors`
/// * `tokenizer_json` - Contents of `potion-base-8m-tokenizer.json`
/// * `config_json` - Contents of `potion-base-8m-config.json`
///
/// # Returns
///
/// A 256-dimensional embedding vector as `Vec<f32>`
///
/// # Example
///
/// ```rust,ignore
/// use elid::models::embed_text_from_bytes;
///
/// let safetensors = include_bytes!("../../models/potion-base-8m.safetensors");
/// let tokenizer = include_bytes!("../../models/potion-base-8m-tokenizer.json");
/// let config = include_bytes!("../../models/potion-base-8m-config.json");
///
/// let embedding = embed_text_from_bytes("Hello!", safetensors, tokenizer, config)?;
/// assert_eq!(embedding.len(), 256);
/// ```
pub fn embed_text_from_bytes(
    text: &str,
    safetensors_bytes: &[u8],
    tokenizer_json: &[u8],
    config_json: &[u8],
) -> Result<Vec<f32>, ModelError> {
    let model = TextModel::load_from_bytes(safetensors_bytes, tokenizer_json, config_json)?;
    model.embed(text)
}

/// Initialize the text model by fetching from GitHub Releases (WASM version).
///
/// Downloads model files from the ELID GitHub release matching the current
/// library version, then caches the loaded model for subsequent `embed_text_cached` calls.
///
/// This is idempotent — calling it multiple times is safe and returns immediately
/// after the first successful load.
#[cfg(target_arch = "wasm32")]
pub async fn init_text_model() -> Result<(), ModelError> {
    if REMOTE_TEXT_MODEL.get().is_some() {
        return Ok(());
    }

    use super::model_url;

    let safetensors = super::fetch_url(&model_url("potion-base-8m.safetensors")).await?;
    let tokenizer = super::fetch_url(&model_url("potion-base-8m-tokenizer.json")).await?;
    let config = super::fetch_url(&model_url("potion-base-8m-config.json")).await?;

    let model = TextModel::load_from_bytes(&safetensors, &tokenizer, &config)?;
    let _ = REMOTE_TEXT_MODEL.set(model);
    Ok(())
}

/// Initialize the text model by fetching from GitHub Releases (native blocking version).
///
/// Downloads model files from the ELID GitHub release matching the current
/// library version, then caches the loaded model.
#[cfg(all(not(target_arch = "wasm32"), feature = "models-fetch"))]
pub fn init_text_model_blocking() -> Result<(), ModelError> {
    if REMOTE_TEXT_MODEL.get().is_some() {
        return Ok(());
    }

    use super::{fetch_url_blocking, model_url};

    let safetensors = fetch_url_blocking(&model_url("potion-base-8m.safetensors"))?;
    let tokenizer = fetch_url_blocking(&model_url("potion-base-8m-tokenizer.json"))?;
    let config = fetch_url_blocking(&model_url("potion-base-8m-config.json"))?;

    let model = TextModel::load_from_bytes(&safetensors, &tokenizer, &config)?;
    let _ = REMOTE_TEXT_MODEL.set(model);
    Ok(())
}

/// Embed text using the remotely-loaded model (call init_text_model first).
///
/// Falls back to the file-based model if available.
pub fn embed_text_cached(text: &str) -> Result<Vec<f32>, ModelError> {
    // Try remote model first
    if let Some(model) = REMOTE_TEXT_MODEL.get() {
        return model.embed(text);
    }
    // Fall back to file-based model
    embed_text(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embed_text_returns_error_without_model() {
        // Set a nonexistent path to ensure model loading fails
        std::env::set_var("ELID_MODELS_DIR", "/nonexistent/path");
        // Note: OnceLock means this test must run before any successful load
        // In practice, CI won't have models downloaded
        let result = embed_text_from_bytes("Hello", &[], &[], &[]);
        assert!(result.is_err());
    }

    #[test]
    #[ignore] // Run with: cargo test --features models-text -- --ignored
    fn test_embed_text_produces_256_dim() {
        let result = embed_text("Hello, world!").unwrap();
        assert_eq!(result.len(), 256);
    }

    #[test]
    #[ignore]
    fn test_embed_text_similar_texts_close() {
        let e1 = embed_text("The quick brown fox").unwrap();
        let e2 = embed_text("The fast brown fox").unwrap();
        let cosine: f32 = e1.iter().zip(e2.iter()).map(|(a, b)| a * b).sum();
        assert!(
            cosine > 0.8,
            "Expected cosine > 0.8 for similar texts, got {cosine}"
        );
    }

    #[test]
    #[ignore]
    fn test_embed_text_different_texts_diverge() {
        let e1 = embed_text("The quick brown fox").unwrap();
        let e2 = embed_text("Quantum physics experiment results").unwrap();
        let cosine: f32 = e1.iter().zip(e2.iter()).map(|(a, b)| a * b).sum();
        assert!(
            cosine < 0.8,
            "Expected cosine < 0.8 for different texts, got {cosine}"
        );
    }
}
