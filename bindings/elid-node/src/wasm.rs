//! WASM bindings for ELID using wasm-bindgen
//!
//! This module provides browser-compatible WASM bindings as a fallback
//! when native Node.js addons are not available.

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use elid_core::{decode, encode, hamming_distance, Elid, Profile};

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub enum WasmProfile {
  Mini128 = 0,
  Morton10x10 = 1,
  Hilbert10x10 = 2,
}

#[cfg(target_arch = "wasm32")]
impl From<WasmProfile> for Profile {
  fn from(profile: WasmProfile) -> Self {
    match profile {
      WasmProfile::Mini128 => Profile::Mini128 { seed: 0 },
      WasmProfile::Morton10x10 => Profile::Morton10x10 {
        dims: 10,
        bits_per_dim: 10,
        transform_id: None,
      },
      WasmProfile::Hilbert10x10 => Profile::Hilbert10x10 {
        dims: 10,
        bits_per_dim: 10,
        transform_id: None,
      },
    }
  }
}

/// Encode a high-dimensional embedding to a sortable string identifier (WASM version).
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_name = encodeElid)]
pub fn encode_elid_wasm(embedding: &[f64], profile: WasmProfile) -> Result<String, JsValue> {
  // Convert f64 to f32
  let embedding_f32: Vec<f32> = embedding.iter().map(|&x| x as f32).collect();

  let rust_profile: Profile = profile.into();

  encode(&embedding_f32, &rust_profile)
    .map(|elid| elid.to_string())
    .map_err(|e| JsValue::from_str(&format!("Encoding failed: {}", e)))
}

/// Decode an ELID string back to raw bytes (WASM version).
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_name = decodeElid)]
pub fn decode_elid_wasm(elid: String) -> Result<Vec<u8>, JsValue> {
  let elid_obj = Elid::from_string(elid)
    .map_err(|e| JsValue::from_str(&format!("Invalid ELID: {}", e)))?;

  decode(&elid_obj)
    .map_err(|e| JsValue::from_str(&format!("Decoding failed: {}", e)))
}

/// Calculate Hamming distance between two ELIDs (WASM version).
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_name = hammingDistanceElid)]
pub fn hamming_distance_elid_wasm(elid1: String, elid2: String) -> Result<u32, JsValue> {
  let elid_obj1 = Elid::from_string(elid1)
    .map_err(|e| JsValue::from_str(&format!("Invalid ELID 1: {}", e)))?;
  let elid_obj2 = Elid::from_string(elid2)
    .map_err(|e| JsValue::from_str(&format!("Invalid ELID 2: {}", e)))?;

  hamming_distance(&elid_obj1, &elid_obj2)
    .map_err(|e| JsValue::from_str(&format!("Hamming distance calculation failed: {}", e)))
}
