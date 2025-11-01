# ELID: Embedding Locality IDentifier

**Compact, sortable identifiers for high-dimensional embeddings**

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)
[![Rust: 1.70+](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)

## What is ELID?

ELID encodes high-dimensional embeddings (64-2048 dimensions) into compact, lexicographically sortable string identifiers that preserve locality properties. Think of it as a "ZIP code" for embeddings - similar embeddings get similar IDs.

**Key Features:**

- **Standard database compatibility**: Works with any database supporting string sorting (Postgres, MySQL, SQLite)
- **Fast similarity search**: Hamming distance computation in ~10ns (single CPU instruction)
- **Compact representation**: 24-29 character strings vs 3-8KB for raw embeddings
- **Three encoding profiles**: Choose between speed and locality quality
- **Multiple formats**: CLI supports CSV, Parquet, and JSONL

## Why ELID?

Traditional vector databases require specialized infrastructure. ELID provides a simpler approach:

| Aspect | Traditional Vector DB | ELID |
|--------|----------------------|------|
| Infrastructure | Specialized (Pinecone, Weaviate, etc.) | Any SQL database |
| Similarity Search | Custom index (HNSW, IVF) | B-tree + Hamming distance |
| Storage Format | Binary vectors (3-8KB) | Sortable strings (24-29 chars) |
| Query Method | KNN queries | Range queries + Hamming filter |
| Complexity | High | Low |

**Perfect for:**
- Projects starting with embeddings but not ready for vector DB infrastructure
- Approximate nearest neighbor search with standard databases
- Deduplication and clustering of embedding data
- Caching and indexing semantic search results

## Language Bindings

ELID is available in **7 programming languages** with production-ready bindings:

| Language | Package | Installation | Documentation |
|----------|---------|--------------|---------------|
| 🐍 **Python** | [elid](bindings/elid-python/) | `pip install elid` | [README](bindings/elid-python/README.md) |
| 📘 **TypeScript** | [elid](bindings/elid-node/) | `npm install elid` | [README](bindings/elid-node/README.md) |
| 📱 **Flutter** | [elid](bindings/elid-flutter/) | `flutter pub add elid` | [README](bindings/elid-flutter/README.md) |
| 🍎 **Swift** | [Elid](bindings/elid-swift/) | SPM | [README](bindings/elid-swift/README.md) |
| 🤖 **Kotlin** | [elid](bindings/elid-kotlin/) | Gradle/Maven | [README](bindings/elid-kotlin/README.md) |
| 💎 **Ruby** | [elid](bindings/elid-ruby/) | `gem install elid` | [README](bindings/elid-ruby/README.md) |
| 🐘 **PHP** | [elid/elid](bindings/elid-php/) | `composer require elid/elid` | [README](bindings/elid-php/README.md) |

**All bindings feature:**
- ✅ Identical API across languages
- ✅ Cross-platform compatibility (Linux, macOS, Windows, iOS, Android)
- ✅ Byte-identical encoding (validated with test vectors)
- ✅ Comprehensive test suites (200+ tests total)
- ✅ Full documentation and examples

See [bindings/README.md](bindings/README.md) for complete binding documentation.

### Quick Examples

<details>
<summary><b>Python</b> - Zero-copy NumPy integration</summary>

```python
import elid
import numpy as np

# Encode NumPy array (zero-copy)
embedding = np.random.randn(768).astype(np.float32)
elid_str = elid.encode(embedding, elid.Profile.Mini128)

# Batch processing (parallel with Rayon)
embeddings = [np.random.randn(768).astype(np.float32) for _ in range(1000)]
elids = elid.encode_batch(embeddings, elid.Profile.Mini128)

# Similarity search
distance = elid.hamming_distance(elid1, elid2)
print(f"Distance: {distance}/128")
```
</details>

<details>
<summary><b>TypeScript</b> - Auto-generated types + WASM fallback</summary>

```typescript
import { encode, hammingDistance, Profile } from 'elid';

// Encode with TypeScript type safety
const embedding = new Float32Array(768);
const elid = encode(embedding, Profile.Mini128);

// Async batch processing (non-blocking)
const elids = await encodeBatch(embeddings, Profile.Mini128);

// Fast similarity computation
const distance = hammingDistance(elid1, elid2);
```
</details>

<details>
<summary><b>Flutter</b> - Non-blocking mobile APIs</summary>

```dart
import 'package:elid/elid.dart';

// UI-thread safe encoding
final embedding = List<double>.generate(768, (i) => i / 768.0);
final elid = Elid.encode(embedding, Profile.mini128);

// Async batch processing (doesn't freeze UI)
final embeddings = [embedding1, embedding2, embedding3];
final elids = await Elid.encodeBatch(embeddings, Profile.mini128);

// Hamming distance
final distance = Elid.hammingDistance(elid1, elid2);
```
</details>

<details>
<summary><b>Swift</b> - Native iOS/macOS</summary>

```swift
import Elid

// Encode from CoreML model output
let embedding: [Float] = [...] // 768 dimensions
let elid = try uniEncode(embedding: embedding, profile: .mini128)

// Batch encoding
let elids = try uniEncodeBatch(embeddings: embeddings, profile: .mini128)

// Similarity
let distance = try uniHammingDistance(elid1: elid1, elid2: elid2)
```
</details>

<details>
<summary><b>Kotlin</b> - Android/JVM with Room</summary>

```kotlin
import com.elid.*

// Idiomatic Kotlin extensions
val embedding = List(768) { it / 768.0f }
val elid = embedding.toElid(ElidProfile.MINI_128)

// DSL for batch processing
val elids = buildElidBatch(ElidProfile.MINI_128) {
    add(embedding1)
    add(embedding2)
}

// Room database integration
@Entity
data class Document(@PrimaryKey val elid: String, val content: String)
```
</details>

<details>
<summary><b>Ruby</b> - Rails integration</summary>

```ruby
require 'elid'

# Encode embedding
embedding = Array.new(768) { |i| i / 768.0 }
elid = Elid.encode(embedding, Elid::Profile::MINI128)

# Rails cache
Rails.cache.write("embedding:#{elid}", data)

# Similarity search
distance = Elid.hamming_distance(elid1, elid2)
```
</details>

<details>
<summary><b>PHP</b> - WordPress/Laravel</summary>

```php
<?php
require 'vendor/autoload.php';

use Elid\Elid;

// Encode embedding array
$embedding = array_map(fn($i) => $i / 768.0, range(0, 767));
$elid = Elid::encode($embedding, Elid::MINI128);

// WordPress meta
update_post_meta($post_id, 'elid', $elid);

// Similarity
$distance = Elid::hammingDistance($elid1, $elid2);
```
</details>

## Quick Start

### Library Usage (Rust)

Add to your `Cargo.toml`:

```toml
[dependencies]
elid-core = "0.1"
```

Basic example:

```rust
use elid_core::{encode, hamming_distance, Profile};

// Encode embeddings from ML model (BERT, OpenAI, etc.)
let embedding1 = vec![0.5; 768];
let embedding2 = vec![0.5; 768];

let profile = Profile::default(); // Mini128
let elid1 = encode(&embedding1, &profile)?;
let elid2 = encode(&embedding2, &profile)?;

// Fast similarity check via Hamming distance
let distance = hamming_distance(&elid1, &elid2)?;
println!("Distance: {}/128 (lower = more similar)", distance);
```

See [elid-core documentation](elid-core/README.md) for detailed API docs.

### CLI Usage

Install the CLI tool:

```bash
cargo install --path elid-cli
```

Encode a batch of embeddings:

```bash
elid encode --profile mini128 --input embeddings.csv --output ids.txt
```

Decode ELIDs to inspect them:

```bash
elid decode --input ids.txt --format json
```

See [elid-cli documentation](elid-cli/README.md) for comprehensive usage examples.

## Encoding Profiles

ELID offers three profiles optimized for different use cases:

### 1. Mini128 - Similarity Search (Default)

Best for approximate nearest neighbor search:

```rust
let profile = Profile::default(); // Mini128
```

- **Output**: 29-character string
- **Use for**: Similarity search, deduplication, fuzzy matching
- **Speed**: ~10-50 μs encoding, ~10 ns Hamming distance
- **Locality**: Excellent (preserves angular distance)

### 2. Morton10x10 - Fast Database Indexing

Best for production indexing with standard databases:

```rust
let profile = Profile::Morton10x10 {
    dims: 10,
    bits_per_dim: 10,
    transform_id: None
};
```

- **Output**: 24-character string
- **Use for**: Database primary keys, general indexing
- **Speed**: Very fast (baseline)
- **Locality**: Good (Z-order curve)

### 3. Hilbert10x10 - Maximum Locality

Best for quality-critical applications:

```rust
let profile = Profile::Hilbert10x10 {
    dims: 10,
    bits_per_dim: 10,
    transform_id: None
};
```

- **Output**: 24-character string
- **Use for**: Research, analytics requiring best locality
- **Speed**: 5-10x slower than Morton
- **Locality**: Best (5-10% better than Morton)

**Choosing a profile:**

- Need similarity search? → **Mini128**
- Need fast database indexing? → **Morton10x10**
- Need best possible locality? → **Hilbert10x10**

## Use Cases

### 1. Vector Database with Standard SQL

```sql
-- Store ELIDs as primary keys in Postgres
CREATE TABLE documents (
    elid TEXT PRIMARY KEY,
    content TEXT,
    metadata JSONB
);

CREATE INDEX idx_elid_prefix ON documents(elid varchar_pattern_ops);

-- Range query finds nearby embeddings
SELECT * FROM documents
WHERE elid BETWEEN 'prefix_start' AND 'prefix_end'
ORDER BY elid
LIMIT 100;
```

### 2. Approximate Similarity Search

```rust
// Two-stage search: fast filter + exact scoring
let query_elid = encode(&query_embedding, &profile)?;
let prefix = &query_elid.as_str()[..6];

// Stage 1: Filter by ELID prefix (fast, uses index)
let candidates = db.query("SELECT * FROM docs WHERE elid LIKE ?", prefix);

// Stage 2: Exact cosine similarity on candidates (slower, but fewer)
let results = candidates
    .map(|doc| (doc, cosine_similarity(&query_embedding, &doc.embedding)))
    .sorted_by_score()
    .take(10);
```

### 3. Deduplication

```rust
use std::collections::HashSet;

let mut seen_elids = HashSet::new();
let threshold = 15; // Hamming distance threshold

for embedding in embeddings {
    let elid = encode(&embedding, &profile)?;

    let is_duplicate = seen_elids.iter()
        .any(|seen| hamming_distance(&elid, seen).unwrap() < threshold);

    if !is_duplicate {
        seen_elids.insert(elid);
        // Process unique embedding...
    }
}
```

### 4. CLI Batch Processing

```bash
# Encode 1 million embeddings from CSV
elid encode -p mini128 -i embeddings.csv --progress -o ids.txt

# Import to database
psql -d mydb -c "COPY documents(elid) FROM STDIN" < ids.txt

# Decode sample to verify
head -n 5 ids.txt | elid decode -i - --format json | jq
```

## String Similarity Preservation

ELIDs preserve semantic similarity for text embeddings, making them ideal for natural language processing applications.

### How It Works

When you embed text strings using models like Sentence-BERT (all-MiniLM-L6-v2) or OpenAI's text-embedding models, similar text produces similar embeddings. ELIDs preserve this relationship through Hamming distance.

**Example Results** (from `string_similarity` example):

| Text 1 | Text 2 | Hamming Distance | Cosine Similarity | Correlation |
|--------|--------|------------------|-------------------|-------------|
| "weather is beautiful today" | "gorgeous weather today" | 9 | 0.961 | 96.9% |
| "cat sat on mat" | "cat sat on rug" | 19 | 0.858 | 99.2% |
| "love programming Rust" | "enjoy writing Python" | 23 | 0.779 | 96.1% |
| "cat on mat" | "quantum physics" | 40 | 0.436 | 75.0% |

### Interpreting Hamming Distance

For Mini128 profile (128 bits):

- **0-20**: Very similar (synonym-level similarity)
- **21-40**: Related topics (different wording, same domain)
- **41-60**: Moderate similarity (different topics, some overlap)
- **61-128**: Low similarity (unrelated content)

### Try It Yourself

Run the interactive example with local Sentence-BERT embeddings:

```bash
cd elid-cli
cargo run --example string_similarity --features fastembed
```

**Output includes:**
- Real embeddings from Sentence-BERT
- ELID encoding for each string
- Hamming distance between pairs
- Cosine similarity comparison
- Correlation analysis

### Use Cases for Text

1. **Semantic Search**
   ```rust
   // Find similar documents using ELID prefix search
   let query_elid = encode(&query_embedding, &Profile::default())?;
   let similar_docs = db.query("SELECT * FROM docs WHERE elid LIKE ?",
                                &query_elid.as_str()[..6]);
   ```

2. **Deduplication**
   ```rust
   // Flag near-duplicate content (Hamming distance < 20)
   let is_duplicate = hamming_distance(&elid1, &elid2)? < 20;
   ```

3. **Clustering**
   ```sql
   -- Group similar documents by ELID prefix
   SELECT substr(elid, 1, 6) as cluster, count(*)
   FROM documents
   GROUP BY cluster;
   ```

### Testing

Semantic similarity is validated through:

- **Synthetic tests** (`elid-core/tests/semantic_similarity_tests.rs`): 9 tests with simulated embeddings
- **Real-world example** (`elid-cli/examples/string_similarity.rs`): Live Sentence-BERT embeddings
- **Correlation**: 75-99% accuracy between Hamming distance and cosine similarity

## Project Structure

```
ELID/
├── elid-core/          # Core Rust library (encode, decode, algorithms)
│   ├── src/            # Core implementation
│   ├── examples/       # Runnable examples
│   ├── tests/          # Unit tests
│   └── benches/        # Performance benchmarks
├── elid-cli/           # Command-line tool
│   ├── src/            # CLI implementation
│   └── tests/          # Integration tests
├── bindings/           # Language bindings (7 languages)
│   ├── elid-ffi/       # FFI foundation (C + UniFFI)
│   ├── elid-python/    # Python bindings (PyO3 + maturin)
│   ├── elid-node/      # TypeScript/Node.js (napi-rs + WASM)
│   ├── elid-flutter/   # Flutter/Dart (flutter_rust_bridge)
│   ├── elid-swift/     # Swift (UniFFI)
│   ├── elid-kotlin/    # Kotlin (UniFFI)
│   ├── elid-ruby/      # Ruby (UniFFI)
│   ├── elid-php/       # PHP (FFI)
│   └── README.md       # Bindings overview
├── scripts/            # Build and test scripts
├── .github/workflows/  # CI/CD pipelines (8 workflows)
├── Cargo.toml          # Workspace configuration
├── IMPLEMENTATION_REPORT.md  # Multi-language bindings report
├── CONTRIBUTING.md     # Contribution guidelines
└── README.md           # This file
```

## Documentation

### Library Documentation

For detailed API documentation:

```bash
cd elid-core
cargo doc --open
```

Or see the [elid-core README](elid-core/README.md)

### CLI Documentation

For CLI usage and examples:

```bash
elid --help
elid encode --help
elid decode --help
```

Or see the [elid-cli README](elid-cli/README.md)

### Examples

Run examples to see ELID in action:

```bash
# String similarity with real Sentence-BERT embeddings
cd elid-cli
cargo run --example string_similarity --features fastembed

# Basic usage example
cd elid-core
cargo run --example basic_usage

# Batch encoding with progress
cargo run --example batch_encoding

# Hamming neighbors search
cargo run --example hamming_neighbors
```

## Performance

Benchmarks on typical hardware (Intel i7, single thread):

- **Encoding**: 10-50 μs per 768-dimensional embedding
- **Hamming Distance**: ~10 ns (single CPU instruction)
- **Throughput**: 20,000-100,000 embeddings/sec
- **Memory**: Zero allocations for distance, minimal for encoding

Actual performance varies by:
- Profile choice (Morton is fastest)
- Embedding dimensions (64-2048 supported)
- Hardware capabilities

Run benchmarks yourself:

```bash
cd elid-core
cargo bench
```

## Development

### Building from Source

```bash
# Clone repository
git clone https://github.com/zachhandley/ELID.git
cd ELID

# Build workspace
cargo build --release

# Run tests
cargo test --workspace

# Run examples
cargo run --example basic_usage
```

### Running Tests

```bash
# All tests
cargo test --workspace

# Core library tests only
cd elid-core && cargo test

# CLI tests only
cd elid-cli && cargo test

# With output
cargo test -- --nocapture
```

### Project Status

**Current Version**: 0.1.0 (MVP)

- ✅ Core encoding/decoding: Complete
- ✅ Three profiles: Complete
- ✅ CLI tool: Complete
- ✅ Language bindings: Complete (7 languages)
- ✅ Documentation: Complete
- ✅ Benchmarks: Complete
- ✅ CI/CD: Complete (8 workflows)

**Implementation Report**: See [IMPLEMENTATION_REPORT.md](IMPLEMENTATION_REPORT.md) for complete multi-language bindings implementation details (126 tasks, 200+ tests, 7 languages).

**Roadmap**:

- v0.2: Package publishing (PyPI, npm, pub.dev, Maven Central, RubyGems, Packagist)
- v0.3: PCA transforms for dimensional reduction
- v0.4: Model-specific presets (BERT, OpenAI, etc.)
- v1.0: Stable API, published to crates.io

## Constitution (Optional for MVP)

ELID development follows these principles:

1. **Simplicity over features**: One well-designed feature beats five half-baked ones
2. **Standard tools**: Work with existing databases and ecosystems
3. **Locality preservation**: Maintain spatial relationships in encoded IDs
4. **Performance**: Sub-microsecond operations where possible
5. **Determinism**: Same input always produces same output

See [PROJECT_GUIDE.md](PROJECT_GUIDE.md) for detailed design decisions.

## References

**Academic Foundations:**

- Charikar, M.S. (2002): "Similarity Estimation Techniques from Rounding Algorithms"
  - Theoretical foundation for SimHash locality-sensitive hashing
- Sagan, H. (1994): "Space-Filling Curves"
  - Hilbert curve locality preservation properties

**Implementation Research:**

- [HILBERT_CURVE_RESEARCH.md](HILBERT_CURVE_RESEARCH.md) - Hilbert curve analysis
- [RUST_CLI_RESEARCH.md](RUST_CLI_RESEARCH.md) - CLI design patterns

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! We have implementations in multiple languages:

- **Rust** (core library): `elid-core/` and `elid-cli/`
- **Language bindings**: Python, TypeScript, Flutter, Swift, Kotlin, Ruby, PHP

Please see [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines on:
- Setting up your development environment
- Building and testing bindings
- Code style and conventions
- Submitting pull requests

For major changes, please open an issue first to discuss what you'd like to change.

## Citation

If you use ELID in your research, please cite:

```bibtex
@software{elid2025,
  author = {Handley, Zach},
  title = {ELID: Embedding Locality IDentifier},
  year = {2025},
  url = {https://github.com/zachhandley/ELID}
}
```

## Support

- **Issues**: [GitHub Issues](https://github.com/zachhandley/ELID/issues)
- **Discussions**: [GitHub Discussions](https://github.com/zachhandley/ELID/discussions)
- **Email**: zachhandley@gmail.com

---

**Status**: Under active development - v0.1 MVP complete, API may change before 1.0 release
