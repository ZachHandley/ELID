//! Integration tests for JSONL I/O functionality

use elid_cli::io::jsonl::{read_jsonl, write_jsonl};
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_jsonl_round_trip() {
    // Create a temporary JSONL file with embeddings
    let mut input_file = NamedTempFile::new().unwrap();
    writeln!(
        input_file,
        r#"{{"id": 1, "embedding": [1.0, 2.0, 3.0, 4.0]}}"#
    )
    .unwrap();
    writeln!(
        input_file,
        r#"{{"id": 2, "embedding": [5.0, 6.0, 7.0, 8.0]}}"#
    )
    .unwrap();
    writeln!(
        input_file,
        r#"{{"id": 3, "embedding": [9.0, 10.0, 11.0, 12.0]}}"#
    )
    .unwrap();
    input_file.flush().unwrap();

    // Read embeddings
    let embeddings = read_jsonl(input_file.path().to_str().unwrap(), "embedding").unwrap();

    // Verify embeddings
    assert_eq!(embeddings.len(), 3);
    assert_eq!(embeddings[0], vec![1.0, 2.0, 3.0, 4.0]);
    assert_eq!(embeddings[1], vec![5.0, 6.0, 7.0, 8.0]);
    assert_eq!(embeddings[2], vec![9.0, 10.0, 11.0, 12.0]);

    // Simulate encoding to ELIDs (just mock strings for now)
    let elids = vec![
        "ELID_1234".to_string(),
        "ELID_5678".to_string(),
        "ELID_9012".to_string(),
    ];

    // Write ELIDs to output file
    let output_file = NamedTempFile::new().unwrap();
    write_jsonl(output_file.path().to_str().unwrap(), &elids, "elid").unwrap();

    // Read back and verify
    let content = std::fs::read_to_string(output_file.path()).unwrap();
    let lines: Vec<&str> = content.lines().collect();

    assert_eq!(lines.len(), 3);

    let json1: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
    let json2: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
    let json3: serde_json::Value = serde_json::from_str(lines[2]).unwrap();

    assert_eq!(json1["elid"], "ELID_1234");
    assert_eq!(json2["elid"], "ELID_5678");
    assert_eq!(json3["elid"], "ELID_9012");
}

#[test]
fn test_nested_field_extraction() {
    // Test with deeply nested structure like API responses
    let mut input_file = NamedTempFile::new().unwrap();
    writeln!(
        input_file,
        r#"{{"response": {{"data": {{"embedding": [1.0, 2.0, 3.0]}}}}}}"#
    )
    .unwrap();
    writeln!(
        input_file,
        r#"{{"response": {{"data": {{"embedding": [4.0, 5.0, 6.0]}}}}}}"#
    )
    .unwrap();
    input_file.flush().unwrap();

    let embeddings = read_jsonl(
        input_file.path().to_str().unwrap(),
        "response.data.embedding",
    )
    .unwrap();

    assert_eq!(embeddings.len(), 2);
    assert_eq!(embeddings[0], vec![1.0, 2.0, 3.0]);
    assert_eq!(embeddings[1], vec![4.0, 5.0, 6.0]);
}

#[test]
fn test_mixed_number_types() {
    // Test with mixed integer and float values
    let mut input_file = NamedTempFile::new().unwrap();
    writeln!(input_file, r#"{{"embedding": [1, 2.5, 3, 4.7]}}"#).unwrap();
    input_file.flush().unwrap();

    let embeddings = read_jsonl(input_file.path().to_str().unwrap(), "embedding").unwrap();

    assert_eq!(embeddings.len(), 1);
    assert_eq!(embeddings[0], vec![1.0, 2.5, 3.0, 4.7]);
}

#[test]
fn test_error_handling_invalid_field() {
    let mut input_file = NamedTempFile::new().unwrap();
    writeln!(input_file, r#"{{"data": [1.0, 2.0, 3.0]}}"#).unwrap();
    input_file.flush().unwrap();

    let result = read_jsonl(input_file.path().to_str().unwrap(), "embedding");
    assert!(result.is_err());
    let err_msg = format!("{:#}", result.unwrap_err());
    // The error message includes context about the field path
    assert!(err_msg.contains("Field") && err_msg.contains("not found"));
}

#[test]
fn test_error_handling_invalid_json() {
    let mut input_file = NamedTempFile::new().unwrap();
    writeln!(input_file, r#"{{invalid json}}"#).unwrap();
    input_file.flush().unwrap();

    let result = read_jsonl(input_file.path().to_str().unwrap(), "embedding");
    assert!(result.is_err());
}

#[test]
fn test_error_handling_non_array_embedding() {
    let mut input_file = NamedTempFile::new().unwrap();
    writeln!(input_file, r#"{{"embedding": "not an array"}}"#).unwrap();
    input_file.flush().unwrap();

    let result = read_jsonl(input_file.path().to_str().unwrap(), "embedding");
    assert!(result.is_err());
    let err_msg = format!("{:#}", result.unwrap_err());
    // The error message should mention array
    assert!(err_msg.contains("array"));
}

#[test]
fn test_large_batch() {
    // Test with a larger dataset
    let mut input_file = NamedTempFile::new().unwrap();

    for i in 0..1000 {
        let embedding: Vec<f32> = (0..128).map(|j| (i * 128 + j) as f32 * 0.01).collect();
        let json = serde_json::json!({"embedding": embedding});
        writeln!(input_file, "{}", serde_json::to_string(&json).unwrap()).unwrap();
    }
    input_file.flush().unwrap();

    let embeddings = read_jsonl(input_file.path().to_str().unwrap(), "embedding").unwrap();

    assert_eq!(embeddings.len(), 1000);
    assert_eq!(embeddings[0].len(), 128);

    // Verify first and last embeddings
    assert_eq!(embeddings[0][0], 0.0);
    assert_eq!(embeddings[999][127], (999 * 128 + 127) as f32 * 0.01);
}
