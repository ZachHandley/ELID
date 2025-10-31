//! CLI integration tests for the ELID command-line tool.
//!
//! These tests verify end-to-end behavior of the CLI including:
//! - Encoding embeddings to ELIDs with different profiles
//! - Decoding ELIDs to various output formats
//! - Stdin/stdout piping workflows
//! - Error handling for invalid inputs
//! - Help message display

use assert_cmd::Command;

macro_rules! elid_cmd {
    () => {{
        let path = assert_cmd::cargo::cargo_bin!("elid");
        Command::from(std::process::Command::new(path))
    }};
}
use predicates::prelude::*;
use std::fs;
use std::io::Write;
use tempfile::{NamedTempFile, TempDir};

/// Helper function to create a test CSV file with embeddings
fn create_test_csv(path: &std::path::Path, num_embeddings: usize) -> anyhow::Result<()> {
    let mut file = fs::File::create(path)?;
    writeln!(file, "embedding")?;

    for i in 0..num_embeddings {
        // Create simple test embeddings (128 dimensions)
        let embedding: Vec<f32> = (0..128).map(|j| (i as f32 + j as f32) / 100.0).collect();

        let embedding_str = embedding
            .iter()
            .map(|f| f.to_string())
            .collect::<Vec<_>>()
            .join(",");

        writeln!(file, "\"[{}]\"", embedding_str)?;
    }

    Ok(())
}

/// Helper function to create a test JSONL file with embeddings
fn create_test_jsonl(path: &std::path::Path, num_embeddings: usize) -> anyhow::Result<()> {
    let mut file = fs::File::create(path)?;

    for i in 0..num_embeddings {
        // Create simple test embeddings (128 dimensions)
        let embedding: Vec<f32> = (0..128).map(|j| (i as f32 + j as f32) / 100.0).collect();

        let json = serde_json::json!({
            "embedding": embedding
        });

        writeln!(file, "{}", serde_json::to_string(&json)?)?;
    }

    Ok(())
}

#[test]
fn test_encode_csv_to_text() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("embeddings.csv");
    let output_path = temp_dir.path().join("ids.txt");

    // Create test CSV with 5 embeddings
    create_test_csv(&input_path, 5).unwrap();

    // Run encode command
    let mut cmd = elid_cmd!();
    cmd.arg("encode")
        .arg("--profile")
        .arg("mini128")
        .arg("--input")
        .arg(&input_path)
        .arg("--format")
        .arg("csv")
        .arg("--output")
        .arg(&output_path);

    cmd.assert()
        .success()
        .stderr(predicate::str::contains("Read 5 embeddings"))
        .stderr(predicate::str::contains("Encoded 5 embeddings"));

    // Verify output file exists and contains ELIDs
    let output = fs::read_to_string(&output_path).unwrap();
    let lines: Vec<&str> = output.lines().collect();

    assert_eq!(lines.len(), 5, "Should have 5 ELIDs");

    // Each ELID should be a non-empty string
    for line in lines {
        assert!(!line.is_empty(), "ELID should not be empty");
        assert!(line.len() > 20, "ELID should be reasonably long");
    }
}

#[test]
fn test_encode_jsonl_to_csv() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("embeddings.jsonl");
    let output_path = temp_dir.path().join("output.csv");

    // Create test JSONL with 3 embeddings
    create_test_jsonl(&input_path, 3).unwrap();

    // Run encode command with CSV output
    let mut cmd = elid_cmd!();
    cmd.arg("encode")
        .arg("-p")
        .arg("mini128")
        .arg("-i")
        .arg(&input_path)
        .arg("--format")
        .arg("jsonl")
        .arg("-o")
        .arg(&output_path)
        .arg("--output-format")
        .arg("csv");

    cmd.assert()
        .success()
        .stderr(predicate::str::contains("Read 3 embeddings"))
        .stderr(predicate::str::contains("Encoded 3 embeddings"));

    // Verify CSV output has header and data
    let output = fs::read_to_string(&output_path).unwrap();
    let lines: Vec<&str> = output.lines().collect();

    assert!(lines.len() >= 4, "Should have header + 3 data rows");
    assert!(
        lines[0].contains("embedding"),
        "First line should be CSV header"
    );
    assert!(
        lines[0].contains("elid"),
        "Header should include elid column"
    );
}

#[test]
fn test_decode_hex_format() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("embeddings.csv");
    let elids_path = temp_dir.path().join("ids.txt");
    let decoded_path = temp_dir.path().join("decoded.txt");

    // Step 1: Create and encode embeddings
    create_test_csv(&input_path, 3).unwrap();

    let mut cmd = elid_cmd!();
    cmd.arg("encode")
        .arg("-p")
        .arg("mini128")
        .arg("-i")
        .arg(&input_path)
        .arg("-o")
        .arg(&elids_path);

    cmd.assert().success();

    // Step 2: Decode the ELIDs to hex format
    let mut cmd = elid_cmd!();
    cmd.arg("decode")
        .arg("--input")
        .arg(&elids_path)
        .arg("--format")
        .arg("hex")
        .arg("--output")
        .arg(&decoded_path);

    cmd.assert()
        .success()
        .stderr(predicate::str::contains("Read 3 ELIDs"))
        .stderr(predicate::str::contains("Successfully decoded 3 ELIDs"));

    // Verify hex output format
    let output = fs::read_to_string(&decoded_path).unwrap();
    let lines: Vec<&str> = output.lines().collect();

    assert_eq!(lines.len(), 3, "Should have 3 decoded lines");

    for line in lines {
        // Each line should be "ELID: hexbytes"
        assert!(line.contains(":"), "Line should contain colon separator");
        let parts: Vec<&str> = line.split(':').collect();
        assert_eq!(parts.len(), 2, "Should have ELID:hex format");

        // Hex part should only contain valid hex characters
        let hex_part = parts[1].trim();
        assert!(
            hex_part.chars().all(|c| c.is_ascii_hexdigit()),
            "Hex part should only contain hex digits"
        );
    }
}

#[test]
fn test_decode_json_format() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("embeddings.csv");
    let elids_path = temp_dir.path().join("ids.txt");
    let decoded_path = temp_dir.path().join("decoded.json");

    // Step 1: Create and encode embeddings
    create_test_csv(&input_path, 2).unwrap();

    let mut cmd = elid_cmd!();
    cmd.arg("encode")
        .arg("-p")
        .arg("morton10x10")
        .arg("-i")
        .arg(&input_path)
        .arg("-o")
        .arg(&elids_path);

    cmd.assert().success();

    // Step 2: Decode to JSON format
    let mut cmd = elid_cmd!();
    cmd.arg("decode")
        .arg("-i")
        .arg(&elids_path)
        .arg("--format")
        .arg("json")
        .arg("-o")
        .arg(&decoded_path);

    cmd.assert().success();

    // Verify JSON output format
    let output = fs::read_to_string(&decoded_path).unwrap();
    let lines: Vec<&str> = output.lines().collect();

    assert_eq!(lines.len(), 2, "Should have 2 JSON lines");

    for line in lines {
        // Parse as JSON and verify structure
        let parsed: serde_json::Value = serde_json::from_str(line).unwrap();

        assert!(parsed["elid"].is_string(), "Should have elid field");
        assert!(parsed["bytes"].is_string(), "Should have bytes field");
        assert!(parsed.get("profile").is_some(), "Should have profile field");
    }
}

#[test]
fn test_stdin_stdout_piping() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("embeddings.csv");

    // Create test CSV
    create_test_csv(&input_path, 2).unwrap();
    let input_content = fs::read_to_string(&input_path).unwrap();

    // Run encode with stdin/stdout
    let mut cmd = elid_cmd!();
    cmd.arg("encode")
        .arg("-p")
        .arg("mini128")
        .arg("-i")
        .arg("-")
        .arg("--format")
        .arg("csv")
        .write_stdin(input_content);

    let output = cmd.assert().success();

    // Verify stdout contains ELIDs
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let lines: Vec<&str> = stdout.lines().collect();

    assert_eq!(lines.len(), 2, "Should have 2 ELIDs in stdout");

    for line in lines {
        assert!(!line.is_empty(), "ELID should not be empty");
    }
}

#[test]
fn test_different_profiles() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("embeddings.csv");
    create_test_csv(&input_path, 2).unwrap();

    // Test each profile
    let profiles = vec!["mini128", "morton10x10", "hilbert10x10"];

    for profile in profiles {
        let output_path = temp_dir.path().join(format!("{}_ids.txt", profile));

        let mut cmd = elid_cmd!();
        cmd.arg("encode")
            .arg("-p")
            .arg(profile)
            .arg("-i")
            .arg(&input_path)
            .arg("-o")
            .arg(&output_path);

        cmd.assert()
            .success()
            .stderr(predicate::str::contains("Encoded 2 embeddings"));

        // Verify output exists
        assert!(
            output_path.exists(),
            "Output file should exist for {}",
            profile
        );

        let output = fs::read_to_string(&output_path).unwrap();
        assert_eq!(
            output.lines().count(),
            2,
            "Should have 2 ELIDs for {}",
            profile
        );
    }
}

#[test]
fn test_invalid_profile() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("embeddings.csv");
    let output_path = temp_dir.path().join("ids.txt");

    create_test_csv(&input_path, 2).unwrap();

    let mut cmd = elid_cmd!();
    cmd.arg("encode")
        .arg("-p")
        .arg("invalid_profile_name")
        .arg("-i")
        .arg(&input_path)
        .arg("-o")
        .arg(&output_path);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Unknown profile"))
        .stderr(predicate::str::contains("Valid profiles"));
}

#[test]
fn test_invalid_input_format() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("embeddings.txt");
    let output_path = temp_dir.path().join("ids.txt");

    // Create a dummy file
    fs::write(&input_path, "dummy content").unwrap();

    let mut cmd = elid_cmd!();
    cmd.arg("encode")
        .arg("-p")
        .arg("mini128")
        .arg("-i")
        .arg(&input_path)
        .arg("--format")
        .arg("invalid_format")
        .arg("-o")
        .arg(&output_path);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Unknown input format"));
}

#[test]
fn test_invalid_output_format() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("embeddings.csv");
    let output_path = temp_dir.path().join("ids.txt");

    create_test_csv(&input_path, 2).unwrap();

    let mut cmd = elid_cmd!();
    cmd.arg("encode")
        .arg("-p")
        .arg("mini128")
        .arg("-i")
        .arg(&input_path)
        .arg("-o")
        .arg(&output_path)
        .arg("--output-format")
        .arg("invalid_format");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Unknown output format"));
}

#[test]
fn test_nonexistent_input_file() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("nonexistent.csv");
    let output_path = temp_dir.path().join("ids.txt");

    let mut cmd = elid_cmd!();
    cmd.arg("encode")
        .arg("-p")
        .arg("mini128")
        .arg("-i")
        .arg(&input_path)
        .arg("-o")
        .arg(&output_path);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Failed"));
}

#[test]
fn test_help_message_encode() {
    let mut cmd = elid_cmd!();
    cmd.arg("encode").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Encode embeddings to ELIDs"))
        .stdout(predicate::str::contains("--profile"))
        .stdout(predicate::str::contains("--input"))
        .stdout(predicate::str::contains("--output"))
        .stdout(predicate::str::contains("Examples:"));
}

#[test]
fn test_help_message_decode() {
    let mut cmd = elid_cmd!();
    cmd.arg("decode").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Decode ELIDs"))
        .stdout(predicate::str::contains("--format"))
        .stdout(predicate::str::contains("--input"))
        .stdout(predicate::str::contains("--output"))
        .stdout(predicate::str::contains("Examples:"));
}

#[test]
fn test_main_help_message() {
    let mut cmd = elid_cmd!();
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "ELID: Embedding Locality IDentifier",
        ))
        .stdout(predicate::str::contains("encode"))
        .stdout(predicate::str::contains("decode"));
}

#[test]
fn test_version_flag() {
    let mut cmd = elid_cmd!();
    cmd.arg("--version");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_progress_bar_flag() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("embeddings.csv");
    let output_path = temp_dir.path().join("ids.txt");

    create_test_csv(&input_path, 5).unwrap();

    let mut cmd = elid_cmd!();
    cmd.arg("encode")
        .arg("-p")
        .arg("mini128")
        .arg("-i")
        .arg(&input_path)
        .arg("-o")
        .arg(&output_path)
        .arg("--progress");

    // Progress bar should work without errors
    cmd.assert()
        .success()
        .stderr(predicate::str::contains("Encoded 5 embeddings"));
}

#[test]
fn test_empty_input_file() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "embedding").unwrap(); // Just header
    temp_file.flush().unwrap();

    let output_path = TempDir::new().unwrap().path().join("ids.txt");

    let mut cmd = elid_cmd!();
    cmd.arg("encode")
        .arg("-p")
        .arg("mini128")
        .arg("-i")
        .arg(temp_file.path())
        .arg("-o")
        .arg(&output_path);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("No embeddings found"));
}

#[test]
fn test_decode_invalid_elid() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "invalid_elid_string").unwrap();
    writeln!(temp_file, "another_invalid").unwrap();
    temp_file.flush().unwrap();

    let output_path = TempDir::new().unwrap().path().join("decoded.txt");

    let mut cmd = elid_cmd!();
    cmd.arg("decode")
        .arg("-i")
        .arg(temp_file.path())
        .arg("--format")
        .arg("hex")
        .arg("-o")
        .arg(&output_path);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("failed"));
}

#[test]
fn test_encode_custom_column_name() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("embeddings.csv");
    let output_path = temp_dir.path().join("ids.txt");

    // Create CSV with custom column name
    let mut file = fs::File::create(&input_path).unwrap();
    writeln!(file, "my_vectors").unwrap();

    for i in 0..2 {
        let embedding: Vec<f32> = (0..128).map(|j| (i as f32 + j as f32) / 100.0).collect();

        let embedding_str = embedding
            .iter()
            .map(|f| f.to_string())
            .collect::<Vec<_>>()
            .join(",");

        writeln!(file, "\"[{}]\"", embedding_str).unwrap();
    }

    let mut cmd = elid_cmd!();
    cmd.arg("encode")
        .arg("-p")
        .arg("mini128")
        .arg("-i")
        .arg(&input_path)
        .arg("--column")
        .arg("my_vectors")
        .arg("-o")
        .arg(&output_path);

    cmd.assert()
        .success()
        .stderr(predicate::str::contains("Encoded 2 embeddings"));
}

#[test]
fn test_encode_jsonl_output_format() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("embeddings.csv");
    let output_path = temp_dir.path().join("output.jsonl");

    create_test_csv(&input_path, 3).unwrap();

    let mut cmd = elid_cmd!();
    cmd.arg("encode")
        .arg("-p")
        .arg("mini128")
        .arg("-i")
        .arg(&input_path)
        .arg("-o")
        .arg(&output_path)
        .arg("--output-format")
        .arg("jsonl");

    cmd.assert()
        .success()
        .stderr(predicate::str::contains("Encoded 3 embeddings"));

    // Verify JSONL output
    let output = fs::read_to_string(&output_path).unwrap();
    let lines: Vec<&str> = output.lines().collect();

    assert_eq!(lines.len(), 3, "Should have 3 JSONL lines");

    for line in lines {
        let parsed: serde_json::Value = serde_json::from_str(line).unwrap();
        assert!(
            parsed.get("elid").is_some(),
            "Each line should have elid field"
        );
    }
}
