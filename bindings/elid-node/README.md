# ELID - Node.js Bindings

[![npm version](https://img.shields.io/npm/v/elid.svg)](https://www.npmjs.com/package/elid)
[![CI Status](https://github.com/zachhandley/ELID/workflows/Node.js%20Bindings%20CI/badge.svg)](https://github.com/zachhandley/ELID/actions)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](https://opensource.org/licenses/MIT)

High-performance Node.js bindings for **ELID** (Embedding Locality-preserving Identifiers) - convert high-dimensional embeddings into sortable, indexable string identifiers while preserving semantic similarity.

## Features

- **🚀 Blazing Fast**: Native Rust implementation with zero-copy TypedArray access
- **🔒 Type-Safe**: Auto-generated TypeScript definitions from Rust source
- **⚡ Non-Blocking**: Async batch processing prevents event loop blocking
- **🌐 Universal**: Pre-built binaries for Linux, macOS, Windows (x64 + ARM64)
- **🕸️ WASM Fallback**: Browser-compatible WASM version when native addons unavailable
- **📦 Zero Dependencies**: No runtime dependencies, small package size

## Installation

```bash
npm install elid
```

Pre-built native binaries are automatically downloaded for:
- **Linux**: x86_64, aarch64 (glibc)
- **macOS**: x86_64 (Intel), aarch64 (Apple Silicon)
- **Windows**: x86_64

For other platforms, the package will automatically fall back to building from source (requires Rust).

## Quick Start

```typescript
import { encodeElid, ElidProfile } from 'elid';

// Generate an ELID from a 768-dimensional embedding
const embedding = new Float64Array(768).fill(0.5);
const elidId = encodeElid(embedding, ElidProfile.Mini128);

console.log(elidId); // "0123456789ABCDEFGHIJKLMNOPQ" (29 chars)
```

## API Reference

### Profiles

```typescript
enum ElidProfile {
  Mini128 = 0,      // 128-bit SimHash (29 chars) - Best for similarity search
  Morton10x10 = 1,  // 10D Morton curve (16-24 chars) - Fast indexing
  Hilbert10x10 = 2  // 10D Hilbert curve (16-24 chars) - Maximum locality
}
```

**Choosing a Profile:**

- **`Mini128`**: Use for semantic similarity search (cosine/angular distance approximation via Hamming distance)
- **`Morton10x10`**: Use for fast lexicographic sorting and range queries
- **`Hilbert10x10`**: Use when maximizing spatial locality for database B-tree indexes (5-10% better than Morton, but slower)

### `encodeElid(embedding, profile)`

Encode a high-dimensional embedding to a sortable string identifier.

```typescript
function encodeElid(embedding: Float64Array, profile: ElidProfile): string;
```

**Parameters:**
- `embedding`: Float64Array of length 64-2048 (normalized or unnormalized)
- `profile`: Encoding profile (Mini128, Morton10x10, or Hilbert10x10)

**Returns:** Sortable string identifier (base32hex encoded)

**Throws:** Error if embedding dimensions are invalid or encoding fails

**Example:**

```typescript
const embedding = new Float64Array(768);
// Fill with actual embedding values from your model
for (let i = 0; i < 768; i++) {
  embedding[i] = Math.random();
}

const elid = encodeElid(embedding, ElidProfile.Mini128);
console.log(elid.length); // 29
```

### `decodeElid(elid)`

Decode an ELID string back to raw bytes.

```typescript
function decodeElid(elid: string): Buffer;
```

**Parameters:**
- `elid`: ELID string identifier (base32hex encoded)

**Returns:** Buffer containing raw bytes including the 2-byte header (18 bytes for Mini128; 15 bytes for Morton/Hilbert10x10)

**Throws:** Error if ELID string is malformed or invalid encoding

**Example:**

```typescript
const elid = "0123456789ABCDEFGHIJKLMNOPQ";
const bytes = decodeElid(elid);
console.log(bytes.length); // 18 (2 header + 16 payload for Mini128)
console.log(Array.from(bytes)); // [1, 35, 69, 103, ...]
```

### `hammingDistanceElid(elid1, elid2)`

Calculate Hamming distance between two ELIDs (Mini128 profile only).

```typescript
function hammingDistanceElid(elid1: string, elid2: string): number;
```

**Parameters:**
- `elid1`: First ELID string
- `elid2`: Second ELID string (must use same profile as elid1)

**Returns:** Hamming distance (0-128 for Mini128). Lower distance indicates higher similarity.

**Throws:** Error if ELIDs use different profiles or are malformed

**Example:**

```typescript
const embedding1 = new Float64Array(768).fill(0.3);
const embedding2 = new Float64Array(768).fill(0.7);

const elid1 = encodeElid(embedding1, ElidProfile.Mini128);
const elid2 = encodeElid(embedding2, ElidProfile.Mini128);

const distance = hammingDistanceElid(elid1, elid2);
console.log(distance); // 64 (example - actual value depends on embeddings)

// Lower distance = higher similarity
// Distance 0 = identical embeddings
// Distance 128 = maximum dissimilarity
```

### `encodeBatch(embeddings, profile)`

Encode multiple embeddings asynchronously using Tokio for non-blocking execution.

```typescript
async function encodeBatch(embeddings: Float64Array[], profile: ElidProfile): Promise<string[]>;
```

**Parameters:**
- `embeddings`: Array of Float64Array, each length 64-2048
- `profile`: Encoding profile for all embeddings

**Returns:** Promise resolving to array of ELID strings, same order as input

**Throws:** Error if any embedding has invalid dimensions

**Example:**

```typescript
const embeddings = Array.from({ length: 1000 }, (_, i) =>
  new Float64Array(768).fill(i / 1000)
);

const elids = await encodeBatch(embeddings, ElidProfile.Mini128);
console.log(elids.length); // 1000

// Process results
elids.forEach((elid, i) => {
  console.log(`Embedding ${i}: ${elid}`);
});
```

## Use Cases

### 1. Semantic Search with Vector Database

```typescript
import { encodeElid, hammingDistanceElid, ElidProfile } from 'elid';

// Index documents with ELIDs
const documents = [
  { text: "Machine learning basics", embedding: getEmbedding("...") },
  { text: "Deep learning tutorial", embedding: getEmbedding("...") },
  // ... more documents
];

const index = documents.map(doc => ({
  ...doc,
  elid: encodeElid(doc.embedding, ElidProfile.Mini128)
}));

// Query
const queryEmbedding = getEmbedding("neural networks");
const queryElid = encodeElid(queryEmbedding, ElidProfile.Mini128);

// Find similar documents by Hamming distance
const results = index
  .map(doc => ({
    doc,
    distance: hammingDistanceElid(queryElid, doc.elid)
  }))
  .sort((a, b) => a.distance - b.distance)
  .slice(0, 10); // Top 10 results
```

### 2. Database Indexing

```typescript
import { encodeElid, ElidProfile } from 'elid';

// Store ELIDs as sortable strings in database
async function indexEmbedding(embedding: Float64Array) {
  const elid = encodeElid(embedding, ElidProfile.Hilbert10x10);

  await db.execute(
    'INSERT INTO embeddings (elid, data) VALUES (?, ?)',
    [elid, serializeEmbedding(embedding)]
  );

  // Create B-tree index on elid column for fast range queries
  await db.execute('CREATE INDEX idx_elid ON embeddings(elid)');
}

// Range query using lexicographic sorting
async function findNeighbors(targetElid: string, radius: number) {
  const startElid = decrementElid(targetElid, radius);
  const endElid = incrementElid(targetElid, radius);

  return await db.query(
    'SELECT * FROM embeddings WHERE elid BETWEEN ? AND ? ORDER BY elid',
    [startElid, endElid]
  );
}
```

### 3. Batch Processing

```typescript
import { encodeBatch, ElidProfile } from 'elid';

async function processLargeDataset(embeddings: Float64Array[]) {
  // Process in batches to avoid memory pressure
  const batchSize = 1000;
  const results: string[] = [];

  for (let i = 0; i < embeddings.length; i += batchSize) {
    const batch = embeddings.slice(i, i + batchSize);
    const elids = await encodeBatch(batch, ElidProfile.Mini128);
    results.push(...elids);

    console.log(`Processed ${Math.min(i + batchSize, embeddings.length)}/${embeddings.length}`);
  }

  return results;
}
```

## Performance

Benchmarks on standard hardware (Intel i7, 2.8GHz):

| Operation | Throughput | Latency |
|-----------|------------|---------|
| `encodeElid` (768D) | >5,000 embeddings/sec | ~200μs |
| `hammingDistanceElid` | >1,000,000 ops/sec | <1μs |
| `encodeBatch` (1000x768D) | Non-blocking | ~200ms |

**Zero-Copy**: Float64Array is accessed as a buffer view with no data copying.

## WASM Fallback

When native bindings are unavailable (e.g., unsupported platform or browser environment), ELID automatically falls back to WebAssembly:

```typescript
// Automatic fallback in Node.js
import { encodeElid, ElidProfile } from 'elid';

// Same API, uses WASM if native addon fails to load
const elid = encodeElid(embedding, ElidProfile.Mini128);

// Explicit WASM import for browser
import { encodeElid, WasmProfile } from 'elid/wasm';

await init(); // Initialize WASM module
const elid = encodeElid(embedding, WasmProfile.Mini128);
```

## Building from Source

If pre-built binaries are unavailable for your platform:

```bash
# Clone repository
git clone https://github.com/zachhandley/ELID.git
cd ELID/bindings/elid-node

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install dependencies
npm install

# Build native addon
npm run build

# Run tests
npm test
```

## Cross-Language Compatibility

ELID bindings are available for multiple languages with byte-identical output:

- **Rust**: `elid-core` (reference implementation)
- **Python**: `elid-py` (via PyO3)
- **TypeScript/Node.js**: `elid` (via napi-rs)
- **Swift**: `elid-swift` (via UniFFI)
- **Kotlin**: `elid-kotlin` (via UniFFI)
- **Flutter/Dart**: `elid-flutter` (via flutter_rust_bridge)

All bindings pass the same test vectors and produce identical ELIDs for identical inputs.

## Contributing

Contributions welcome! See [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Citation

If you use ELID in research, please cite:

```bibtex
@software{elid2025,
  title = {ELID: Embedding Locality-preserving Identifiers},
  author = {Handley, Zach},
  year = {2025},
  url = {https://github.com/zachhandley/ELID}
}
```

## References

- **SimHash**: Charikar, M. (2002). "Similarity estimation techniques from rounding algorithms"
- **Hilbert Curve**: Hilbert, D. (1891). "Über die stetige Abbildung einer Linie auf ein Flächenstück"
- **Morton Code**: Morton, G. M. (1966). "A computer oriented geodetic data base"

## Support

- 📖 [Documentation](https://github.com/zachhandley/ELID#readme)
- 🐛 [Issue Tracker](https://github.com/zachhandley/ELID/issues)
- 💬 [Discussions](https://github.com/zachhandley/ELID/discussions)
