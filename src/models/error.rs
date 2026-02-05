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
            Self::ModelLoad(msg) => write!(f, "Model load error: {}", msg),
            Self::Preprocessing(msg) => write!(f, "Preprocessing error: {}", msg),
            Self::Inference(msg) => write!(f, "Inference error: {}", msg),
            Self::Postprocessing(msg) => write!(f, "Postprocessing error: {}", msg),
        }
    }
}

impl std::error::Error for ModelError {}
