//! Example demonstrating JSONL I/O functionality

use elid_cli::io::jsonl::{read_jsonl, write_jsonl};
use std::io::Write;
use tempfile::NamedTempFile;

fn main() -> anyhow::Result<()> {
    println!("JSONL I/O Demo\n");

    // Create a sample JSONL file with embeddings
    let mut input_file = NamedTempFile::new()?;
    println!("Creating sample JSONL file with embeddings...");

    // Write sample embeddings in JSONL format
    writeln!(
        input_file,
        r#"{{"id": 1, "text": "hello world", "embedding": [0.1, 0.2, 0.3, 0.4]}}"#
    )?;
    writeln!(
        input_file,
        r#"{{"id": 2, "text": "goodbye world", "embedding": [0.5, 0.6, 0.7, 0.8]}}"#
    )?;
    writeln!(
        input_file,
        r#"{{"id": 3, "text": "rust is great", "embedding": [0.9, 1.0, 1.1, 1.2]}}"#
    )?;
    input_file.flush()?;

    println!("Sample file created at: {}\n", input_file.path().display());

    // Read embeddings from the JSONL file
    println!("Reading embeddings from JSONL file...");
    let embeddings = read_jsonl(input_file.path().to_str().unwrap(), "embedding")?;

    println!("Successfully read {} embeddings:", embeddings.len());
    for (i, embedding) in embeddings.iter().enumerate() {
        println!("  Embedding {}: {:?}", i + 1, embedding);
    }
    println!();

    // Simulate encoding embeddings to ELIDs
    println!("Simulating ELID encoding...");
    let elids = vec![
        "ELID_ABC123XYZ".to_string(),
        "ELID_DEF456UVW".to_string(),
        "ELID_GHI789RST".to_string(),
    ];

    // Write ELIDs to output file
    let output_file = NamedTempFile::new()?;
    println!("Writing ELIDs to JSONL file...");
    write_jsonl(output_file.path().to_str().unwrap(), &elids, "elid")?;

    println!("ELIDs written to: {}", output_file.path().display());

    // Read back and display
    let content = std::fs::read_to_string(output_file.path())?;
    println!("\nOutput file contents:");
    println!("{}", content);

    // Demo nested field extraction
    println!("Demo: Reading nested fields\n");
    let mut nested_file = NamedTempFile::new()?;
    writeln!(
        nested_file,
        r#"{{"response": {{"data": {{"embedding": [1.1, 2.2, 3.3]}}}}}}"#
    )?;
    writeln!(
        nested_file,
        r#"{{"response": {{"data": {{"embedding": [4.4, 5.5, 6.6]}}}}}}"#
    )?;
    nested_file.flush()?;

    println!("Reading deeply nested field 'response.data.embedding'...");
    let nested_embeddings = read_jsonl(
        nested_file.path().to_str().unwrap(),
        "response.data.embedding",
    )?;

    println!(
        "Successfully read {} nested embeddings:",
        nested_embeddings.len()
    );
    for (i, embedding) in nested_embeddings.iter().enumerate() {
        println!("  Embedding {}: {:?}", i + 1, embedding);
    }

    println!("\nDemo completed successfully!");

    Ok(())
}
