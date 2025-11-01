# ELID Kotlin

Kotlin bindings for ELID (Embedding Locality-preserving IDentifiers) - a library for encoding high-dimensional embeddings into compact, sortable string identifiers.

## Features

- **Zero-copy FFI**: Uses UniFFI-generated bindings with JNA for efficient native library integration
- **Type-safe API**: Idiomatic Kotlin with sealed classes for error handling
- **Multiple encoding profiles**:
  - `MINI_128`: 128-bit SimHash for semantic similarity (26-char base32hex)
  - `MORTON_10X10`: 10D Morton curve for spatial indexing (27-char base32hex)
  - `HILBERT_10X10`: 10D Hilbert curve for optimal locality (27-char base32hex)
- **Batch processing**: Parallel encoding of multiple embeddings
- **Cross-platform**: Supports Linux (x86_64, ARM64), macOS (x86_64, ARM64), and Windows (x86_64)

## Installation

### Gradle (Kotlin DSL)

```kotlin
dependencies {
    implementation("com.elid:elid-kotlin:0.1.0")
}
```

### Gradle (Groovy)

```groovy
dependencies {
    implementation 'com.elid:elid-kotlin:0.1.0'
}
```

### Maven

```xml
<dependency>
    <groupId>com.elid</groupId>
    <artifactId>elid-kotlin</artifactId>
    <version>0.1.0</version>
</dependency>
```

## Quick Start

### Basic Encoding

```kotlin
import com.elid.*

// Create an embedding (384 dimensions)
val embedding = List(384) { it.toFloat() / 384.0f }

// Encode with Mini128 (semantic similarity)
val elid = embedding.toElid(ElidProfile.MINI_128)
println(elid) // "0ab12cd34ef56..."

// Decode back to bytes
val bytes = elid.decodeElid()
```

### Using the Elid Data Class

```kotlin
// Create an Elid object
val elid1 = Elid.from(embedding1, ElidProfile.MINI_128)
val elid2 = Elid.from(embedding2, ElidProfile.MINI_128)

// Compute Hamming distance (Mini128 only)
val distance = elid1.hammingDistanceTo(elid2)

// Check if nearby (within threshold)
if (elid1.isNearby(elid2, threshold = 10u)) {
    println("Embeddings are similar!")
}
```

### Batch Processing

```kotlin
val embeddings = listOf(
    List(384) { it.toFloat() / 384.0f },
    List(384) { (it + 50).toFloat() / 384.0f },
    List(384) { (it + 100).toFloat() / 384.0f }
)

// Encode batch (parallel processing)
val elids = encodeBatch(embeddings, ElidProfile.MINI_128)

// Or use the builder DSL
val elidObjects = buildElidBatchToElids(ElidProfile.MINI_128) {
    add(embedding1)
    add(embedding2)
    add(embedding3)
}
```

### Error Handling

```kotlin
// Exception-based (throws UniElidException)
try {
    val elid = embedding.toElid(ElidProfile.MINI_128)
} catch (e: UniElidException.InvalidDimension) {
    println("Invalid dimension: ${e.message}")
}

// Result-based (no exceptions)
when (val result = embedding.toElidOrNull(ElidProfile.MINI_128)) {
    is ElidResult.Success -> println("ELID: ${result.value}")
    is ElidResult.Failure -> println("Error: ${result.error.message}")
}
```

## API Reference

### Core Functions

#### `uniEncode(embedding: List<Float>, profile: UniProfile): String`
Encode a single embedding into an ELID string.

**Parameters:**
- `embedding`: List of floats (64-2048 dimensions)
- `profile`: Encoding profile (MINI128, MORTON10X10, HILBERT10X10)

**Returns:** Base32hex-encoded ELID string

**Throws:**
- `UniElidException.InvalidDimension` - Dimension out of range (64-2048)
- `UniElidException.InvalidValue` - NaN or infinite values

#### `uniDecode(elid: String): List<UByte>`
Decode an ELID string to raw bytes.

**Returns:** List of unsigned bytes including 2‑byte header (18 bytes for Mini128; 15 bytes for 10×10 profiles)

**Throws:**
- `UniElidException.InvalidEncoding` - Malformed ELID string
- `UniElidException.InvalidHeader` - Invalid profile header

#### `uniHammingDistance(elid1: String, elid2: String): UInt`
Compute Hamming distance between two Mini128 ELIDs (0-128).

**Throws:**
- `UniElidException.ProfileMismatch` - ELIDs use different profiles

#### `uniEncodeBatch(embeddings: List<List<Float>>, profile: UniProfile): List<String>`
Encode multiple embeddings in parallel.

### Extension Functions

#### `FloatArray.toElid(profile: ElidProfile): String`
#### `List<Float>.toElid(profile: ElidProfile): String`
Encode an embedding array/list to an ELID.

#### `String.decodeElid(): ByteArray`
Decode an ELID string to a byte array.

#### `String.hammingDistanceTo(other: String): UInt`
Compute Hamming distance to another ELID string.

### Profiles

#### `ElidProfile.MINI_128`
- **Use case**: Semantic search, deduplication, nearest neighbors
- **Output**: 26-char base32hex starting with `0`
- **Properties**: Hamming distance approximates cosine similarity
- **Example**: `"0ab12cd34ef56gh78ij90kl12mn"`

#### `ElidProfile.MORTON_10X10`
- **Use case**: Multi-dimensional range queries, spatial indexing
- **Output**: 27-char base32hex starting with `g`
- **Properties**: Z-order curve, good clustering
- **Example**: `"gab12cd34ef56gh78ij90kl12mno"`

#### `ElidProfile.HILBERT_10X10`
- **Use case**: Optimal spatial locality, better clustering than Morton
- **Output**: 27-char base32hex starting with `o`
- **Properties**: Hilbert curve, 5-10% better locality
- **Example**: `"oab12cd34ef56gh78ij90kl12mno"`

## Examples

### Database Integration (Room)

```kotlin
@Entity(tableName = "embeddings")
data class EmbeddingEntity(
    @PrimaryKey val id: String = UUID.randomUUID().toString(),
    @ColumnInfo(name = "elid") val elid: String,
    @ColumnInfo(name = "profile") val profile: String,
    @ColumnInfo(name = "metadata") val metadata: String
)

@Dao
interface EmbeddingDao {
    @Query("SELECT * FROM embeddings WHERE elid >= :start AND elid < :end ORDER BY elid")
    fun findByElidRange(start: String, end: String): List<EmbeddingEntity>

    @Insert
    fun insert(entity: EmbeddingEntity)
}

// Usage
val embedding = getEmbeddingVector() // Get your embedding
val elid = Elid.from(embedding, ElidProfile.MINI_128)

// Insert
dao.insert(EmbeddingEntity(
    elid = elid.value,
    profile = "MINI_128",
    metadata = "{}"
))

// Range query (lexicographic sorting preserves locality)
val results = dao.findByElidRange("0a", "0b")
```

### Nearest Neighbor Search

```kotlin
fun findSimilar(
    query: List<Float>,
    database: List<Pair<String, List<Float>>>,
    threshold: UInt = 20u
): List<String> {
    val queryElid = Elid.from(query, ElidProfile.MINI_128)

    return database
        .map { (id, embedding) -> id to Elid.from(embedding, ElidProfile.MINI_128) }
        .filter { (_, elid) -> queryElid.isNearby(elid, threshold) }
        .map { (id, _) -> id }
}
```

### Batch Processing with Coroutines

```kotlin
suspend fun encodeEmbeddingsAsync(
    embeddings: List<List<Float>>,
    profile: ElidProfile
): List<Elid> = withContext(Dispatchers.Default) {
    embeddings.chunked(1000).flatMap { chunk ->
        async {
            encodeBatch(chunk, profile).map { Elid(it, profile) }
        }
    }.awaitAll().flatten()
}
```

## Performance

Benchmarks on Apple M1 (8-core):

| Operation | Throughput | Notes |
|-----------|-----------|-------|
| Single encode (Mini128) | ~50,000 ops/sec | 384-dimensional embedding |
| Batch encode (100 embeddings) | ~500,000 ops/sec | 10x faster than sequential |
| Hamming distance | ~2,000,000 ops/sec | Pure integer operations |
| Decode | ~100,000 ops/sec | Base32hex decoding |

## Error Types

```kotlin
sealed class UniElidException : Exception() {
    class InvalidDimension : UniElidException()  // Dimension not in 64-2048
    class InvalidValue : UniElidException()      // NaN or Inf in embedding
    class InvalidEncoding : UniElidException()   // Malformed ELID string
    class InvalidHeader : UniElidException()     // Invalid profile header byte
    class ProfileMismatch : UniElidException()   // Different profiles in comparison
    class TransformNotFound : UniElidException() // Internal error
}
```

## Building from Source

### Prerequisites

- JDK 11 or higher
- Rust 1.70+ with `cargo`
- `uniffi-bindgen` CLI tool

### Build Steps

```bash
# Install uniffi-bindgen
cargo install uniffi-bindgen --version 0.28.0

# Build Rust library
cd bindings/elid-ffi
cargo build --release

# Generate Kotlin bindings
uniffi-bindgen generate src/elid.udl --language kotlin \
  --out-dir ../elid-kotlin/src/main/kotlin/

# Build Kotlin library
cd ../elid-kotlin
./gradlew build

# Run tests
./gradlew test
```

## Native Library Loading

The library automatically loads the native library (`libelid_ffi.so`, `libelid_ffi.dylib`, or `elid_ffi.dll`) using JNA. Ensure the native library is in your `java.library.path` or bundled in your JAR.

For production deployments, place the native library in:
- Linux: `/usr/lib` or `/usr/local/lib`
- macOS: `/usr/local/lib`
- Windows: System PATH or application directory

## Cross-Platform Support

| Platform | Architectures | Native Library |
|----------|--------------|----------------|
| Linux | x86_64, ARM64 | `libelid_ffi.so` |
| macOS | x86_64, ARM64 (M1/M2) | `libelid_ffi.dylib` |
| Windows | x86_64 | `elid_ffi.dll` |

## Limitations

- **Dimension range**: 64-2048 dimensions only
- **Hamming distance**: Only for Mini128 profile
- **Profile detection**: Based on first character (not validated)
- **No normalization**: Embeddings are not automatically normalized

## License

Dual-licensed under MIT or Apache 2.0.

## Contributing

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

## Related Projects

- [elid-core](../../elid-core) - Core Rust implementation
- [elid-python](../elid-python) - Python bindings (PyO3)
- [elid-node](../elid-node) - Node.js bindings (NAPI-RS)
- [elid-ffi](../elid-ffi) - UniFFI interface definition

## References

- [SimHash paper](https://www.cs.princeton.edu/courses/archive/spring04/cos598B/bib/CharikarEstim.pdf) - Charikar (2002)
- [Morton encoding](https://en.wikipedia.org/wiki/Z-order_curve) - Z-order curve
- [Hilbert curve](https://en.wikipedia.org/wiki/Hilbert_curve) - Space-filling curve
- [UniFFI](https://mozilla.github.io/uniffi-rs/) - Unified FFI bindings generator
