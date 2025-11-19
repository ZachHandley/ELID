# ELID - Efficient Levenshtein and String Similarity Library

[![Rust](https://img.shields.io/badge/rust-2021-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

A fast, zero-dependency Rust library for computing various string similarity metrics. ELID provides efficient implementations of classic algorithms like Levenshtein distance, Jaro-Winkler similarity, and more.

## Features

- 🚀 **Fast**: Optimized implementations with minimal allocations
- 📦 **Zero dependencies**: Pure Rust, no external crates required for core functionality
- 🔧 **Multiple algorithms**: Levenshtein, Jaro-Winkler, Hamming, OSA
- 🌍 **Unicode support**: Full support for Unicode strings
- 🎯 **Easy to use**: Simple, well-documented API
- ⚡ **Query-time ready**: Suitable for real-time search and matching

## Algorithms

### Levenshtein Distance
The classic edit distance metric. Counts the minimum number of single-character edits (insertions, deletions, or substitutions) needed to transform one string into another.

**Best for**: General-purpose string comparison, spell checking

### Jaro-Winkler Similarity
A variant of the Jaro similarity that gives more favorable ratings to strings with common prefixes.

**Best for**: Short strings like names, record linkage

### Hamming Distance
Counts the number of positions at which corresponding characters differ. Only works with equal-length strings.

**Best for**: Fixed-length strings, DNA sequences, error detection codes

### OSA Distance (Optimal String Alignment)
Similar to Levenshtein but also considers transpositions (swapping of adjacent characters) as a single operation.

**Best for**: Detecting typos, keyboard errors

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
elid = "0.1.0"
```

## Quick Start

```rust
use elid::*;

fn main() {
    // Levenshtein distance
    let distance = levenshtein("kitten", "sitting");
    println!("Distance: {}", distance); // Output: 3

    // Normalized similarity (0.0 to 1.0)
    let similarity = normalized_levenshtein("hello", "hallo");
    println!("Similarity: {:.2}", similarity); // Output: 0.80

    // Jaro-Winkler similarity (great for names)
    let similarity = jaro_winkler("Martha", "Marhta");
    println!("Name similarity: {:.3}", similarity); // Output: 0.961

    // Find best match in a list
    let candidates = vec!["apple", "application", "apply"];
    let (idx, score) = find_best_match("app", &candidates);
    println!("Best match: {} (score: {:.3})", candidates[idx], score);
}
```

## Usage Examples

### Basic Distance Calculation

```rust
use elid::levenshtein;

let distance = levenshtein("kitten", "sitting");
assert_eq!(distance, 3);
```

### Normalized Similarity

```rust
use elid::normalized_levenshtein;

let similarity = normalized_levenshtein("hello", "hallo");
assert!(similarity > 0.7); // Returns value between 0.0 and 1.0
```

### Finding Matches

```rust
use elid::find_best_match;

let candidates = vec!["apple", "application", "apply", "banana"];
let (idx, score) = find_best_match("app", &candidates);
println!("Best match: {}", candidates[idx]);
```

### Filtering by Threshold

```rust
use elid::find_matches_above_threshold;

let candidates = vec!["apple", "application", "apply", "banana"];
let matches = find_matches_above_threshold("app", &candidates, 0.5);

for (idx, score) in matches {
    println!("{}: {:.2}", candidates[idx], score);
}
```

### Case-Insensitive and Trimmed Comparison

```rust
use elid::{levenshtein_with_opts, SimilarityOpts};

let opts = SimilarityOpts {
    case_sensitive: false,
    trim_whitespace: true,
    ..Default::default()
};

let distance = levenshtein_with_opts("  HELLO  ", "hello", &opts);
assert_eq!(distance, 0);
```

### Jaro-Winkler for Name Matching

```rust
use elid::jaro_winkler;

let similarity = jaro_winkler("John Smith", "Jon Smith");
assert!(similarity > 0.8);
```

### Hamming Distance for Fixed-Length Strings

```rust
use elid::hamming;

// DNA sequence comparison
let distance = hamming("ACGTACGT", "ACGTACCT");
assert_eq!(distance, Some(1));

// Returns None for different lengths
assert_eq!(hamming("hello", "world!"), None);
```

### OSA Distance for Transpositions

```rust
use elid::osa_distance;

// Detects transposition as single operation
let distance = osa_distance("ca", "ac");
assert_eq!(distance, 1);
```

## Real-World Use Cases

### Product Search

```rust
use elid::{best_match, SimilarityOpts};

fn search_products(query: &str, products: &[&str]) -> Vec<(usize, f64)> {
    let mut scored: Vec<_> = products
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let score = best_match(&query.to_lowercase(), &p.to_lowercase());
            (i, score)
        })
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    scored
}

let products = vec!["iPhone 14 Pro", "Samsung Galaxy S23", "Google Pixel 7"];
let results = search_products("iphone", &products);
```

### Spell Checking

```rust
use elid::find_best_match;

fn suggest_correction(word: &str, dictionary: &[&str]) -> &str {
    let (idx, _) = find_best_match(word, dictionary);
    dictionary[idx]
}

let dictionary = vec!["receive", "believe", "achieve"];
let suggestion = suggest_correction("recieve", &dictionary);
assert_eq!(suggestion, "receive");
```

### Name Deduplication

```rust
use elid::jaro_winkler;

fn is_duplicate_name(name1: &str, name2: &str) -> bool {
    jaro_winkler(name1, name2) > 0.9
}

assert!(is_duplicate_name("John Smith", "Jon Smith"));
```

## Performance

ELID is designed for performance:

- **Space-optimized**: Levenshtein uses O(min(m,n)) space instead of O(m*n)
- **Minimal allocations**: Algorithms minimize heap allocations
- **Pure Rust**: No FFI overhead, benefits from LLVM optimizations
- **Release mode**: Compile with `--release` for maximum performance

## API Reference

### Main Functions

- `levenshtein(a, b)` - Calculate Levenshtein distance
- `normalized_levenshtein(a, b)` - Get normalized similarity (0.0-1.0)
- `jaro(a, b)` - Calculate Jaro similarity
- `jaro_winkler(a, b)` - Calculate Jaro-Winkler similarity
- `hamming(a, b)` - Calculate Hamming distance (equal-length strings)
- `osa_distance(a, b)` - Calculate OSA distance

### Helper Functions

- `best_match(a, b)` - Get best score from multiple algorithms
- `find_best_match(query, candidates)` - Find best matching candidate
- `find_matches_above_threshold(query, candidates, threshold)` - Filter by threshold

### Configuration

Use `SimilarityOpts` to configure comparison behavior:

```rust
pub struct SimilarityOpts {
    pub case_sensitive: bool,      // Default: true
    pub trim_whitespace: bool,     // Default: false
    pub prefix_scale: f64,         // Default: 0.1 (for Jaro-Winkler)
}
```

## Use at Query Time

ELID is designed to be fast enough for query-time usage:

```rust
// Example: Real-time search as user types
fn handle_search_input(query: &str, database: &[&str]) -> Vec<String> {
    find_matches_above_threshold(query, database, 0.6)
        .iter()
        .take(10)
        .map(|(idx, _)| database[*idx].to_string())
        .collect()
}
```

For very large datasets, consider:
1. Pre-filtering with simpler metrics (length, first character)
2. Using an index structure (BK-tree, VP-tree)
3. Parallel processing with rayon
4. Caching frequent queries

## Building from Source

```bash
# Clone the repository
git clone https://github.com/ZachHandley/ELID.git
cd ELID

# Build
cargo build --release

# Run tests
cargo test

# Run examples
cargo run --example basic_usage

# Run benchmarks (requires nightly)
cargo bench
```

## Testing

ELID has comprehensive test coverage:

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run integration tests only
cargo test --test integration_tests
```

## Examples

Run the included examples:

```bash
cargo run --example basic_usage
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Acknowledgments

- Levenshtein distance algorithm based on Wagner-Fischer algorithm
- Jaro-Winkler implementation based on the original paper by William E. Winkler
- Inspired by similar libraries in other languages (strsim, jellyfish)

## Future Plans

- [ ] Add more algorithms (Sørensen-Dice, Jaccard, etc.)
- [ ] Phonetic algorithms (Soundex, Metaphone)
- [ ] SIMD optimizations
- [ ] Optional parallel processing
- [ ] Python bindings via PyO3
- [ ] WebAssembly support

## Citation

If you use ELID in your research, please cite:

```bibtex
@software{elid,
  title = {ELID: Efficient Levenshtein and String Similarity Library},
  author = {ELID Contributors},
  year = {2024},
  url = {https://github.com/ZachHandley/ELID}
}
```
