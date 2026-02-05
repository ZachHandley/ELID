//! String similarity algorithms
//!
//! This module provides various algorithms for computing string similarity and distance metrics.
//! All algorithms in this module have zero external dependencies.

mod common;
mod hamming;
mod jaro_winkler;
mod levenshtein;
mod osa;
mod text_simhash;

pub use common::SimilarityOpts;
pub use hamming::{hamming, normalized_hamming};
pub use jaro_winkler::{jaro, jaro_winkler, jaro_winkler_with_prefix};
pub use levenshtein::{levenshtein, levenshtein_with_opts, normalized_levenshtein};
pub use osa::{normalized_osa, osa_distance};
pub use text_simhash::{find_similar_hashes, simhash, simhash_distance, simhash_similarity};
