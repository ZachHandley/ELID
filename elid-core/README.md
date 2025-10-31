# elid-core

**Compact, sortable identifiers for high-dimensional embeddings**

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](../LICENSE-MIT)
[![Rust: 1.70+](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)

## What is ELID?

ELID (Embedding Locality IDentifier) encodes high-dimensional embeddings (64-2048 dimensions) into compact, lexicographically sortable string identifiers. These identifiers preserve locality properties, making them ideal for:

- **Database indexing**: Use ELIDs as sortable primary keys for vector databases
- **Approximate search**: Filter candidates by ELID prefix before computing exact similarity
- **Deduplication**: Find near-duplicate embeddings via Hamming distance
- **Caching**: Use ELIDs as cache keys for semantic search results

## Why use ELID?

Traditional approaches to indexing embeddings require specialized vector databases or complex HNSW/IVF index structures. ELID provides a simpler alternative:

1. **Standard databases work**: ELIDs are strings that work with any database supporting lexicographic sorting (Postgres, MySQL, SQLite)
2. **Fast similarity search**: Hamming distance computation is ~10ns (single CPU instruction)
3. **Locality preservation**: Similar embeddings have similar ELIDs, enabling range queries
4. **Compact representation**: 24-29 character strings (vs 3-8KB for raw embeddings)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
elid-core = "0.1"
```

## Quick Start (10 lines)

```rust
use elid_core::{encode, hamming_distance, Profile};

// Encode embeddings (typically from ML models like BERT, OpenAI, etc.)
let embedding1 = vec![0.5; 768]; // Example: BERT embeddings
let embedding2 = vec![0.5; 768]; // Similar embedding

let profile = Profile::default(); // Mini128 profile
let elid1 = encode(&embedding1, &profile)?;
let elid2 = encode(&embedding2, &profile)?;

// Compute similarity via Hamming distance (0-128, lower = more similar)
let distance = hamming_distance(&elid1, &elid2)?;
println!("Similar embeddings have distance: {}/128", distance);
```

## Encoding Profiles

ELID supports three encoding profiles, each optimized for different use cases:

### 1. Mini128 (Default) - Similarity Search

**Use for**: Approximate nearest neighbor search via Hamming distance

```rust
let profile = Profile::default(); // Mini128
let elid = encode(&embedding, &profile)?;
```

- **Output**: 29-character string
- **Algorithm**: 128-bit SimHash using signed random projections
- **Best at**: Fast similarity comparisons, deduplication, fuzzy matching
- **Locality**: Preserves angular distance (Charikar 2002)
- **Performance**: ~10-50 μs encoding, ~10 ns Hamming distance

**Use case**: Finding similar documents in a corpus of millions

### 2. Morton10x10 - Fast Database Indexing

**Use for**: Production database indexing with sortable IDs

```rust
let profile = Profile::Morton10x10 {
    dims: 10,          // Use first 10 dimensions
    bits_per_dim: 10,  // 1024 quantization levels per dimension
    transform_id: None // No PCA transform (v0.2+ feature)
};
let elid = encode(&embedding, &profile)?;
```

- **Output**: 24-character string
- **Algorithm**: Z-order curve (Morton code) encoding
- **Best at**: Fast encoding, good spatial locality, range queries
- **Locality**: Good clustering of nearby points
- **Performance**: Fast encoding (baseline)

**Use case**: Primary key for vector database with B-tree indexing

### 3. Hilbert10x10 - Maximum Locality

**Use for**: Quality-critical applications requiring best possible locality

```rust
let profile = Profile::Hilbert10x10 {
    dims: 10,
    bits_per_dim: 10,
    transform_id: None
};
let elid = encode(&embedding, &profile)?;
```

- **Output**: 24-character string
- **Algorithm**: Hilbert curve encoding
- **Best at**: Superior spatial locality (5-10% better than Morton)
- **Tradeoff**: 5-10x slower encoding than Morton
- **Performance**: Higher quality at cost of encoding speed

**Use case**: Research or analytics where locality quality matters most

### Choosing a Profile

| Need | Recommended Profile |
|------|-------------------|
| Similarity search with Hamming distance | **Mini128** |
| Fast production database indexing | **Morton10x10** |
| Best possible locality (slower encoding) | **Hilbert10x10** |
| Approximate search + filtering | **Mini128** |
| Range queries in database | **Morton10x10** or **Hilbert10x10** |

## Performance Characteristics

- **Encoding**: 10-50 μs per 768-dimensional embedding (Mini128)
- **Hamming Distance**: ~10 ns (single CPU instruction)
- **Memory**: Zero allocations for Hamming distance, minimal for encoding
- **Throughput**: ~20,000-100,000 embeddings/sec (single thread)

## Complete Example

See [`examples/basic_usage.rs`](examples/basic_usage.rs) for a complete working example:

```bash
cargo run --example basic_usage
```

Or for batch processing with progress tracking:

```bash
cargo run --example batch_encoding
```

## Documentation

For comprehensive API documentation:

```bash
cargo doc --open
```

Or visit [docs.rs/elid-core](https://docs.rs/elid-core) (when published)

## Advanced Features

### Hamming Distance for Similarity Search

```rust
use elid_core::{encode, hamming_distance, Profile};

let profile = Profile::default();
let elid1 = encode(&embedding1, &profile)?;
let elid2 = encode(&embedding2, &profile)?;

let distance = hamming_distance(&elid1, &elid2)?;

// Interpretation (for normalized embeddings):
// 0-30:   Very similar (cosine similarity > 0.85)
// 31-64:  Somewhat similar (cosine similarity 0.5-0.85)
// 65-128: Dissimilar (cosine similarity < 0.5)
```

### Approximate Cosine Similarity

```rust
use elid_core::simhash::cosine_similarity_approx;

// Extract SimHash from ELID bytes
let bytes1 = elid1.to_bytes()?;
let hash1 = u128::from_be_bytes(bytes1[2..18].try_into().unwrap());

let bytes2 = elid2.to_bytes()?;
let hash2 = u128::from_be_bytes(bytes2[2..18].try_into().unwrap());

// Approximate cosine similarity: -1.0 to 1.0
let similarity = cosine_similarity_approx(hash1, hash2);
println!("Approximate cosine similarity: {:.3}", similarity);
```

### Finding Similar ELIDs

```rust
use elid_core::simhash::hamming_neighbors;

// Find all ELIDs within Hamming distance 10
let query_elid = encode(&query_embedding, &profile)?;
let candidates: Vec<&Elid> = vec![&elid1, &elid2, &elid3];

let neighbors = hamming_neighbors(&query_elid, &candidates, 10)?;
println!("Found {} similar items", neighbors.len());
```

## Use Cases

### Vector Database Indexing

```rust
// Store ELIDs as primary keys in database
let elid = encode(&embedding, &Profile::Morton10x10 {
    dims: 10,
    bits_per_dim: 10,
    transform_id: None
})?;

// SQL example (Postgres)
// CREATE TABLE embeddings (
//     elid TEXT PRIMARY KEY,
//     content TEXT,
//     metadata JSONB
// );
//
// CREATE INDEX idx_elid_prefix ON embeddings(elid varchar_pattern_ops);
//
// -- Range query to find nearby embeddings
// SELECT * FROM embeddings
// WHERE elid BETWEEN 'prefix_start' AND 'prefix_end'
// ORDER BY elid
// LIMIT 100;
```

### Deduplication

```rust
use std::collections::HashSet;

let mut seen_elids = HashSet::new();
let threshold = 15; // Hamming distance threshold

for embedding in embeddings {
    let elid = encode(&embedding, &profile)?;

    // Check if similar ELID already exists
    let is_duplicate = seen_elids.iter().any(|existing| {
        hamming_distance(&elid, existing).unwrap() < threshold
    });

    if !is_duplicate {
        seen_elids.insert(elid);
        // Process unique embedding...
    }
}
```

### Two-Stage Approximate Search

```rust
// Stage 1: Filter candidates by ELID prefix (fast)
let query_elid = encode(&query_embedding, &profile)?;
let prefix = &query_elid.as_str()[..6]; // Use first 6 characters

// SELECT * FROM embeddings WHERE elid LIKE 'prefix%'
let candidates = filter_by_prefix(prefix);

// Stage 2: Compute exact similarity only for candidates (slower, but fewer)
let results = candidates.iter()
    .map(|candidate| {
        let exact_similarity = cosine_similarity(&query_embedding, &candidate.embedding);
        (candidate, exact_similarity)
    })
    .sorted_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap())
    .take(10)
    .collect();
```

## Error Handling

All operations return `Result<T, ElidError>`:

```rust
use elid_core::ElidError;

match encode(&embedding, &profile) {
    Ok(elid) => println!("Success: {}", elid),
    Err(ElidError::InvalidDimension { got, min, max }) => {
        eprintln!("Dimension {} not in range [{}, {}]", got, min, max);
    }
    Err(ElidError::InvalidValue) => {
        eprintln!("Embedding contains NaN or Inf");
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## References

- **Charikar, M.S. (2002)**: "Similarity Estimation Techniques from Rounding Algorithms"
  - Theoretical foundation for SimHash locality-sensitive hashing
- **Sagan, H. (1994)**: "Space-Filling Curves"
  - Hilbert curve locality preservation properties

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! This project is part of the ELID v0.1 MVP.

---

**Status**: Under active development - API may change before 1.0 release
