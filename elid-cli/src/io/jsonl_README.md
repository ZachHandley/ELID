# JSONL (JSON Lines) I/O Module

This module provides functionality for reading embeddings from and writing ELIDs to JSONL (JSON Lines) format files.

## Features

- **Read embeddings from JSONL files** with support for:
  - Simple top-level fields: `{"embedding": [1.0, 2.0, 3.0]}`
  - Nested fields using dot notation: `{"data": {"embedding": [1.0, 2.0, 3.0]}}`
  - Mixed integer and float values (automatically converted to f32)
  - Empty line handling (automatically skipped)
  - stdin support (use "-" as path)

- **Write ELIDs to JSONL files** with:
  - One JSON object per line
  - Customizable field names
  - stdout support (use "-" as path)

## API

### `read_jsonl`

```rust
pub fn read_jsonl(
    path: &str,
    field_path: &str,
) -> anyhow::Result<Vec<Vec<f32>>>
```

Reads embeddings from a JSONL file.

**Arguments:**
- `path` - File path or "-" for stdin
- `field_path` - Dot-notation path to the embedding field (e.g., "embedding" or "data.embedding")

**Returns:**
- `Vec<Vec<f32>>` - Vector of embeddings

**Example:**

```rust
use elid_cli::io::jsonl::read_jsonl;

// Read from file
let embeddings = read_jsonl("embeddings.jsonl", "embedding")?;

// Read nested field
let embeddings = read_jsonl("data.jsonl", "response.data.embedding")?;

// Read from stdin
let embeddings = read_jsonl("-", "embedding")?;
```

### `write_jsonl`

```rust
pub fn write_jsonl(
    path: &str,
    elids: &[String],
    field_name: &str,
) -> anyhow::Result<()>
```

Writes ELIDs to a JSONL file.

**Arguments:**
- `path` - File path or "-" for stdout
- `elids` - Slice of ELID strings to write
- `field_name` - Field name for the ELID in each JSON object

**Example:**

```rust
use elid_cli::io::jsonl::write_jsonl;

let elids = vec!["ELID123".to_string(), "ELID456".to_string()];

// Write to file
write_jsonl("output.jsonl", &elids, "elid")?;

// Write to stdout
write_jsonl("-", &elids, "elid")?;

// Custom field name
write_jsonl("output.jsonl", &elids, "identifier")?;
```

## Input Format

### Simple Format

```jsonl
{"embedding": [1.0, 2.0, 3.0]}
{"embedding": [4.0, 5.0, 6.0]}
```

### Nested Format

```jsonl
{"id": 1, "data": {"embedding": [1.0, 2.0, 3.0]}}
{"id": 2, "data": {"embedding": [4.0, 5.0, 6.0]}}
```

### Deeply Nested Format (API Response Style)

```jsonl
{"response": {"status": "ok", "data": {"embedding": [1.0, 2.0, 3.0]}}}
{"response": {"status": "ok", "data": {"embedding": [4.0, 5.0, 6.0]}}}
```

## Output Format

```jsonl
{"elid":"ELID_ABC123"}
{"elid":"ELID_DEF456"}
{"elid":"ELID_GHI789"}
```

## Error Handling

The module provides detailed error messages with context:

- **Field not found**: `Field 'embedding' not found in path 'data.embedding'`
- **Invalid JSON**: `Failed to parse JSON at line 5`
- **Non-array embedding**: `Embedding field is not an array at line 3`
- **File not found**: `Failed to open file: embeddings.jsonl`

All errors include line numbers for easy debugging.

## Performance

- Uses `BufReader` and `BufWriter` for efficient I/O
- Streaming processing - reads line by line
- Pre-allocates vectors when possible
- Zero-copy string slicing for field extraction

## Testing

Run the tests:

```bash
# Unit tests
cargo test --package elid-cli --lib io::jsonl

# Integration tests
cargo test --package elid-cli --test jsonl_integration

# All tests
cargo test --package elid-cli
```

Run the demo example:

```bash
cargo run --package elid-cli --example jsonl_demo
```

## CLI Usage

### Encoding

```bash
# Read from JSONL file, write ELIDs
elid encode --profile mini128 \
  --input embeddings.jsonl \
  --format jsonl \
  --column embedding \
  --output ids.jsonl \
  --output-format jsonl

# Read from stdin with nested field
cat data.jsonl | elid encode --profile hilbert10x10 \
  --input - \
  --format jsonl \
  --column response.data.embedding \
  --output - \
  --output-format jsonl
```

### Decoding

```bash
# Read ELIDs from JSONL file
elid decode --input ids.jsonl --format json --output decoded.json
```

## Implementation Details

### Nested Field Extraction

The module uses a simple dot-notation parser that splits the field path and iteratively navigates through the JSON structure:

```rust
// Input: {"a": {"b": {"c": [1, 2, 3]}}}
// Field path: "a.b.c"
// Result: [1, 2, 3]
```

### Type Conversion

The module accepts both integers and floats in the embedding arrays:

```jsonl
{"embedding": [1, 2.5, 3, 4.7]}  // Valid, converted to [1.0, 2.5, 3.0, 4.7]
```

### stdin/stdout Handling

- Use "-" as the path to read from stdin or write to stdout
- Useful for piping data between commands
- Fully buffered for performance

## Limitations

- All embeddings must be numeric arrays (no nested objects)
- Field paths cannot contain dots in field names (e.g., "my.field" would be interpreted as two fields)
- Large files are processed sequentially (no parallel processing)

## Future Enhancements

Potential improvements for future versions:

1. Parallel processing for large files
2. Memory-mapped I/O for very large files
3. Streaming iterator API for low-memory environments
4. Support for escaped dots in field names
5. Compression support (gzip, zstd)
