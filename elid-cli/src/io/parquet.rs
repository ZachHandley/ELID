//! Parquet I/O module for reading and writing embeddings with Arrow integration.
//!
//! This module provides functions to read and write embedding vectors from/to Parquet files
//! using Apache Arrow's columnar format for efficient storage and retrieval.

use anyhow::{anyhow, Context, Result};
use arrow::array::{Array, ArrayRef, Float32Array, ListArray, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::arrow::ArrowWriter;
use parquet::file::properties::WriterProperties;
use std::fs::File;
use std::sync::Arc;

/// Read embeddings from a Parquet file.
///
/// # Arguments
///
/// * `path` - Path to the Parquet file
/// * `column` - Name of the column containing embedding arrays
///
/// # Returns
///
/// Vector of embeddings, where each embedding is a `Vec<f32>`
///
/// # Errors
///
/// Returns error if:
/// - File cannot be opened
/// - Column doesn't exist
/// - Column is not a ListArray of Float32
/// - Data conversion fails
///
/// # Example
///
/// ```no_run
/// use elid_cli::io::parquet::read_parquet;
///
/// let embeddings = read_parquet("embeddings.parquet", "embedding")?;
/// println!("Read {} embeddings", embeddings.len());
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn read_parquet(path: &str, column: &str) -> Result<Vec<Vec<f32>>> {
    // Open the Parquet file
    let file =
        File::open(path).with_context(|| format!("Failed to open Parquet file: {}", path))?;

    // Create Arrow reader
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)
        .with_context(|| "Failed to create Parquet reader")?;

    let reader = builder
        .build()
        .with_context(|| "Failed to build Parquet record batch reader")?;

    let mut all_embeddings = Vec::new();

    // Read all record batches
    for batch_result in reader {
        let batch = batch_result.with_context(|| "Failed to read record batch")?;

        // Find the column index
        let schema = batch.schema();
        let column_index = schema
            .index_of(column)
            .with_context(|| format!("Column '{}' not found in Parquet file", column))?;

        // Get the column array
        let array = batch.column(column_index);

        // Downcast to ListArray
        let list_array = array
            .as_any()
            .downcast_ref::<ListArray>()
            .with_context(|| {
                format!(
                    "Column '{}' is not a ListArray (found type: {:?})",
                    column,
                    array.data_type()
                )
            })?;

        // Extract embeddings from this batch
        for i in 0..list_array.len() {
            if list_array.is_null(i) {
                return Err(anyhow!(
                    "Null embedding found at index {} in column '{}'",
                    i,
                    column
                ));
            }

            let value_array = list_array.value(i);

            // Downcast to Float32Array
            let float_array = value_array
                .as_any()
                .downcast_ref::<Float32Array>()
                .with_context(|| {
                    format!(
                        "Embedding values are not Float32 (found type: {:?})",
                        value_array.data_type()
                    )
                })?;

            // Convert to Vec<f32>
            let embedding: Vec<f32> = float_array.values().to_vec();

            all_embeddings.push(embedding);
        }
    }

    Ok(all_embeddings)
}

/// Write embeddings and their ELIDs to a Parquet file.
///
/// # Arguments
///
/// * `path` - Output Parquet file path
/// * `embeddings` - Slice of embedding vectors
/// * `elids` - Slice of ELID strings (must match embeddings length)
/// * `column_name` - Name for the embedding column
///
/// # Returns
///
/// `Ok(())` on success
///
/// # Errors
///
/// Returns error if:
/// - embeddings and elids have different lengths
/// - File cannot be created
/// - Schema creation fails
/// - Data writing fails
///
/// # Example
///
/// ```no_run
/// use elid_cli::io::parquet::write_parquet;
///
/// let embeddings = vec![
///     vec![0.1, 0.2, 0.3],
///     vec![0.4, 0.5, 0.6],
/// ];
/// let elids = vec![
///     "ELID_001".to_string(),
///     "ELID_002".to_string(),
/// ];
///
/// write_parquet("output.parquet", &embeddings, &elids, "embedding")?;
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn write_parquet(
    path: &str,
    embeddings: &[Vec<f32>],
    elids: &[String],
    column_name: &str,
) -> Result<()> {
    // Validate input
    if embeddings.len() != elids.len() {
        return Err(anyhow!(
            "Embeddings count ({}) must match ELIDs count ({})",
            embeddings.len(),
            elids.len()
        ));
    }

    if embeddings.is_empty() {
        return Err(anyhow!("Cannot write empty embeddings to Parquet"));
    }

    // Create schema: [elid: String, embedding: List<Float32>]
    let schema = Arc::new(Schema::new(vec![
        Field::new("elid", DataType::Utf8, false),
        Field::new(
            column_name,
            DataType::List(Arc::new(Field::new("item", DataType::Float32, false))),
            false,
        ),
    ]));

    // Create output file
    let file =
        File::create(path).with_context(|| format!("Failed to create Parquet file: {}", path))?;

    // Configure writer properties for optimal compression
    let props = WriterProperties::builder()
        .set_compression(parquet::basic::Compression::SNAPPY)
        .build();

    // Create Arrow writer
    let mut writer = ArrowWriter::try_new(file, schema.clone(), Some(props))
        .with_context(|| "Failed to create Arrow writer")?;

    // Build arrays for the record batch
    let elid_array = StringArray::from(elids.to_vec());

    // Build embedding list array with explicit field configuration
    let field = Arc::new(Field::new("item", DataType::Float32, false));
    let mut list_builder =
        arrow::array::ListBuilder::new(arrow::array::Float32Builder::new()).with_field(field);

    for embedding in embeddings {
        let value_builder = list_builder.values();
        for &value in embedding {
            value_builder.append_value(value);
        }
        list_builder.append(true);
    }

    let embedding_array = list_builder.finish();

    // Create record batch
    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(elid_array) as ArrayRef,
            Arc::new(embedding_array) as ArrayRef,
        ],
    )
    .with_context(|| "Failed to create record batch")?;

    // Write batch
    writer
        .write(&batch)
        .with_context(|| "Failed to write record batch")?;

    // Finalize file
    writer
        .close()
        .with_context(|| "Failed to close Parquet writer")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn get_test_file_path(name: &str) -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("test_data");
        std::fs::create_dir_all(&path).unwrap();
        path.push(name);
        path
    }

    #[test]
    fn test_write_and_read_parquet() {
        let test_file = get_test_file_path("test_embeddings.parquet");
        let path_str = test_file.to_str().unwrap();

        // Test data
        let embeddings = vec![
            vec![0.1, 0.2, 0.3, 0.4],
            vec![0.5, 0.6, 0.7, 0.8],
            vec![0.9, 1.0, 1.1, 1.2],
        ];

        let elids = vec![
            "ELID_001".to_string(),
            "ELID_002".to_string(),
            "ELID_003".to_string(),
        ];

        // Write
        write_parquet(path_str, &embeddings, &elids, "embedding").unwrap();

        // Read back
        let read_embeddings = read_parquet(path_str, "embedding").unwrap();

        // Verify
        assert_eq!(read_embeddings.len(), embeddings.len());
        for (original, read) in embeddings.iter().zip(read_embeddings.iter()) {
            assert_eq!(original.len(), read.len());
            for (o, r) in original.iter().zip(read.iter()) {
                assert!((o - r).abs() < 1e-6);
            }
        }

        // Cleanup
        std::fs::remove_file(test_file).ok();
    }

    #[test]
    fn test_mismatched_lengths() {
        let test_file = get_test_file_path("test_mismatch.parquet");
        let path_str = test_file.to_str().unwrap();

        let embeddings = vec![vec![0.1, 0.2]];
        let elids = vec!["ELID_001".to_string(), "ELID_002".to_string()];

        let result = write_parquet(path_str, &embeddings, &elids, "embedding");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must match ELIDs count"));
    }

    #[test]
    fn test_empty_embeddings() {
        let test_file = get_test_file_path("test_empty.parquet");
        let path_str = test_file.to_str().unwrap();

        let embeddings: Vec<Vec<f32>> = vec![];
        let elids: Vec<String> = vec![];

        let result = write_parquet(path_str, &embeddings, &elids, "embedding");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Cannot write empty embeddings"));
    }

    #[test]
    fn test_varying_dimension_embeddings() {
        let test_file = get_test_file_path("test_varying_dim.parquet");
        let path_str = test_file.to_str().unwrap();

        // Different dimension embeddings (valid in ListArray)
        let embeddings = vec![vec![0.1, 0.2], vec![0.3, 0.4, 0.5], vec![0.6]];

        let elids = vec![
            "ELID_001".to_string(),
            "ELID_002".to_string(),
            "ELID_003".to_string(),
        ];

        // Write should succeed (ListArray supports varying lengths)
        write_parquet(path_str, &embeddings, &elids, "embedding").unwrap();

        // Read back
        let read_embeddings = read_parquet(path_str, "embedding").unwrap();

        // Verify dimensions are preserved
        assert_eq!(read_embeddings[0].len(), 2);
        assert_eq!(read_embeddings[1].len(), 3);
        assert_eq!(read_embeddings[2].len(), 1);

        // Cleanup
        std::fs::remove_file(test_file).ok();
    }

    #[test]
    fn test_read_nonexistent_file() {
        let result = read_parquet("/nonexistent/path/file.parquet", "embedding");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to open Parquet file"));
    }

    #[test]
    fn test_read_nonexistent_column() {
        let test_file = get_test_file_path("test_wrong_column.parquet");
        let path_str = test_file.to_str().unwrap();

        let embeddings = vec![vec![0.1, 0.2]];
        let elids = vec!["ELID_001".to_string()];

        write_parquet(path_str, &embeddings, &elids, "embedding").unwrap();

        let result = read_parquet(path_str, "wrong_column_name");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));

        // Cleanup
        std::fs::remove_file(test_file).ok();
    }
}
