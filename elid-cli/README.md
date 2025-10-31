# elid-cli

**Command-line interface for encoding/decoding high-dimensional embeddings**

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](../LICENSE-MIT)
[![Rust: 1.70+](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)

## Overview

The `elid` CLI provides batch encoding and decoding of high-dimensional embeddings into compact, sortable identifiers. It wraps the `elid-core` library with support for CSV, Parquet, and JSONL input/output formats.

Perfect for:
- Processing large batches of embeddings from ML models
- Converting between formats (CSV, Parquet, JSONL)
- Integration with data pipelines and shell scripts
- Quick testing and prototyping with embeddings

## Installation

Install from the repository:

```bash
cargo install --path elid-cli
```

Or install from the workspace root:

```bash
cargo install --path .
```

Verify installation:

```bash
elid --version
```

## Quick Start

Encode embeddings from a CSV file:

```bash
elid encode --profile mini128 --input embeddings.csv --output ids.txt
```

Decode ELIDs to see their contents:

```bash
elid decode --input ids.txt --format json
```

## Command Overview

### `elid encode` - Convert embeddings to ELIDs

Encodes embeddings from CSV, Parquet, or JSONL files into compact string identifiers.

**Basic usage:**

```bash
elid encode --profile PROFILE --input FILE --output FILE
```

**Options:**

- `-p, --profile PROFILE` - Encoding profile (required)
  - `mini128` - 128-bit SimHash for similarity search
  - `morton10x10` - 10-dim Morton curve for fast indexing
  - `hilbert10x10` - 10-dim Hilbert curve for best locality
- `-i, --input FILE` - Input file path (use "-" for stdin)
- `-o, --output FILE` - Output file path (use "-" for stdout)
- `--format FORMAT` - Input format: `csv`, `parquet`, `jsonl` (default: `csv`)
- `--column COLUMN` - Column name containing embeddings (default: `embedding`)
- `--output-format FORMAT` - Output format: `text`, `csv`, `jsonl` (default: `text`)
- `--progress` - Show progress bar for large batches

### `elid decode` - Convert ELIDs back to raw bytes

Decodes ELID strings to reveal their raw byte representation and profile information.

**Basic usage:**

```bash
elid decode --input FILE --format FORMAT
```

**Options:**

- `-i, --input FILE` - Input file containing ELIDs (use "-" for stdin)
- `-o, --output FILE` - Output file path (use "-" for stdout)
- `--format FORMAT` - Output format: `hex`, `json`, `binary` (default: `hex`)

## Format Support

### Input Formats

#### CSV Format

CSV files with embeddings as array columns:

```csv
id,embedding,label
1,"[0.1, 0.2, 0.3, 0.4]",positive
2,"[0.5, 0.6, 0.7, 0.8]",negative
```

**Usage:**

```bash
elid encode -p mini128 -i data.csv --format csv --column embedding
```

#### Parquet Format

Apache Parquet files with list/array columns (file input only, no stdin):

```bash
elid encode -p morton10x10 -i embeddings.parquet --format parquet --column vectors
```

#### JSONL Format

JSON Lines format with one embedding object per line:

```jsonl
{"id": 1, "embedding": [0.1, 0.2, 0.3], "text": "example"}
{"id": 2, "embedding": [0.4, 0.5, 0.6], "text": "another"}
```

**Usage:**

```bash
elid encode -p mini128 -i data.jsonl --format jsonl --column embedding
```

### Output Formats

#### Text Format (Default)

One ELID per line:

```
04g8c4g0c8g4c0g8c4g0c8g4
08h2d1h4d8h2d1h4d8h2d1h4
```

#### CSV Format

CSV with both embeddings and ELIDs:

```csv
embedding,elid
"[0.1, 0.2, 0.3]",04g8c4g0c8g4c0g8c4g0c8g4
"[0.4, 0.5, 0.6]",08h2d1h4d8h2d1h4d8h2d1h4
```

#### JSONL Format

JSON Lines with ELID field added:

```jsonl
{"elid": "04g8c4g0c8g4c0g8c4g0c8g4"}
{"elid": "08h2d1h4d8h2d1h4d8h2d1h4"}
```

## Common Usage Examples

### 1. Basic encoding with Mini128 profile

```bash
elid encode --profile mini128 --input embeddings.csv --output ids.txt
```

### 2. Use Morton profile with progress bar

```bash
elid encode -p morton10x10 -i data.csv --progress -o ids.csv
```

### 3. Encode from Parquet file

```bash
elid encode -p hilbert10x10 -i embeddings.parquet --format parquet -o ids.txt
```

### 4. Use custom column name

```bash
elid encode -p mini128 -i data.csv --column vectors -o ids.txt
```

### 5. Output as CSV with embeddings

```bash
elid encode -p mini128 -i data.csv -o output.csv --output-format csv
```

### 6. Output as JSONL

```bash
elid encode -p mini128 -i data.jsonl --format jsonl --output-format jsonl -o output.jsonl
```

### 7. Decode to hex format

```bash
elid decode --input ids.txt --format hex
```

### 8. Decode to JSON with profile info

```bash
elid decode -i ids.txt --format json -o decoded.json
```

### 9. Decode to binary

```bash
elid decode -i ids.txt --format binary -o output.bin
```

## Piping with stdin/stdout

The CLI supports Unix-style piping for integration with other tools:

### Pipe from stdin to stdout

```bash
cat embeddings.csv | elid encode -p mini128 -i - > ids.txt
```

### Chain encoding and decoding

```bash
cat embeddings.csv | elid encode -p mini128 -i - | elid decode -i - --format json
```

### Pipe to other tools (e.g., jq)

```bash
elid decode -i ids.txt --format json | jq '.profile'
```

### Filter with grep

```bash
elid encode -p mini128 -i embeddings.csv | grep "^04"
```

### Count unique ELIDs

```bash
elid encode -p mini128 -i embeddings.csv | sort | uniq | wc -l
```

## Batch Processing Tips

### Large Files

For files with >1000 embeddings, use `--progress` to monitor progress:

```bash
elid encode -p mini128 -i large_dataset.csv --progress -o ids.txt
```

### Parallel Processing

Split large files and process in parallel with GNU parallel:

```bash
# Split CSV into 4 parts
split -n l/4 embeddings.csv chunk_

# Process in parallel
parallel elid encode -p mini128 -i {} -o {.}.ids ::: chunk_*

# Combine results
cat chunk_*.ids > all_ids.txt
```

### Error Handling

The CLI continues processing even if some embeddings fail. Check stderr for warnings:

```bash
elid encode -p mini128 -i data.csv -o ids.txt 2> errors.log
```

## Profile Selection Guide

Choose the right profile for your use case:

| Profile | Use Case | Speed | Locality | Output Size |
|---------|----------|-------|----------|-------------|
| **mini128** | Similarity search, deduplication | Fast | Excellent (angular) | 29 chars |
| **morton10x10** | Database indexing, general use | Very fast | Good | 24 chars |
| **hilbert10x10** | Research, quality-critical apps | Moderate | Best | 24 chars |

**Recommendations:**

- **mini128**: Default choice for most similarity search tasks
- **morton10x10**: Best for production database indexing (fastest)
- **hilbert10x10**: When you need absolute best locality (5-10% better than Morton)

## Integration Examples

### Postgres Database

```bash
# Generate ELIDs from embeddings
elid encode -p morton10x10 -i embeddings.csv -o ids.txt

# Import to Postgres
psql -d mydb -c "COPY embeddings(elid) FROM STDIN" < ids.txt
```

### Python Integration

```python
import subprocess
import json

# Encode embeddings via subprocess
result = subprocess.run(
    ['elid', 'encode', '-p', 'mini128', '-i', 'embeddings.csv'],
    capture_output=True,
    text=True
)

elids = result.stdout.strip().split('\n')
print(f"Generated {len(elids)} ELIDs")

# Decode with JSON output
result = subprocess.run(
    ['elid', 'decode', '-i', 'ids.txt', '--format', 'json'],
    capture_output=True,
    text=True
)

for line in result.stdout.strip().split('\n'):
    data = json.loads(line)
    print(f"ELID: {data['elid']}, Profile: {data['profile']}")
```

### Shell Script

```bash
#!/bin/bash

# Process embeddings from multiple sources
for file in data/*.csv; do
    echo "Processing $file..."
    elid encode -p mini128 -i "$file" -o "${file%.csv}.ids" --progress
done

echo "All files processed!"
```

## Performance

- **Throughput**: ~10,000-50,000 embeddings/sec (single thread)
- **Memory**: Minimal - streams through batches
- **Progress**: Shows ETA for batches >1000 embeddings

Actual performance depends on:
- Profile choice (Morton is fastest)
- Embedding dimensions (768 is typical)
- I/O speed (SSD vs network drive)
- Input format (Parquet is faster than CSV)

## Library Documentation

For programmatic usage, see the library documentation:

```bash
cd ../elid-core
cargo doc --open
```

Or visit the [elid-core README](../elid-core/README.md)

## Troubleshooting

### "No embeddings found in input"

- Check that your CSV/JSONL has the correct column name
- Use `--column` to specify the embedding column
- Verify your file is not empty

### "Failed to read Parquet input"

- Parquet format does not support stdin (use a file path)
- Ensure the Parquet file has list/array columns
- Check column name with `--column`

### "Invalid dimension" errors

- ELID requires embeddings between 64-2048 dimensions
- Check your embedding size matches this range

### Progress bar not showing

- Progress only shows for batches >1000 embeddings
- Add `--progress` flag explicitly
- Progress is written to stderr, not stdout

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## See Also

- [elid-core](../elid-core/README.md) - Core library documentation
- [Project README](../README.md) - Overall project information
- [GitHub Repository](https://github.com/zachhandley/ELID) - Source code and issues

---

**Status**: Under active development - API may change before 1.0 release
