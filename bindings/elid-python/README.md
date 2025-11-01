# ELID Python Bindings

High-performance Python bindings for ELID (Embedding Locality IDentifier) using PyO3.

## Features

- **Zero-copy NumPy integration**: Direct access to NumPy arrays without memory duplication
- **Parallel batch encoding**: Process multiple embeddings using Rayon's parallel iterators
- **Full API support**: `encode()`, `decode()`, `hamming_distance()`, `encode_batch()`
- **Type hints**: Complete `.pyi` stub file for IDE autocomplete and type checking
- **Performance**: >5000 embeddings/sec for 768-dimensional vectors on standard hardware

## Installation

### From PyPI (when published)

```bash
pip install elid
```

### From source

```bash
cd bindings/elid-python
pip install maturin
maturin develop --release
```

## Quick Start

```python
import elid
import numpy as np

# Create an embedding (e.g., from BERT, OpenAI, etc.)
embedding = np.random.randn(768)

# Encode with Mini128 profile (default, optimized for similarity search)
profile = elid.Profile("Mini128")
elid_str = elid.encode(embedding, profile)

print(f"ELID: {elid_str}")  # 29-character sortable string
print(f"Length: {len(elid_str)}")  # 29

# Compute similarity via Hamming distance
embedding2 = embedding + np.random.randn(768) * 0.1  # Similar embedding
elid2 = elid.encode(embedding2, profile)

distance = elid.hamming_distance(elid_str, elid2)
print(f"Hamming distance: {distance}/128")  # Low distance = high similarity
```

## API Reference

### Profiles

ELID supports three encoding profiles:

- **`Mini128`**: 128-bit SimHash using signed random projections (29 characters)
  - Best for: Approximate nearest neighbor search via Hamming distance
  - Use case: Similarity search, deduplication, fuzzy matching

- **`Morton10x10`**: Z-order curve encoding (24 characters)
  - Best for: Fast database indexing with good spatial locality
  - Use case: Database primary keys, range queries

- **`Hilbert10x10`**: Hilbert curve encoding (24 characters)
  - Best for: Maximum locality preservation (5-10% better than Morton)
  - Use case: Quality-critical applications, slower encoding

### Functions

#### `encode(embedding, profile) -> str`

Encode a single embedding into an ELID string.

**Arguments:**
- `embedding`: NumPy array of shape `(N,)` where `64 ≤ N ≤ 2048`
- `profile`: Encoding profile (Mini128, Morton10x10, or Hilbert10x10)

**Returns:**
- Sortable string identifier (29 chars for Mini128, ~24 for others)

**Raises:**
- `ValueError`: If embedding dimensions are invalid or encoding fails

**Example:**
```python
embedding = np.random.randn(768)
elid_str = elid.encode(embedding, elid.Profile("Mini128"))
```

#### `decode(elid_str) -> bytes`

Decode an ELID string back to raw bytes.

**Arguments:**
- `elid_str`: ELID string identifier (base32hex encoded)

**Returns:**
- Raw bytes (18 bytes for Mini128: 2 header + 16 payload)

**Raises:**
- `ValueError`: If ELID string is malformed

**Example:**
```python
raw_bytes = elid.decode(elid_str)
print(f"Raw bytes: {len(raw_bytes)}")  # 18 for Mini128
```

#### `hamming_distance(elid1, elid2) -> int`

Compute Hamming distance between two ELIDs (Mini128 only).

**Arguments:**
- `elid1`: First ELID string
- `elid2`: Second ELID string (must use Mini128 profile)

**Returns:**
- Hamming distance (0-128 for Mini128). Lower distance = higher similarity.

**Raises:**
- `ValueError`: If ELIDs use different profiles or are malformed

**Example:**
```python
distance = elid.hamming_distance(elid1, elid2)
print(f"Hamming distance: {distance}/128")
```

#### `encode_batch(embeddings, profile) -> List[str]`

Encode multiple embeddings in parallel using Rayon.

**Arguments:**
- `embeddings`: List of NumPy arrays, each shape `(N,)` where `64 ≤ N ≤ 2048`
- `profile`: Encoding profile for all embeddings

**Returns:**
- List of ELID strings, same order as input

**Raises:**
- `ValueError`: If any embedding has invalid dimensions

**Example:**
```python
embeddings = [np.random.randn(768) for _ in range(1000)]
elids = elid.encode_batch(embeddings, elid.Profile("Mini128"))
print(f"Encoded {len(elids)} embeddings")
```

## Performance

Benchmark results on AMD Ryzen 9 5950X (16 cores):

```
Single encoding:     ~10-50 μs per 768D embedding (Mini128)
Batch encoding:      >20,000 embeddings/sec (parallel with Rayon)
Hamming distance:    ~10 ns per comparison
Memory overhead:     Zero-copy NumPy access (no duplication)
```

## Advanced Usage

### Batch Processing Large Datasets

```python
import elid
import numpy as np

# Load embeddings from your vector database
embeddings = load_embeddings()  # Returns list of NumPy arrays

# Encode in batches for optimal throughput
batch_size = 1000
profile = elid.Profile("Mini128")

all_elids = []
for i in range(0, len(embeddings), batch_size):
    batch = embeddings[i:i+batch_size]
    elids = elid.encode_batch(batch, profile)
    all_elids.extend(elids)

print(f"Encoded {len(all_elids)} embeddings")
```

### Database Integration

```python
import elid
import numpy as np
import sqlite3

# Create table with ELID primary key
conn = sqlite3.connect("embeddings.db")
conn.execute("""
    CREATE TABLE documents (
        elid TEXT PRIMARY KEY,
        content TEXT,
        embedding BLOB
    )
""")

# Insert documents with ELID keys
profile = elid.Profile("Morton10x10")  # Sortable for range queries
for doc in documents:
    embedding = get_embedding(doc["content"])
    elid_str = elid.encode(embedding, profile)

    conn.execute(
        "INSERT INTO documents (elid, content, embedding) VALUES (?, ?, ?)",
        (elid_str, doc["content"], embedding.tobytes())
    )

conn.commit()

# Range query using ELID sortability
query_embedding = get_embedding("search query")
query_elid = elid.encode(query_embedding, profile)

# Find nearby ELIDs (approximate nearest neighbors)
results = conn.execute("""
    SELECT elid, content FROM documents
    WHERE elid BETWEEN ? AND ?
    ORDER BY elid
    LIMIT 100
""", (query_elid[:10], query_elid[:10] + "z")).fetchall()
```

## Development

### Building from source

```bash
# Install Rust and Python dependencies
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
pip install maturin pytest numpy

# Build and install in development mode
cd bindings/elid-python
maturin develop --release

# Run tests
pytest tests/ -v
```

### Running tests

```bash
# Run all tests
pytest tests/ -v

# Run specific test
pytest tests/test_encoding.py::test_encode_batch_performance -v -s

# Run with coverage
pip install pytest-cov
pytest tests/ --cov=elid --cov-report=html
```

### Building wheels

```bash
# Build wheel for current platform
maturin build --release

# Build manylinux wheels (requires Docker)
docker run --rm -v $(pwd):/io ghcr.io/pyo3/maturin build --release

# Wheels will be in target/wheels/
```

## Contributing

Contributions are welcome! Please:

1. Run tests: `pytest tests/ -v`
2. Format code: `cargo fmt`
3. Run clippy: `cargo clippy`
4. Add tests for new features

## License

Dual-licensed under MIT or Apache 2.0.
