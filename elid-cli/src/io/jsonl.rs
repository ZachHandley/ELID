use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::fs::File;
use std::io::{stdin, stdout, BufRead, BufReader, BufWriter, Write};

/// Extracts a nested field from a JSON value using dot notation.
///
/// This is an internal helper function used by `read_jsonl`.
fn extract_field<'a>(value: &'a Value, field_path: &str) -> Result<&'a Value> {
    let mut current = value;

    for part in field_path.split('.') {
        current = current
            .get(part)
            .with_context(|| format!("Field '{}' not found in path '{}'", part, field_path))?;
    }

    Ok(current)
}

/// Converts a JSON value to a Vec<f32> embedding.
fn value_to_embedding(value: &Value) -> Result<Vec<f32>> {
    let array = value
        .as_array()
        .with_context(|| "Embedding field is not an array")?;

    let mut embedding = Vec::with_capacity(array.len());

    for (idx, item) in array.iter().enumerate() {
        let num = item
            .as_f64()
            .with_context(|| format!("Element at index {} is not a number", idx))?;
        embedding.push(num as f32);
    }

    Ok(embedding)
}

/// Reads embeddings from a JSONL file.
///
/// # Arguments
///
/// * `path` - File path or "-" for stdin
/// * `field_path` - Dot-notation path to the embedding field (e.g., "data.embedding")
///
/// # Returns
///
/// A vector of embeddings, where each embedding is a Vec<f32>
///
/// # Examples
///
/// ```no_run
/// use elid_cli::io::jsonl::read_jsonl;
///
/// // Read from file
/// let embeddings = read_jsonl("embeddings.jsonl", "embedding").unwrap();
///
/// // Read from stdin
/// let embeddings = read_jsonl("-", "data.embedding").unwrap();
/// ```
pub fn read_jsonl(path: &str, field_path: &str) -> Result<Vec<Vec<f32>>> {
    let reader: Box<dyn BufRead> = if path == "-" {
        Box::new(BufReader::new(stdin()))
    } else {
        let file = File::open(path).with_context(|| format!("Failed to open file: {}", path))?;
        Box::new(BufReader::new(file))
    };

    let mut embeddings = Vec::new();

    for (line_num, line_result) in reader.lines().enumerate() {
        let line = line_result.with_context(|| format!("Failed to read line {}", line_num + 1))?;

        // Skip empty lines
        if line.trim().is_empty() {
            continue;
        }

        let json: Value = serde_json::from_str(&line)
            .with_context(|| format!("Failed to parse JSON at line {}", line_num + 1))?;

        let field_value = extract_field(&json, field_path)
            .with_context(|| format!("At line {}", line_num + 1))?;

        let embedding =
            value_to_embedding(field_value).with_context(|| format!("At line {}", line_num + 1))?;

        embeddings.push(embedding);
    }

    Ok(embeddings)
}

/// Writes ELIDs to a JSONL file.
///
/// # Arguments
///
/// * `path` - File path or "-" for stdout
/// * `elids` - Slice of ELID strings to write
/// * `field_name` - Field name for the ELID in each JSON object
///
/// # Examples
///
/// ```no_run
/// use elid_cli::io::jsonl::write_jsonl;
///
/// let elids = vec!["ELID123".to_string(), "ELID456".to_string()];
///
/// // Write to file
/// write_jsonl("output.jsonl", &elids, "elid").unwrap();
///
/// // Write to stdout
/// write_jsonl("-", &elids, "elid").unwrap();
/// ```
pub fn write_jsonl(path: &str, elids: &[String], field_name: &str) -> Result<()> {
    if field_name.is_empty() {
        bail!("Field name cannot be empty");
    }

    let writer: Box<dyn Write> = if path == "-" {
        Box::new(BufWriter::new(stdout()))
    } else {
        let file =
            File::create(path).with_context(|| format!("Failed to create file: {}", path))?;
        Box::new(BufWriter::new(file))
    };

    let mut writer = writer;

    for (idx, elid) in elids.iter().enumerate() {
        let json_obj = serde_json::json!({
            field_name: elid
        });

        let json_line = serde_json::to_string(&json_obj)
            .with_context(|| format!("Failed to serialize ELID at index {}", idx))?;

        writeln!(writer, "{}", json_line)
            .with_context(|| format!("Failed to write line {} to {}", idx + 1, path))?;
    }

    writer
        .flush()
        .with_context(|| format!("Failed to flush output to {}", path))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_extract_field_simple() {
        let json = serde_json::json!({"embedding": [1.0, 2.0, 3.0]});
        let result = extract_field(&json, "embedding").unwrap();
        assert_eq!(result, &serde_json::json!([1.0, 2.0, 3.0]));
    }

    #[test]
    fn test_extract_field_nested() {
        let json = serde_json::json!({
            "data": {
                "embedding": [1.0, 2.0, 3.0]
            }
        });
        let result = extract_field(&json, "data.embedding").unwrap();
        assert_eq!(result, &serde_json::json!([1.0, 2.0, 3.0]));
    }

    #[test]
    fn test_extract_field_deep_nested() {
        let json = serde_json::json!({
            "response": {
                "data": {
                    "embedding": [1.0, 2.0, 3.0]
                }
            }
        });
        let result = extract_field(&json, "response.data.embedding").unwrap();
        assert_eq!(result, &serde_json::json!([1.0, 2.0, 3.0]));
    }

    #[test]
    fn test_extract_field_not_found() {
        let json = serde_json::json!({"embedding": [1.0, 2.0, 3.0]});
        let result = extract_field(&json, "nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_value_to_embedding() {
        let value = serde_json::json!([1.0, 2.5, 3.7]);
        let embedding = value_to_embedding(&value).unwrap();
        assert_eq!(embedding, vec![1.0, 2.5, 3.7]);
    }

    #[test]
    fn test_value_to_embedding_integers() {
        let value = serde_json::json!([1, 2, 3]);
        let embedding = value_to_embedding(&value).unwrap();
        assert_eq!(embedding, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_value_to_embedding_not_array() {
        let value = serde_json::json!({"not": "array"});
        let result = value_to_embedding(&value);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_jsonl_simple() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"{{"embedding": [1.0, 2.0, 3.0]}}"#).unwrap();
        writeln!(temp_file, r#"{{"embedding": [4.0, 5.0, 6.0]}}"#).unwrap();
        temp_file.flush().unwrap();

        let embeddings = read_jsonl(temp_file.path().to_str().unwrap(), "embedding").unwrap();

        assert_eq!(embeddings.len(), 2);
        assert_eq!(embeddings[0], vec![1.0, 2.0, 3.0]);
        assert_eq!(embeddings[1], vec![4.0, 5.0, 6.0]);
    }

    #[test]
    fn test_read_jsonl_nested() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"{{"data": {{"embedding": [1.0, 2.0]}}}}"#).unwrap();
        writeln!(temp_file, r#"{{"data": {{"embedding": [3.0, 4.0]}}}}"#).unwrap();
        temp_file.flush().unwrap();

        let embeddings = read_jsonl(temp_file.path().to_str().unwrap(), "data.embedding").unwrap();

        assert_eq!(embeddings.len(), 2);
        assert_eq!(embeddings[0], vec![1.0, 2.0]);
        assert_eq!(embeddings[1], vec![3.0, 4.0]);
    }

    #[test]
    fn test_read_jsonl_empty_lines() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"{{"embedding": [1.0, 2.0]}}"#).unwrap();
        writeln!(temp_file).unwrap();
        writeln!(temp_file, r#"{{"embedding": [3.0, 4.0]}}"#).unwrap();
        temp_file.flush().unwrap();

        let embeddings = read_jsonl(temp_file.path().to_str().unwrap(), "embedding").unwrap();

        assert_eq!(embeddings.len(), 2);
    }

    #[test]
    fn test_write_jsonl() {
        let temp_file = NamedTempFile::new().unwrap();
        let elids = vec!["ELID123".to_string(), "ELID456".to_string()];

        write_jsonl(temp_file.path().to_str().unwrap(), &elids, "elid").unwrap();

        let content = std::fs::read_to_string(temp_file.path()).unwrap();
        let lines: Vec<&str> = content.lines().collect();

        assert_eq!(lines.len(), 2);

        let json1: Value = serde_json::from_str(lines[0]).unwrap();
        let json2: Value = serde_json::from_str(lines[1]).unwrap();

        assert_eq!(json1["elid"], "ELID123");
        assert_eq!(json2["elid"], "ELID456");
    }

    #[test]
    fn test_write_jsonl_custom_field() {
        let temp_file = NamedTempFile::new().unwrap();
        let elids = vec!["ID1".to_string()];

        write_jsonl(temp_file.path().to_str().unwrap(), &elids, "custom_id").unwrap();

        let content = std::fs::read_to_string(temp_file.path()).unwrap();
        let json: Value = serde_json::from_str(content.trim()).unwrap();

        assert_eq!(json["custom_id"], "ID1");
    }

    #[test]
    fn test_write_jsonl_empty_field_name() {
        let temp_file = NamedTempFile::new().unwrap();
        let elids = vec!["ELID123".to_string()];

        let result = write_jsonl(temp_file.path().to_str().unwrap(), &elids, "");
        assert!(result.is_err());
    }
}
