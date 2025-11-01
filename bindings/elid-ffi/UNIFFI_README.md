# UniFFI Foundation Layer for ELID

This layer provides automatic Swift, Kotlin, and Ruby bindings through Mozilla's UniFFI framework.

## Build Status

✅ **elid-ffi library built successfully**
- Shared library: `target/release/libelid_ffi.so` (509KB)
- Static library: `target/release/libelid_ffi.a` (7.4MB)
- UniFFI scaffolding: Embedded in library

## Architecture

### Files

- **src/elid.udl**: UniFFI interface definition (defines the API contract)
- **src/lib.rs**: Rust implementation with C exports (PHP) and UniFFI exports (Swift/Kotlin/Ruby)
- **build.rs**: Generates UniFFI scaffolding at compile time
- **uniffi.toml**: Configuration for binding generation
- **Cargo.toml**: Dependencies (uniffi 0.25, thiserror 1.0)

### API Surface (UniFFI)

All functions are exposed through the `elid_ffi` namespace:

#### Functions

1. **uni_encode(embedding: Vec<f32>, profile: UniProfile) -> Result<String, UniElidError>**
   - Encodes an embedding (64-2048 dimensions) into an ELID string

2. **uni_decode(elid: String) -> Result<Vec<u8>, UniElidError>**
   - Decodes an ELID string to raw bytes

3. **uni_hamming_distance(elid1: String, elid2: String) -> Result<u32, UniElidError>**
   - Computes Hamming distance between two Mini128 ELIDs (0-128)

4. **uni_encode_batch(embeddings: Vec<Vec<f32>>, profile: UniProfile) -> Result<Vec<String>, UniElidError>**
   - Batch encodes multiple embeddings

#### Types

**UniProfile enum:**
- `Mini128`: 128-bit SimHash (29 char ELID, supports Hamming distance)
- `Morton10x10`: Z-order curve (24 char ELID, fast, sortable)
- `Hilbert10x10`: Hilbert curve (24 char ELID, best locality, slower)

**UniElidError enum:**
- `InvalidDimension`: Embedding dimension outside [64, 2048]
- `InvalidValue`: Embedding contains NaN or Inf
- `InvalidEncoding`: Malformed base32hex string
- `InvalidHeader`: Corrupted ELID header
- `ProfileMismatch`: Hamming distance requires both ELIDs to be Mini128
- `TransformNotFound`: Transform ID not found (future feature)

## Generating Language Bindings

UniFFI can generate bindings automatically from the UDL file. There are two approaches:

### Option 1: Using uniffi-bindgen CLI (Recommended for development)

Install the CLI tool (requires matching UniFFI version):

```bash
cargo install uniffi-bindgen --version 0.28  # Use latest compatible version
```

Generate bindings:

```bash
cd bindings/elid-ffi

# Swift bindings
uniffi-bindgen generate src/elid.udl --language swift --out-dir generated/swift

# Kotlin bindings
uniffi-bindgen generate src/elid.udl --language kotlin --out-dir generated/kotlin

# Ruby bindings
uniffi-bindgen generate src/elid.udl --language ruby --out-dir generated/ruby
```

### Option 2: Using the Compiled Library (Recommended for production)

The compiled library (`libelid_ffi.so`) contains all necessary UniFFI scaffolding. Language-specific tools can extract bindings automatically:

#### Swift

Use Swift Package Manager or Xcode. The library includes module maps and headers.

```swift
import Elid

let embedding: [Float] = Array(repeating: 0.1, count: 768)
do {
    let elid = try uniEncode(embedding: embedding, profile: .mini128)
    print("ELID: \(elid)")
} catch let error as UniElidError {
    print("Error: \(error)")
}
```

#### Kotlin

Use Gradle with JNA or native bindings:

```kotlin
import uniffi.elid_ffi.*

val embedding = FloatArray(768) { 0.1f }
try {
    val elid = uniEncode(embedding.toList(), UniProfile.MINI128)
    println("ELID: $elid")
} catch (e: UniElidException) {
    println("Error: ${e.message}")
}
```

#### Ruby

Use FFI gem:

```ruby
require 'ffi'

embedding = [0.1] * 768
begin
  elid = ElId::Ffi.uni_encode(embedding, :Mini128)
  puts "ELID: #{elid}"
rescue ElId::ElidError => e
  puts "Error: #{e.message}"
end
```

## Testing the UniFFI Layer

### Verify Library Exports

```bash
# Check exported symbols (Linux)
nm -D target/release/libelid_ffi.so | grep uniffi

# Check size and dependencies
ldd target/release/libelid_ffi.so
```

### Test with UDL

The UDL file can be validated:

```bash
# Check UDL syntax (requires uniffi-bindgen)
uniffi-bindgen generate src/elid.udl --language swift --out-dir /tmp/test-swift
```

## Coexistence with C Exports

The elid-ffi crate maintains **both** C-compatible exports (for PHP) and UniFFI exports (for Swift/Kotlin/Ruby):

### C Exports (for PHP FFI)
- `elid_encode()`
- `elid_decode()`
- `elid_hamming_distance()`
- `elid_free_string()`
- `elid_free_bytes()`

### UniFFI Exports (for Swift/Kotlin/Ruby)
- `uni_encode()`
- `uni_decode()`
- `uni_hamming_distance()`
- `uni_encode_batch()`

Both sets of exports are available in the same shared library without conflict.

## Next Steps

For language-specific agent implementation (T079-T085):

1. **Swift Agent** (T079-T080): Generate Swift bindings, create Xcode project
2. **Kotlin Agent** (T081-T082): Generate Kotlin bindings, create Gradle project
3. **Ruby Agent** (T083-T084): Generate Ruby bindings, create Gem structure

Each agent will:
- Generate language-specific bindings from UDL
- Write wrapper code for idiomatic usage
- Add tests using test vectors from `bindings/test-vectors.json`
- Create example programs demonstrating encode/decode/hamming_distance

## References

- [UniFFI Documentation](https://mozilla.github.io/uniffi-rs/)
- [UniFFI Tutorial](https://mozilla.github.io/uniffi-rs/tutorial/Rust_scaffolding.html)
- [UniFFI Examples](https://github.com/mozilla/uniffi-rs/tree/main/examples)
- [UDL Syntax Reference](https://mozilla.github.io/uniffi-rs/udl/)
