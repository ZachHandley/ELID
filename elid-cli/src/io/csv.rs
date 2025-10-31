use anyhow::{bail, Context, Result};
use csv::{ReaderBuilder, WriterBuilder};
use std::fs::File;
use std::io::{stdin, stdout, BufReader, BufWriter};

/// Read embeddings from a CSV file.
///
/// # Arguments
/// * `path` - File path to read from, or "-" for stdin
/// * `column` - Name of the column containing embeddings (expects JSON array format)
///
/// # Returns
/// Vector of embeddings, where each embedding is a Vec<f32>
///
/// # Errors
/// Returns an error if:
/// - File cannot be opened or read
/// - CSV format is invalid
/// - Specified column is missing
/// - Embedding values cannot be parsed as JSON arrays
pub fn read_csv(path: &str, column: &str) -> Result<Vec<Vec<f32>>> {
    let mut embeddings = Vec::new();

    // Handle stdin or file input
    if path == "-" {
        let reader = BufReader::new(stdin());
        let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);

        parse_csv_embeddings(&mut csv_reader, column, &mut embeddings)?;
    } else {
        let file =
            File::open(path).with_context(|| format!("Failed to open CSV file: {}", path))?;
        let reader = BufReader::new(file);
        let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);

        parse_csv_embeddings(&mut csv_reader, column, &mut embeddings)?;
    }

    Ok(embeddings)
}

/// Helper function to parse embeddings from a CSV reader.
fn parse_csv_embeddings<R: std::io::Read>(
    csv_reader: &mut csv::Reader<R>,
    column: &str,
    embeddings: &mut Vec<Vec<f32>>,
) -> Result<()> {
    // Get headers to find the column index
    let headers = csv_reader.headers().context("Failed to read CSV headers")?;

    let column_index = headers
        .iter()
        .position(|h| h == column)
        .with_context(|| format!("Column '{}' not found in CSV headers", column))?;

    // Process each record
    for (row_num, result) in csv_reader.records().enumerate() {
        let record =
            result.with_context(|| format!("Failed to read CSV record at row {}", row_num + 1))?;

        let embedding_str = record
            .get(column_index)
            .with_context(|| format!("Missing column '{}' at row {}", column, row_num + 1))?;

        // Parse JSON array format: "[0.1, 0.2, 0.3, ...]"
        let embedding: Vec<f32> = serde_json::from_str(embedding_str)
            .with_context(|| format!(
                "Failed to parse embedding as JSON array at row {}. Expected format: [0.1, 0.2, ...]",
                row_num + 1
            ))?;

        if embedding.is_empty() {
            bail!("Empty embedding at row {}", row_num + 1);
        }

        embeddings.push(embedding);
    }

    Ok(())
}

/// Write embeddings and ELIDs to a CSV file.
///
/// # Arguments
/// * `path` - File path to write to, or "-" for stdout
/// * `embeddings` - Original embeddings to write
/// * `elids` - Corresponding ELID strings
/// * `column_name` - Name for the embedding column (ELID column will be "elid")
///
/// # Errors
/// Returns an error if:
/// - File cannot be created or written
/// - embeddings and elids have different lengths
/// - CSV writing fails
pub fn write_csv(
    path: &str,
    embeddings: &[Vec<f32>],
    elids: &[String],
    column_name: &str,
) -> Result<()> {
    if embeddings.len() != elids.len() {
        bail!(
            "Embeddings and ELIDs length mismatch: {} vs {}",
            embeddings.len(),
            elids.len()
        );
    }

    // Handle stdout or file output
    if path == "-" {
        let writer = BufWriter::new(stdout());
        let mut csv_writer = WriterBuilder::new().has_headers(true).from_writer(writer);

        write_csv_records(&mut csv_writer, embeddings, elids, column_name)?;
    } else {
        let file =
            File::create(path).with_context(|| format!("Failed to create CSV file: {}", path))?;
        let writer = BufWriter::new(file);
        let mut csv_writer = WriterBuilder::new().has_headers(true).from_writer(writer);

        write_csv_records(&mut csv_writer, embeddings, elids, column_name)?;
    }

    Ok(())
}

/// Helper function to write CSV records.
fn write_csv_records<W: std::io::Write>(
    csv_writer: &mut csv::Writer<W>,
    embeddings: &[Vec<f32>],
    elids: &[String],
    column_name: &str,
) -> Result<()> {
    // Write headers
    csv_writer
        .write_record([column_name, "elid"])
        .context("Failed to write CSV headers")?;

    // Write records row-by-row (streaming)
    for (embedding, elid) in embeddings.iter().zip(elids.iter()) {
        // Serialize embedding as JSON array
        let embedding_json =
            serde_json::to_string(embedding).context("Failed to serialize embedding to JSON")?;

        csv_writer
            .write_record([&embedding_json, elid])
            .context("Failed to write CSV record")?;
    }

    csv_writer.flush().context("Failed to flush CSV writer")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_round_trip() -> Result<()> {
        // Create a temporary CSV file with embeddings using proper CSV quoting
        let mut temp_input = NamedTempFile::new()?;
        writeln!(temp_input, "embedding,metadata")?;
        writeln!(temp_input, r#""[0.1, 0.2, 0.3]",data1"#)?;
        writeln!(temp_input, r#""[0.4, 0.5, 0.6]",data2"#)?;
        writeln!(temp_input, r#""[0.7, 0.8, 0.9]",data3"#)?;
        temp_input.flush()?;

        // Read embeddings
        let embeddings = read_csv(temp_input.path().to_str().unwrap(), "embedding")?;

        assert_eq!(embeddings.len(), 3);
        assert_eq!(embeddings[0], vec![0.1, 0.2, 0.3]);
        assert_eq!(embeddings[1], vec![0.4, 0.5, 0.6]);
        assert_eq!(embeddings[2], vec![0.7, 0.8, 0.9]);

        // Create mock ELIDs
        let elids = vec![
            "ELID001".to_string(),
            "ELID002".to_string(),
            "ELID003".to_string(),
        ];

        // Write to a new temporary file
        let temp_output = NamedTempFile::new()?;
        let output_path = temp_output.path().to_str().unwrap();

        write_csv(output_path, &embeddings, &elids, "embedding")?;

        // Read back the written file
        let read_back_embeddings = read_csv(output_path, "embedding")?;

        // Verify round-trip
        assert_eq!(read_back_embeddings.len(), embeddings.len());
        for (original, read_back) in embeddings.iter().zip(read_back_embeddings.iter()) {
            assert_eq!(original, read_back);
        }

        // Verify ELIDs were written (read the file manually to check)
        let content = std::fs::read_to_string(output_path)?;
        assert!(content.contains("ELID001"));
        assert!(content.contains("ELID002"));
        assert!(content.contains("ELID003"));

        Ok(())
    }

    #[test]
    fn test_missing_column() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "wrong_column").unwrap();
        writeln!(temp_file, r#"[0.1, 0.2]"#).unwrap();
        temp_file.flush().unwrap();

        let result = read_csv(temp_file.path().to_str().unwrap(), "embedding");

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_invalid_json_format() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "embedding").unwrap();
        writeln!(temp_file, "not_a_json_array").unwrap();
        temp_file.flush().unwrap();

        let result = read_csv(temp_file.path().to_str().unwrap(), "embedding");

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("parse"));
    }

    #[test]
    fn test_length_mismatch() {
        let embeddings = vec![vec![0.1, 0.2], vec![0.3, 0.4]];
        let elids = vec!["ELID001".to_string()]; // Only 1 ELID for 2 embeddings

        let temp_file = NamedTempFile::new().unwrap();
        let result = write_csv(
            temp_file.path().to_str().unwrap(),
            &embeddings,
            &elids,
            "embedding",
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("mismatch"));
    }
}
