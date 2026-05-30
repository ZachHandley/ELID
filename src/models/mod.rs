//! Embedding model support for text and images
//!
//! This module provides local embedding models for generating vector representations
//! of text and images. Text uses Model2Vec (safetensors + tokenizer), image uses
//! MobileNetV3 (ONNX inference via tract).
//!
//! ## Features
//!
//! - `models`: Base support (error types, path resolution)
//! - `models-text`: Text embedding using Model2Vec (potion-base-8M, 256-dim)
//! - `models-image`: Image embedding using MobileNetV3-Small (1000-dim)
//!
//! ## Example
//!
//! ```rust,ignore
//! use elid::models::{embed_text, embed_image};
//!
//! // Text embedding (256-dim)
//! let text_embedding = embed_text("Hello, world!")?;
//! assert_eq!(text_embedding.len(), 256);
//!
//! // Image embedding (1000-dim)
//! let image_bytes = std::fs::read("image.jpg")?;
//! let image_embedding = embed_image(&image_bytes)?;
//! assert_eq!(image_embedding.len(), 1000);
//! ```

mod error;

#[cfg(feature = "models-text")]
pub mod text;

#[cfg(feature = "models-image")]
pub mod image;

pub use error::ModelError;

#[cfg(feature = "models-text")]
pub use text::embed_text;

#[cfg(feature = "models-text")]
pub use text::embed_text_from_bytes;

#[cfg(feature = "models-image")]
pub use image::embed_image;

#[cfg(feature = "models-image")]
pub use image::embed_image_from_bytes;

#[cfg(feature = "models-text")]
pub use text::embed_text_cached;

#[cfg(feature = "models-image")]
pub use image::embed_image_cached;

/// Dimensionality of text embeddings (Model2Vec potion-base-8M)
pub const TEXT_EMBEDDING_DIM: usize = 256;

/// Dimensionality of image embeddings (MobileNetV3-Small classification logits)
pub const IMAGE_EMBEDDING_DIM: usize = 1000;

/// Base URL for model downloads from GitHub Releases
pub const MODEL_BASE_URL: &str = "https://github.com/ZachHandley/ELID/releases/download";

/// Current version used for model download URLs
pub const MODEL_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Get the download URL for a model file at the current version.
///
/// At build time, setting `ELID_MODEL_BASE_URL` redirects fetches to that prefix
/// (filename is appended as-is, no version path inserted). This lets the demo
/// site self-host the model files instead of relying on a GitHub release —
/// useful when no published release matches `CARGO_PKG_VERSION` yet.
pub fn model_url(filename: &str) -> String {
    match option_env!("ELID_MODEL_BASE_URL") {
        Some(base) => format!("{base}/{filename}"),
        None => format!("{MODEL_BASE_URL}/v{MODEL_VERSION}/{filename}"),
    }
}

/// Resolve the models directory path.
///
/// Checks `ELID_MODELS_DIR` env var first, then falls back to `models/`
/// relative to the current directory.
pub fn models_dir() -> std::path::PathBuf {
    if let Ok(dir) = std::env::var("ELID_MODELS_DIR") {
        std::path::PathBuf::from(dir)
    } else {
        std::path::PathBuf::from("models")
    }
}

/// Fetch bytes from a URL (WASM version using gloo-net)
#[cfg(target_arch = "wasm32")]
pub async fn fetch_url(url: &str) -> Result<Vec<u8>, ModelError> {
    use gloo_net::http::Request;
    let resp = Request::get(url)
        .send()
        .await
        .map_err(|e| ModelError::ModelLoad(format!("Fetch failed for {url}: {e}")))?;
    if !resp.ok() {
        return Err(ModelError::ModelLoad(format!(
            "HTTP {} fetching {url}",
            resp.status()
        )));
    }
    resp.binary()
        .await
        .map_err(|e| ModelError::ModelLoad(format!("Read failed for {url}: {e}")))
}

/// Fetch bytes from a URL (native version using ureq)
#[cfg(all(not(target_arch = "wasm32"), feature = "models-fetch"))]
pub fn fetch_url_blocking(url: &str) -> Result<Vec<u8>, ModelError> {
    let resp = ureq::get(url)
        .call()
        .map_err(|e| ModelError::ModelLoad(format!("Fetch failed for {url}: {e}")))?;
    resp.into_body()
        .read_to_vec()
        .map_err(|e| ModelError::ModelLoad(format!("Read failed for {url}: {e}")))
}
