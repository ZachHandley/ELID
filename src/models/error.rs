//! Error types for model operations

use std::fmt;

/// Errors that can occur during model inference
#[derive(Debug)]
pub enum ModelError {
    /// Model file not found or failed to load
    ModelLoad(String),
    /// Input preprocessing failed
    Preprocessing(String),
    /// Inference failed
    Inference(String),
    /// Output postprocessing failed
    Postprocessing(String),
}

impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ModelLoad(msg) => write!(f, "Model load error: {msg}"),
            Self::Preprocessing(msg) => write!(f, "Preprocessing error: {msg}"),
            Self::Inference(msg) => write!(f, "Inference error: {msg}"),
            Self::Postprocessing(msg) => write!(f, "Postprocessing error: {msg}"),
        }
    }
}

impl std::error::Error for ModelError {}

impl From<std::io::Error> for ModelError {
    fn from(e: std::io::Error) -> Self {
        Self::ModelLoad(e.to_string())
    }
}

#[cfg(feature = "models-text")]
impl From<tokenizers::Error> for ModelError {
    fn from(e: tokenizers::Error) -> Self {
        Self::Preprocessing(e.to_string())
    }
}

#[cfg(feature = "models-text")]
impl From<safetensors::SafeTensorError> for ModelError {
    fn from(e: safetensors::SafeTensorError) -> Self {
        Self::ModelLoad(e.to_string())
    }
}

#[cfg(feature = "models-text")]
impl From<serde_json::Error> for ModelError {
    fn from(e: serde_json::Error) -> Self {
        Self::ModelLoad(format!("Config parse error: {e}"))
    }
}

#[cfg(feature = "models-image")]
impl From<tract_onnx::prelude::TractError> for ModelError {
    fn from(e: tract_onnx::prelude::TractError) -> Self {
        Self::Inference(e.to_string())
    }
}

#[cfg(feature = "models-image")]
impl From<image::ImageError> for ModelError {
    fn from(e: image::ImageError) -> Self {
        Self::Preprocessing(e.to_string())
    }
}
