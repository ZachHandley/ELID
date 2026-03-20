//! Image embedding using MobileNetV3-Small
//!
//! Uses MobileNetV3-Small ONNX model (Qualcomm AI Hub export) for lightweight image
//! feature extraction. The model runs via tract-onnx and produces 1000-dimensional
//! output vectors (ImageNet classification logits, L2-normalized for similarity use).
//!
//! ## Model Details
//!
//! - **Model**: MobileNetV3-Small (Qualcomm AI Hub ONNX export)
//! - **Input**: JPEG or PNG images (resized to 224x224)
//! - **Dimensions**: 1000
//! - **License**: Apache 2.0
//!
//! ## Preprocessing
//!
//! Images are automatically preprocessed:
//! 1. Resize shortest edge to 256px (bicubic)
//! 2. Center crop to 224x224
//! 3. Normalize with ImageNet mean/std
//!
//! ## Example
//!
//! ```rust,ignore
//! use elid::models::embed_image;
//!
//! let bytes = std::fs::read("image.jpg")?;
//! let embedding = embed_image(&bytes)?;
//! assert_eq!(embedding.len(), 1000);
//! ```

use super::error::ModelError;
use super::models_dir;
use std::sync::OnceLock;
use tract_onnx::prelude::*;

/// ImageNet normalization mean (RGB)
const IMAGENET_MEAN: [f32; 3] = [0.485, 0.456, 0.406];
/// ImageNet normalization std (RGB)
const IMAGENET_STD: [f32; 3] = [0.229, 0.224, 0.225];
/// Model input size
const INPUT_SIZE: u32 = 224;
/// Resize shortest edge to this before cropping
const RESIZE_SIZE: u32 = 256;

/// Cached image model (file-based loading)
static IMAGE_MODEL: OnceLock<Result<ImageModel, String>> = OnceLock::new();

/// Cached image model loaded via remote fetch
static REMOTE_IMAGE_MODEL: OnceLock<ImageModel> = OnceLock::new();

/// MobileNetV3-Small image embedding model
pub(crate) struct ImageModel {
    model: TypedRunnableModel<TypedModel>,
}

impl ImageModel {
    /// Load from an ONNX file path
    fn load_from_path(path: &std::path::Path) -> Result<Self, ModelError> {
        if !path.exists() {
            return Err(ModelError::ModelLoad(format!(
                "Model file not found: {}. Run: python scripts/download_models.py --image-only",
                path.display()
            )));
        }

        let model = Self::build_model(tract_onnx::onnx().model_for_path(path)?)?;

        Ok(Self { model })
    }

    /// Load from raw ONNX bytes (for WASM)
    pub(crate) fn load_from_bytes(onnx_bytes: &[u8]) -> Result<Self, ModelError> {
        let mut cursor = std::io::Cursor::new(onnx_bytes);
        let model = Self::build_model(tract_onnx::onnx().model_for_read(&mut cursor)?)?;

        Ok(Self { model })
    }

    /// Build a runnable model from an InferenceModel, setting fixed input shape
    fn build_model(
        inference_model: tract_onnx::prelude::InferenceModel,
    ) -> Result<TypedRunnableModel<TypedModel>, ModelError> {
        let dim = INPUT_SIZE as i64;
        let model = inference_model
            .with_input_fact(
                0,
                InferenceFact::dt_shape(f32::datum_type(), tvec!(1, 3, dim, dim)),
            )?
            .into_typed()?
            .into_runnable()?;
        Ok(model)
    }

    /// Preprocess image bytes into a normalized input tensor
    fn preprocess(image_bytes: &[u8]) -> Result<Tensor, ModelError> {
        let img = image::load_from_memory(image_bytes)?;
        let rgb = img.to_rgb8();

        // Resize: shortest edge to RESIZE_SIZE, maintain aspect ratio
        let (w, h) = (rgb.width(), rgb.height());
        let (new_w, new_h) = if w < h {
            (
                RESIZE_SIZE,
                (RESIZE_SIZE as f64 * h as f64 / w as f64) as u32,
            )
        } else {
            (
                (RESIZE_SIZE as f64 * w as f64 / h as f64) as u32,
                RESIZE_SIZE,
            )
        };

        let resized = image::imageops::resize(
            &rgb,
            new_w,
            new_h,
            image::imageops::FilterType::CatmullRom, // bicubic
        );

        // Center crop to INPUT_SIZE x INPUT_SIZE
        let crop_x = (new_w.saturating_sub(INPUT_SIZE)) / 2;
        let crop_y = (new_h.saturating_sub(INPUT_SIZE)) / 2;

        // Build NCHW tensor [1, 3, 224, 224]
        let mut data = vec![0.0f32; 3 * INPUT_SIZE as usize * INPUT_SIZE as usize];

        for y in 0..INPUT_SIZE {
            for x in 0..INPUT_SIZE {
                let px = resized.get_pixel(crop_x + x, crop_y + y);
                for c in 0..3usize {
                    let val = px[c] as f32 / 255.0;
                    let normalized = (val - IMAGENET_MEAN[c]) / IMAGENET_STD[c];
                    let idx = c * (INPUT_SIZE as usize * INPUT_SIZE as usize)
                        + y as usize * INPUT_SIZE as usize
                        + x as usize;
                    data[idx] = normalized;
                }
            }
        }

        let tensor = tract_ndarray::Array4::from_shape_vec(
            (1, 3, INPUT_SIZE as usize, INPUT_SIZE as usize),
            data,
        )
        .map_err(|e| ModelError::Preprocessing(format!("Tensor creation failed: {e}")))?;

        Ok(tensor.into())
    }

    /// Run inference on preprocessed input
    fn infer(&self, input: Tensor) -> Result<Vec<f32>, ModelError> {
        let result = self.model.run(tvec!(input.into()))?;

        let output = result[0]
            .to_array_view::<f32>()
            .map_err(|e| ModelError::Inference(format!("Output extraction failed: {e}")))?;

        let mut embedding: Vec<f32> = output.iter().copied().collect();

        // L2 normalize for consistent similarity comparisons
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for val in embedding.iter_mut() {
                *val /= norm;
            }
        }

        Ok(embedding)
    }

    /// Full pipeline: preprocess + infer
    pub(crate) fn embed(&self, image_bytes: &[u8]) -> Result<Vec<f32>, ModelError> {
        let input = Self::preprocess(image_bytes)?;
        self.infer(input)
    }
}

/// Embed an image into a 1000-dimensional vector representation
///
/// Uses MobileNetV3-Small to generate a normalized feature vector from the
/// input image. The model is loaded from the `models/` directory (or
/// `ELID_MODELS_DIR`) on first call and cached for subsequent calls.
///
/// # Arguments
///
/// * `image_bytes` - Raw image bytes (JPEG or PNG format)
///
/// # Returns
///
/// A 1000-dimensional L2-normalized feature vector as `Vec<f32>`
///
/// # Errors
///
/// Returns `ModelError::ModelLoad` if the model file is not found.
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
/// assert_eq!(embedding.len(), 1000);
/// ```
pub fn embed_image(image_bytes: &[u8]) -> Result<Vec<f32>, ModelError> {
    let model = IMAGE_MODEL.get_or_init(|| {
        let path = models_dir().join("mobilenetv3-small.onnx");
        ImageModel::load_from_path(&path).map_err(|e| e.to_string())
    });

    match model {
        Ok(m) => m.embed(image_bytes),
        Err(e) => Err(ModelError::ModelLoad(e.clone())),
    }
}

/// Embed an image using model data provided as raw bytes
///
/// This is the WASM-compatible variant — pass the ONNX model as bytes
/// instead of relying on filesystem access.
///
/// # Arguments
///
/// * `image_bytes` - Raw image bytes (JPEG or PNG format)
/// * `model_onnx` - Contents of `mobilenetv3-small.onnx`
///
/// # Returns
///
/// A 1000-dimensional L2-normalized feature vector as `Vec<f32>`
///
/// # Example
///
/// ```rust,ignore
/// use elid::models::embed_image_from_bytes;
///
/// let image = include_bytes!("../../test_image.jpg");
/// let model = include_bytes!("../../models/mobilenetv3-small.onnx");
///
/// let embedding = embed_image_from_bytes(image, model)?;
/// assert_eq!(embedding.len(), 1000);
/// ```
pub fn embed_image_from_bytes(
    image_bytes: &[u8],
    model_onnx: &[u8],
) -> Result<Vec<f32>, ModelError> {
    let model = ImageModel::load_from_bytes(model_onnx)?;
    model.embed(image_bytes)
}

/// Initialize the image model by fetching from GitHub Releases (WASM version).
#[cfg(target_arch = "wasm32")]
pub async fn init_image_model() -> Result<(), ModelError> {
    if REMOTE_IMAGE_MODEL.get().is_some() {
        return Ok(());
    }

    let onnx_bytes = super::fetch_url(&super::model_url("mobilenetv3-small.onnx")).await?;
    let model = ImageModel::load_from_bytes(&onnx_bytes)?;
    let _ = REMOTE_IMAGE_MODEL.set(model);
    Ok(())
}

/// Initialize the image model by fetching from GitHub Releases (native blocking version).
#[cfg(all(not(target_arch = "wasm32"), feature = "models-fetch"))]
pub fn init_image_model_blocking() -> Result<(), ModelError> {
    if REMOTE_IMAGE_MODEL.get().is_some() {
        return Ok(());
    }

    let onnx_bytes = super::fetch_url_blocking(&super::model_url("mobilenetv3-small.onnx"))?;
    let model = ImageModel::load_from_bytes(&onnx_bytes)?;
    let _ = REMOTE_IMAGE_MODEL.set(model);
    Ok(())
}

/// Embed image using the remotely-loaded model (call init_image_model first).
///
/// Falls back to the file-based model if available.
pub fn embed_image_cached(image_bytes: &[u8]) -> Result<Vec<f32>, ModelError> {
    if let Some(model) = REMOTE_IMAGE_MODEL.get() {
        return model.embed(image_bytes);
    }
    embed_image(image_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embed_image_returns_error_without_model() {
        let result = embed_image_from_bytes(&[0u8; 100], &[]);
        assert!(result.is_err());
    }

    #[test]
    #[ignore] // Run with: cargo test --features models-image -- --ignored
    fn test_embed_image_produces_1000_dim() {
        let bytes = std::fs::read("tests/fixtures/test_image.jpg")
            .expect("Test image not found at tests/fixtures/test_image.jpg");
        let result = embed_image(&bytes).unwrap();
        assert_eq!(result.len(), 1000);
    }

    #[test]
    #[ignore]
    fn test_embed_image_is_normalized() {
        let bytes = std::fs::read("tests/fixtures/test_image.jpg")
            .expect("Test image not found at tests/fixtures/test_image.jpg");
        let result = embed_image(&bytes).unwrap();
        let norm: f32 = result.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!(
            (norm - 1.0).abs() < 0.01,
            "Expected L2 norm ~1.0, got {norm}"
        );
    }
}
