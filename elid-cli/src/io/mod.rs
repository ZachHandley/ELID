//! I/O module for reading and writing embeddings in various formats.

/// CSV input/output functionality.
pub mod csv;

/// JSONL (JSON Lines) input/output functionality.
pub mod jsonl;

/// Parquet input/output functionality using Apache Arrow.
pub mod parquet;
