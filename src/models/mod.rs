//! Embedding model support for text and images
//!
//! This module provides ONNX-based embedding models that can run in WASM.
//! Models are loaded from files and run inference using tract-onnx.
//!
//! ## Features
//!
//! - `models`: Base ONNX model support (tract-onnx dependency)
//! - `models-text`: Text embedding using Model2Vec (potion-base-8M)
//! - `models-image`: Image embedding using MobileNetV3-Small
//!
//! ## Example
//!
//! ```rust,ignore
//! use elid::models::{embed_text, embed_image};
//!
//! // Text embedding
//! let text_embedding = embed_text("Hello, world!")?;
//! assert_eq!(text_embedding.len(), 256);
//!
//! // Image embedding
//! let image_bytes = std::fs::read("image.jpg")?;
//! let image_embedding = embed_image(&image_bytes)?;
//! assert_eq!(image_embedding.len(), 1024);
//! ```

mod error;

#[cfg(feature = "models-text")]
pub mod text;

#[cfg(feature = "models-image")]
pub mod image;

pub use error::ModelError;

#[cfg(feature = "models-text")]
pub use text::embed_text;

#[cfg(feature = "models-image")]
pub use image::embed_image;
