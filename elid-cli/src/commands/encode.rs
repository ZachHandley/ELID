//! Encode command implementation for converting embeddings to ELIDs.

use anyhow::{bail, Context, Result};
use elid_core::{encode, Profile};
use std::fs::File;
use std::io::{stdout, BufWriter, Write};
use std::time::Instant;

use crate::io::{csv, jsonl, parquet};
use crate::progress::{create_progress_bar, finish_progress, update_progress};

/// Handle the encode command - convert embeddings to ELIDs
///
/// # Arguments
///
/// * `profile_str` - Profile specification string (e.g., "mini128", "morton10x10")
/// * `input` - Input file path or None for stdin (format dependent)
/// * `format` - Input format: "csv", "parquet", "jsonl"
/// * `column` - Column name containing embeddings
/// * `output` - Output file path or None for stdout
/// * `output_format` - Output format: "csv", "jsonl", "text"
/// * `show_progress` - Whether to show progress bar for batches >1000
///
/// # Returns
///
/// `Ok(())` on success, or an error describing what went wrong
pub fn handle_encode(
    profile_str: &str,
    input: Option<String>,
    format: &str,
    column: &str,
    output: Option<String>,
    output_format: &str,
    show_progress: bool,
) -> Result<()> {
    let start_time = Instant::now();

    // Step 1: Parse profile string to Profile enum
    let profile = parse_profile(profile_str)
        .with_context(|| format!("Failed to parse profile: {}", profile_str))?;

    // Step 2: Read embeddings based on input format
    let embeddings =
        read_embeddings(&input, format, column).with_context(|| "Failed to read embeddings")?;

    if embeddings.is_empty() {
        bail!("No embeddings found in input");
    }

    eprintln!("Read {} embeddings", embeddings.len());

    // Step 3: Create progress bar for batches >1000
    let pb = create_progress_bar(embeddings.len() as u64, show_progress);

    // Step 4: Encode each embedding using elid_core::encode
    let mut elids = Vec::with_capacity(embeddings.len());
    let mut encode_errors = 0;

    for (idx, embedding) in embeddings.iter().enumerate() {
        match encode(embedding, &profile) {
            Ok(elid) => {
                elids.push(elid.to_string());
                update_progress(&pb, 1);
            }
            Err(e) => {
                encode_errors += 1;
                eprintln!(
                    "Warning: Failed to encode embedding at index {}: {}",
                    idx, e
                );
                // Continue processing other embeddings
            }
        }
    }

    finish_progress(pb);

    if encode_errors > 0 {
        eprintln!(
            "Warning: {} embeddings failed to encode and were skipped",
            encode_errors
        );
    }

    if elids.is_empty() {
        bail!("All embeddings failed to encode");
    }

    // Step 5: Write output based on output_format
    write_output(&output, output_format, &embeddings, &elids, column)
        .with_context(|| "Failed to write output")?;

    // Step 6: Show encoding rate and total time
    let elapsed = start_time.elapsed();
    let rate = elids.len() as f64 / elapsed.as_secs_f64();

    eprintln!(
        "Encoded {} embeddings in {:.2}s ({:.0} embeddings/sec)",
        elids.len(),
        elapsed.as_secs_f64(),
        rate
    );

    Ok(())
}

/// Parse a profile string into a Profile enum
fn parse_profile(profile_str: &str) -> Result<Profile> {
    match profile_str.to_lowercase().as_str() {
        "mini128" => Ok(Profile::default()), // Mini128 with default seed
        "morton10x10" => Ok(Profile::Morton10x10 {
            dims: 10,
            bits_per_dim: 10,
            transform_id: None,
        }),
        "hilbert10x10" => Ok(Profile::Hilbert10x10 {
            dims: 10,
            bits_per_dim: 10,
            transform_id: None,
        }),
        _ => bail!(
            "Unknown profile '{}'. Valid profiles: mini128, morton10x10, hilbert10x10",
            profile_str
        ),
    }
}

/// Read embeddings from input based on format
fn read_embeddings(input: &Option<String>, format: &str, column: &str) -> Result<Vec<Vec<f32>>> {
    let input_path = input.as_deref().unwrap_or("-");

    match format.to_lowercase().as_str() {
        "csv" => csv::read_csv(input_path, column).with_context(|| "Failed to read CSV input"),
        "parquet" => {
            if input_path == "-" {
                bail!("Parquet format does not support stdin input");
            }
            parquet::read_parquet(input_path, column)
                .with_context(|| "Failed to read Parquet input")
        }
        "jsonl" => {
            jsonl::read_jsonl(input_path, column).with_context(|| "Failed to read JSONL input")
        }
        _ => bail!(
            "Unknown input format '{}'. Valid formats: csv, parquet, jsonl",
            format
        ),
    }
}

/// Write output in the specified format
fn write_output(
    output: &Option<String>,
    output_format: &str,
    embeddings: &[Vec<f32>],
    elids: &[String],
    column: &str,
) -> Result<()> {
    match output_format.to_lowercase().as_str() {
        "text" => write_text_output(output, elids),
        "csv" => {
            let output_path = output.as_deref().unwrap_or("-");
            // For CSV output, we need to filter embeddings to match elids length
            // (in case some failed to encode)
            let valid_count = elids.len();
            let valid_embeddings: Vec<Vec<f32>> =
                embeddings.iter().take(valid_count).cloned().collect();

            csv::write_csv(output_path, &valid_embeddings, elids, column)
                .with_context(|| "Failed to write CSV output")
        }
        "jsonl" => {
            let output_path = output.as_deref().unwrap_or("-");
            jsonl::write_jsonl(output_path, elids, "elid")
                .with_context(|| "Failed to write JSONL output")
        }
        _ => bail!(
            "Unknown output format '{}'. Valid formats: text, csv, jsonl",
            output_format
        ),
    }
}

/// Write ELIDs as plain text (one per line)
fn write_text_output(output: &Option<String>, elids: &[String]) -> Result<()> {
    let mut writer: Box<dyn Write> = if let Some(path) = output {
        if path == "-" {
            Box::new(BufWriter::new(stdout()))
        } else {
            let file = File::create(path)
                .with_context(|| format!("Failed to create output file: {}", path))?;
            Box::new(BufWriter::new(file))
        }
    } else {
        Box::new(BufWriter::new(stdout()))
    };

    for elid in elids {
        writeln!(writer, "{}", elid).with_context(|| "Failed to write ELID to output")?;
    }

    writer.flush().with_context(|| "Failed to flush output")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_profile_mini128() {
        let profile = parse_profile("mini128").unwrap();
        match profile {
            Profile::Mini128 { .. } => {}
            _ => panic!("Expected Mini128 profile"),
        }
    }

    #[test]
    fn test_parse_profile_case_insensitive() {
        let profile = parse_profile("MINI128").unwrap();
        match profile {
            Profile::Mini128 { .. } => {}
            _ => panic!("Expected Mini128 profile"),
        }
    }

    #[test]
    fn test_parse_profile_morton() {
        let profile = parse_profile("morton10x10").unwrap();
        match profile {
            Profile::Morton10x10 {
                dims,
                bits_per_dim,
                transform_id,
            } => {
                assert_eq!(dims, 10);
                assert_eq!(bits_per_dim, 10);
                assert_eq!(transform_id, None);
            }
            _ => panic!("Expected Morton10x10 profile"),
        }
    }

    #[test]
    fn test_parse_profile_hilbert() {
        let profile = parse_profile("hilbert10x10").unwrap();
        match profile {
            Profile::Hilbert10x10 {
                dims,
                bits_per_dim,
                transform_id,
            } => {
                assert_eq!(dims, 10);
                assert_eq!(bits_per_dim, 10);
                assert_eq!(transform_id, None);
            }
            _ => panic!("Expected Hilbert10x10 profile"),
        }
    }

    #[test]
    fn test_parse_profile_invalid() {
        let result = parse_profile("invalid");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown profile"));
    }
}
