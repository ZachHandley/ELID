# ELID - Efficient Levenshtein and String Similarity Library

[![CI](https://github.com/ZachHandley/ELID/actions/workflows/ci.yml/badge.svg)](https://github.com/ZachHandley/ELID/actions)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

A fast, zero-dependency Rust library for computing string similarity metrics with bindings for Python, JavaScript (WASM), and C.

## Algorithms

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
[dependencies]
elid = "0.0.0-dev"
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

elid.levenshtein("kitten", "sitting")  # 3
elid.jaro_winkler("martha", "marhta")  # 0.961
elid.simhash_similarity("iPhone 14", "iPhone 15")  # 0.922
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

## Performance

- Zero external dependencies for core algorithms
- O(min(m,n)) space-optimized Levenshtein
- 1.4M+ string comparisons per second (Python benchmarks)
- ~96KB WASM binary

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
