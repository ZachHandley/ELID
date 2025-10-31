//! Decode command implementation for converting ELIDs back to raw bytes or profile info.

use anyhow::{bail, Context, Result};
use elid_core::{decode, Elid};
use serde_json::json;
use std::fs::File;
use std::io::{stdin, stdout, BufRead, BufReader, BufWriter, Write};

/// Handle the decode command - convert ELIDs to raw bytes or profile info
///
/// # Arguments
///
/// * `input` - Input file path or None for stdin
/// * `format` - Output format: "hex", "json", "binary"
/// * `output` - Output file path or None for stdout
///
/// # Returns
///
/// `Ok(())` on success, or an error describing what went wrong
pub fn handle_decode(input: Option<String>, format: &str, output: Option<String>) -> Result<()> {
    // Step 1: Read ELIDs from input (one per line or stdin)
    let elids = read_elids(&input).with_context(|| "Failed to read ELIDs from input")?;

    if elids.is_empty() {
        bail!("No ELIDs found in input");
    }

    eprintln!("Read {} ELIDs", elids.len());

    // Step 2: Decode each ELID
    let mut decoded_results = Vec::with_capacity(elids.len());
    let mut decode_errors = 0;

    for (idx, elid_str) in elids.iter().enumerate() {
        // Create ELID from string
        match Elid::from_string(elid_str.clone()) {
            Ok(elid) => {
                // Decode the ELID
                match decode(&elid) {
                    Ok(bytes) => {
                        // Extract profile info
                        let profile_info = elid.profile().ok();
                        decoded_results.push((elid_str.clone(), bytes, profile_info));
                    }
                    Err(e) => {
                        decode_errors += 1;
                        eprintln!("Warning: Failed to decode ELID at line {}: {}", idx + 1, e);
                    }
                }
            }
            Err(e) => {
                decode_errors += 1;
                eprintln!("Warning: Invalid ELID at line {}: {}", idx + 1, e);
            }
        }
    }

    if decode_errors > 0 {
        eprintln!(
            "Warning: {} ELIDs failed to decode and were skipped",
            decode_errors
        );
    }

    if decoded_results.is_empty() {
        bail!("All ELIDs failed to decode");
    }

    // Step 3: Format and write output
    write_decoded_output(&output, format, &decoded_results)
        .with_context(|| "Failed to write decoded output")?;

    eprintln!("Successfully decoded {} ELIDs", decoded_results.len());

    Ok(())
}

/// Read ELIDs from input (one per line)
fn read_elids(input: &Option<String>) -> Result<Vec<String>> {
    let reader: Box<dyn BufRead> = if let Some(path) = input {
        if path == "-" {
            Box::new(BufReader::new(stdin()))
        } else {
            let file =
                File::open(path).with_context(|| format!("Failed to open input file: {}", path))?;
            Box::new(BufReader::new(file))
        }
    } else {
        Box::new(BufReader::new(stdin()))
    };

    let mut elids = Vec::new();

    for (line_num, line_result) in reader.lines().enumerate() {
        let line = line_result.with_context(|| format!("Failed to read line {}", line_num + 1))?;

        let trimmed = line.trim();

        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }

        elids.push(trimmed.to_string());
    }

    Ok(elids)
}

/// Write decoded output in the specified format
fn write_decoded_output(
    output: &Option<String>,
    format: &str,
    results: &[(String, Vec<u8>, Option<elid_core::ProfileInfo>)],
) -> Result<()> {
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

    match format.to_lowercase().as_str() {
        "hex" => write_hex_format(&mut writer, results)?,
        "json" => write_json_format(&mut writer, results)?,
        "binary" => write_binary_format(&mut writer, results)?,
        _ => bail!(
            "Unknown output format '{}'. Valid formats: hex, json, binary",
            format
        ),
    }

    writer.flush().with_context(|| "Failed to flush output")?;

    Ok(())
}

/// Write output in hexadecimal format
fn write_hex_format(
    writer: &mut dyn Write,
    results: &[(String, Vec<u8>, Option<elid_core::ProfileInfo>)],
) -> Result<()> {
    for (elid, bytes, _) in results {
        let hex_str = bytes
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
            .join("");

        writeln!(writer, "{}: {}", elid, hex_str).with_context(|| "Failed to write hex output")?;
    }
    Ok(())
}

/// Write output in JSON format
fn write_json_format(
    writer: &mut dyn Write,
    results: &[(String, Vec<u8>, Option<elid_core::ProfileInfo>)],
) -> Result<()> {
    for (elid, bytes, profile_info) in results {
        let hex_bytes = bytes
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
            .join("");

        let profile_json = if let Some(info) = profile_info {
            json!({
                "version": info.version,
                "profile_type": format!("0x{:02x}", info.profile_type),
                "transform_id": info.transform_id,
                "model_id": info.model_id,
            })
        } else {
            json!(null)
        };

        let output = json!({
            "elid": elid,
            "bytes": hex_bytes,
            "profile": profile_json,
        });

        writeln!(writer, "{}", serde_json::to_string(&output)?)
            .with_context(|| "Failed to write JSON output")?;
    }
    Ok(())
}

/// Write output in binary format (raw bytes)
fn write_binary_format(
    writer: &mut dyn Write,
    results: &[(String, Vec<u8>, Option<elid_core::ProfileInfo>)],
) -> Result<()> {
    for (_, bytes, _) in results {
        writer
            .write_all(bytes)
            .with_context(|| "Failed to write binary output")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_read_elids_basic() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "ELID001").unwrap();
        writeln!(temp_file, "ELID002").unwrap();
        writeln!(temp_file, "ELID003").unwrap();
        temp_file.flush().unwrap();

        let elids = read_elids(&Some(temp_file.path().to_str().unwrap().to_string())).unwrap();

        assert_eq!(elids.len(), 3);
        assert_eq!(elids[0], "ELID001");
        assert_eq!(elids[1], "ELID002");
        assert_eq!(elids[2], "ELID003");
    }

    #[test]
    fn test_read_elids_with_empty_lines() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "ELID001").unwrap();
        writeln!(temp_file).unwrap();
        writeln!(temp_file, "ELID002").unwrap();
        writeln!(temp_file, "  ").unwrap();
        writeln!(temp_file, "ELID003").unwrap();
        temp_file.flush().unwrap();

        let elids = read_elids(&Some(temp_file.path().to_str().unwrap().to_string())).unwrap();

        assert_eq!(elids.len(), 3);
        assert_eq!(elids[0], "ELID001");
        assert_eq!(elids[1], "ELID002");
        assert_eq!(elids[2], "ELID003");
    }

    #[test]
    fn test_read_elids_with_whitespace() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "  ELID001  ").unwrap();
        writeln!(temp_file, "\tELID002\t").unwrap();
        temp_file.flush().unwrap();

        let elids = read_elids(&Some(temp_file.path().to_str().unwrap().to_string())).unwrap();

        assert_eq!(elids.len(), 2);
        assert_eq!(elids[0], "ELID001");
        assert_eq!(elids[1], "ELID002");
    }

    #[test]
    fn test_write_hex_format() {
        let results = vec![
            ("ELID001".to_string(), vec![0x01, 0x02, 0x03, 0x04], None),
            ("ELID002".to_string(), vec![0xAB, 0xCD, 0xEF], None),
        ];

        let mut output = Vec::new();
        write_hex_format(&mut output, &results).unwrap();

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("ELID001: 01020304"));
        assert!(output_str.contains("ELID002: abcdef"));
    }

    #[test]
    fn test_write_binary_format() {
        let results = vec![
            ("ELID001".to_string(), vec![0x01, 0x02, 0x03], None),
            ("ELID002".to_string(), vec![0x04, 0x05], None),
        ];

        let mut output = Vec::new();
        write_binary_format(&mut output, &results).unwrap();

        assert_eq!(output, vec![0x01, 0x02, 0x03, 0x04, 0x05]);
    }
}
