# ELID - Embedding Locality IDentifier

[![CI](https://github.com/ZachHandley/ELID/actions/workflows/ci.yml/badge.svg)](https://github.com/ZachHandley/ELID/actions)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

**ELID** enables vector search without a vector store by encoding high-dimensional embeddings into sortable string IDs that preserve locality. Similar vectors produce similar IDs, allowing you to use standard database indexes for similarity search.

ELID also includes a complete suite of fast string similarity algorithms.

## Features

### Embedding Encoding (Vector Search Without Vector Stores)

Convert embeddings from any ML model into compact, sortable identifiers:

| Profile | Output | Best For |
|---------|--------|----------|
| **Mini128** | 26-char base32hex | Fast similarity via Hamming distance |
| **Morton10x10** | 20-char base32hex | Database range queries (Z-order) |
| **Hilbert10x10** | 20-char base32hex | Maximum locality preservation |

**Key benefits:**
- Similar vectors produce similar IDs (locality preservation)
- IDs are lexicographically sortable for database indexing
- No vector store required - use any database with string indexes
- Deterministic: same embedding always produces the same ID

### String Similarity Algorithms

| Algorithm | Type | Best For |
|-----------|------|----------|
| **Levenshtein** | Edit distance | General-purpose comparison, spell checking |
| **Normalized Levenshtein** | Similarity (0-1) | When you need a percentage match |
| **Jaro** | Similarity (0-1) | Short strings |
| **Jaro-Winkler** | Similarity (0-1) | Names and record linkage |
| **Hamming** | Distance | Fixed-length strings, DNA, error codes |
| **OSA** | Edit distance | Typo detection (counts transpositions) |
| **SimHash** | LSH fingerprint | Database-queryable similarity, near-duplicate detection |
| **Best Match** | Composite (0-1) | When unsure which algorithm fits |

## Installation

### Rust

```toml
# String similarity only (zero dependencies)
[dependencies]
elid = "0.1"

# Embedding encoding
[dependencies]
elid = { version = "0.1", features = ["embeddings"] }

# Both features
[dependencies]
elid = { version = "0.1", features = ["strings", "embeddings"] }
```

### Python

```bash
pip install elid
```

### JavaScript (WASM)

```bash
npm install elid-wasm
```

### C/C++

Build with `cargo build --release --features ffi` to get `libelid.so` and `elid.h`.

## Quick Start

### Embedding Encoding (Rust)

```rust
use elid::embeddings::{encode, Profile, Elid};

// Get an embedding from your ML model (e.g., OpenAI, Cohere, sentence-transformers)
let embedding: Vec<f32> = model.embed("Hello, world!")?;

// Encode to a sortable ELID
let profile = Profile::default(); // Mini128
let elid: Elid = encode(&embedding, &profile)?;

println!("ELID: {}", elid); // e.g., "01a3f5g7h9jklmnopqrstuv"

// Similar texts produce similar ELIDs
let elid2 = encode(&model.embed("Hello, universe!")?, &profile)?;

// Compare similarity via Hamming distance
use elid::embeddings::hamming_distance;
let distance = hamming_distance(&elid, &elid2)?; // Lower = more similar
```

### Encoding Profiles

```rust
use elid::embeddings::Profile;

// Mini128: 128-bit SimHash (default)
// Best for: Fast similarity search via Hamming distance
let mini = Profile::Mini128 {
    seed: 0x454c4944_53494d48, // Deterministic seed
};

// Morton10x10: Z-order curve encoding
// Best for: Database range queries
let morton = Profile::Morton10x10 {
    dims: 10,
    bits_per_dim: 10,
    transform_id: None,
};

// Hilbert10x10: Hilbert curve encoding
// Best for: Maximum locality preservation
let hilbert = Profile::Hilbert10x10 {
    dims: 10,
    bits_per_dim: 10,
    transform_id: None,
};
```

### String Similarity (Rust)

```rust
use elid::*;

// Edit distance
let distance = levenshtein("kitten", "sitting"); // 3

// Normalized similarity (0.0 to 1.0)
let similarity = normalized_levenshtein("hello", "hallo"); // 0.8

// Name matching
let similarity = jaro_winkler("Martha", "Marhta"); // 0.961

// SimHash for database queries
let hash = simhash("iPhone 14");
let sim = simhash_similarity("iPhone 14", "iPhone 15"); // ~0.92

// Find best match in a list
let candidates = vec!["apple", "application", "apply"];
let (idx, score) = find_best_match("app", &candidates);
```

### Python

```python
import elid

# String similarity
elid.levenshtein("kitten", "sitting")  # 3
elid.jaro_winkler("martha", "marhta")  # 0.961
elid.simhash_similarity("iPhone 14", "iPhone 15")  # 0.922

# Embedding encoding (with embeddings feature)
embedding = model.embed("Hello, world!")
elid_str = elid.encode_embedding(embedding)
```

### JavaScript

```javascript
import init, { levenshtein, jaroWinkler, simhashSimilarity } from 'elid';

await init();
levenshtein("kitten", "sitting");  // 3
jaroWinkler("martha", "marhta");   // 0.961
simhashSimilarity("iPhone 14", "iPhone 15");  // 0.922
```

## Configuration

Use `SimilarityOpts` for case-insensitive or whitespace-trimmed comparisons:

```rust
use elid::{levenshtein_with_opts, SimilarityOpts};

let opts = SimilarityOpts {
    case_sensitive: false,
    trim_whitespace: true,
    ..Default::default()
};
let distance = levenshtein_with_opts("  HELLO  ", "hello", &opts); // 0
```

## Feature Flags

| Feature | Description | Dependencies |
|---------|-------------|--------------|
| `strings` | String similarity algorithms (default) | None |
| `embeddings` | Embedding encoding (default) | rand, blake3, etc. |
| `models` | Base ONNX model support | tract-onnx |
| `models-text` | Text embedding (Model2Vec, 256-dim) | models |
| `models-image` | Image embedding (MobileNetV3, 1024-dim) | models, image |
| `wasm` | WebAssembly bindings (includes embeddings) | wasm-bindgen, js-sys, getrandom |
| `python` | Python bindings via PyO3 (includes embeddings) | pyo3, numpy, rayon |
| `ffi` | C FFI bindings | None (enables unsafe) |

## Performance

- Zero external dependencies for string-only use
- O(min(m,n)) space-optimized Levenshtein
- 1.4M+ string comparisons per second (Python benchmarks)
- ~96KB WASM binary (strings only)
- Embedding encoding: <1ms per vector

## Built-in Embedding Models

ELID includes optional ONNX models for generating embeddings directly, without external API calls. Models are bundled via separate packages:

| Package | Model | Dimensions | Size |
|---------|-------|------------|------|
| `elid-text` | Model2Vec potion-base-8M | 256 | ~8MB |
| `elid-image` | MobileNetV3-Small | 1024 | ~5MB |

**Text embeddings:**
```rust
use elid::models::embed_text;

let embedding = embed_text("Hello, world!")?;
assert_eq!(embedding.len(), 256);
```

**Image embeddings:**
```rust
use elid::models::embed_image;

let bytes = std::fs::read("photo.jpg")?;
let embedding = embed_image(&bytes)?;
assert_eq!(embedding.len(), 1024);
```

### LSH Bands for Database Querying

Convert embeddings to LSH bands for efficient database similarity search:

```javascript
import { embeddingToBands } from 'elid';

// Split embedding into 4 bands (32 bits each)
const bands = embeddingToBands(embedding, 4);

// Store bands in database columns
// Query with OR across bands for approximate nearest neighbors:
// SELECT * FROM embeddings WHERE band0 = ? OR band1 = ? OR band2 = ? OR band3 = ?
```

```rust
use elid::embeddings::embedding_to_bands;

let bands = embedding_to_bands(&embedding, 4, 0x454c4944_53494d48);
// bands: Vec<String> with 4 base32hex-encoded band strings
```

## Use Cases

### Vector Search Without Vector Stores

Store ELIDs directly in PostgreSQL, SQLite, or any database:

```sql
-- Create index on ELID column
CREATE INDEX idx_documents_elid ON documents(elid);

-- Find similar documents using string prefix matching
SELECT * FROM documents
WHERE elid LIKE 'abc%'  -- Prefix match for locality
ORDER BY elid;
```

### Deduplication

Use SimHash to find near-duplicate content:

```rust
let hash1 = simhash("The quick brown fox");
let hash2 = simhash("The quick brown dog");
let similarity = simhash_similarity_from_hashes(hash1, hash2);
if similarity > 0.9 {
    println!("Likely duplicates!");
}
```

### Fuzzy Search

Find matches with typo tolerance:

```rust
let candidates = vec!["apple", "application", "apply", "banana"];
let matches = find_matches_above_threshold("aple", &candidates, 0.7);
// Returns: [("apple", 0.8), ...]
```

## Building

```bash
git clone https://github.com/ZachHandley/ELID.git
cd ELID

cargo build --release
cargo test
cargo bench
cargo run --example basic_usage
```

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at your option.
