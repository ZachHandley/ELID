# ELID Swift Package

Swift bindings for ELID (Embedding Locality-preserving IDentifier) using UniFFI. This package provides idiomatic Swift APIs for encoding embeddings into sortable, locality-preserving identifiers for iOS and macOS applications.

## Features

- **Native Swift API**: UniFFI-generated Swift bindings with automatic memory management
- **Multiple Encoding Profiles**:
  - `Mini128`: SimHash-128 for 64-2048 dimensional embeddings
  - `Morton10x10`: Z-order curve encoding for 10D embeddings
  - `Hilbert10x10`: Hilbert curve encoding for 10D embeddings (better locality)
- **Batch Processing**: Parallel encoding of multiple embeddings
- **Hamming Distance**: Fast similarity computation for Mini128 ELIDs
- **XCFramework Distribution**: Universal binary for iOS and macOS (arm64 + x86_64)

## Requirements

- iOS 13.0+ / macOS 10.15+
- Swift 5.9+
- Xcode 15.0+ (for building)

## Installation

### Swift Package Manager

Add the following to your `Package.swift`:

```swift
dependencies: [
    .package(url: "https://github.com/your-org/ELID.git", from: "0.1.0")
]
```

Or in Xcode:
1. File → Add Package Dependencies
2. Enter the repository URL
3. Select version and add to your target

### Manual XCFramework Installation

1. Download `ElidFFI.xcframework` from the releases page
2. Drag it into your Xcode project
3. Ensure "Embed & Sign" is selected

## Quick Start

```swift
import Elid

// Create an embedding (e.g., from CoreML model output)
let embedding: [Float] = [0.1, 0.2, ..., 0.99] // 128 dimensions

// Encode to ELID
let elid = try uniEncode(embedding: embedding, profile: .mini128)
print("ELID: \(elid)") // Output: "0123456789ABCDEFGHIJKLMNOP"

// Decode ELID back to bytes
let bytes = try uniDecode(elid: elid)

// Compute Hamming distance between two ELIDs
let elid2 = try uniEncode(embedding: anotherEmbedding, profile: .mini128)
let distance = try uniHammingDistance(elid1: elid, elid2: elid2)
print("Hamming distance: \(distance)") // 0-128

// Batch encode multiple embeddings
let embeddings: [[Float]] = [[...], [...], [...]]
let elids = try uniEncodeBatch(embeddings: embeddings, profile: .mini128)
```

## API Reference

### Encoding Profiles

```swift
enum UniProfile {
    case mini128       // SimHash-128: 64-2048 dimensions → 26-char ELID
    case morton10x10   // Morton curve: 10 dimensions → 20-char ELID
    case hilbert10x10  // Hilbert curve: 10 dimensions → 20-char ELID
}
```

### Functions

#### `uniEncode(embedding:profile:)`
Encode a single embedding into an ELID string.

```swift
func uniEncode(embedding: [Float], profile: UniProfile) throws -> String
```

**Parameters:**
- `embedding`: Array of Float values (dimensions depend on profile)
- `profile`: Encoding profile (`.mini128`, `.morton10x10`, `.hilbert10x10`)

**Returns:** Base32hex-encoded ELID string (uppercase, no padding)

**Throws:**
- `UniElidError.InvalidDimension`: Wrong number of dimensions
- `UniElidError.InvalidValue`: NaN or infinite values

---

#### `uniDecode(elid:)`
Decode an ELID string back to raw bytes.

```swift
func uniDecode(elid: String) throws -> [UInt8]
```

**Parameters:**
- `elid`: ELID string to decode

**Returns:** Byte array including 2‑byte header (18 bytes for Mini128; 15 bytes for Morton/Hilbert)

**Throws:**
- `UniElidError.InvalidEncoding`: Invalid base32hex characters
- `UniElidError.InvalidHeader`: Unrecognized profile header

---

#### `uniHammingDistance(elid1:elid2:)`
Compute Hamming distance between two Mini128 ELIDs (0-128).

```swift
func uniHammingDistance(elid1: String, elid2: String) throws -> UInt32
```

**Parameters:**
- `elid1`: First ELID (must be Mini128)
- `elid2`: Second ELID (must be Mini128)

**Returns:** Hamming distance (0 = identical, 128 = opposite)

**Throws:**
- `UniElidError.ProfileMismatch`: ELIDs use different profiles
- `UniElidError.InvalidEncoding`: Invalid ELID strings

---

#### `uniEncodeBatch(embeddings:profile:)`
Encode multiple embeddings in parallel (Rayon-based).

```swift
func uniEncodeBatch(embeddings: [[Float]], profile: UniProfile) throws -> [String]
```

**Parameters:**
- `embeddings`: Array of embeddings
- `profile`: Encoding profile

**Returns:** Array of ELID strings (same order as input)

**Throws:** Same as `uniEncode`

---

### Error Types

```swift
enum UniElidError: Error {
    case InvalidDimension   // Wrong embedding dimensions
    case InvalidValue       // NaN or infinite values
    case InvalidEncoding    // Malformed ELID string
    case InvalidHeader      // Unknown profile in ELID
    case ProfileMismatch    // Operations require same profile
    case TransformNotFound  // Internal error (should not occur)
}
```

## Usage Examples

### CoreML Integration

See [Examples/CoreMLIntegration.swift](Examples/CoreMLIntegration.swift) for a complete example of using ELID with CoreML embeddings.

```swift
import CoreML
import Elid

// Load a CoreML model that outputs embeddings
let model = try YourEmbeddingModel(configuration: MLModelConfiguration())

// Process input and get embedding
let input = YourEmbeddingModelInput(image: image)
let output = try model.prediction(input: input)

// Assuming output.embedding is an MLMultiArray with shape [1, 128]
let embedding: [Float] = (0..<128).map {
    Float(truncating: output.embedding[$0])
}

// Encode to ELID
let elid = try uniEncode(embedding: embedding, profile: .mini128)

// Store in database with sortable index
let record = DatabaseRecord(id: elid, metadata: metadata)
try database.insert(record)

// Query for nearest neighbors using Hamming distance
let neighbors = try database.query(
    where: { record in
        try uniHammingDistance(elid1: record.id, elid2: targetElid) < 20
    }
)
```

### Vector Search in SwiftData

```swift
import SwiftData
import Elid

@Model
class EmbeddingRecord {
    @Attribute(.unique) var id: String // ELID
    var text: String
    var timestamp: Date

    init(text: String, embedding: [Float]) throws {
        self.id = try uniEncode(embedding: embedding, profile: .mini128)
        self.text = text
        self.timestamp = Date()
    }
}

// Query with lexicographic range (exploits sortability)
let prefix = "01" // Common ELID prefix
let records = try context.fetch(
    FetchDescriptor<EmbeddingRecord>(
        predicate: #Predicate { $0.id.hasPrefix(prefix) }
    )
)
```

### Image Similarity Search

```swift
import Vision
import Elid

// Generate embeddings using Vision framework
func generateEmbedding(for image: UIImage) async throws -> [Float] {
    let request = VNGenerateImageFeaturePrintRequest()
    let handler = VNImageRequestHandler(cgImage: image.cgImage!)
    try handler.perform([request])

    guard let observation = request.results?.first else {
        throw ImageError.noFeatures
    }

    // Convert VNFeaturePrintObservation to [Float]
    let featureData = observation.data
    let count = featureData.count / MemoryLayout<Float>.stride
    return featureData.withUnsafeBytes {
        Array(UnsafeBufferPointer<Float>(
            start: $0.baseAddress!.assumingMemoryBound(to: Float.self),
            count: count
        ))
    }
}

// Find similar images
let queryEmbedding = try await generateEmbedding(for: queryImage)
let queryElid = try uniEncode(embedding: queryEmbedding, profile: .mini128)

let similarImages = images
    .map { (image, elid) in
        (image, try! uniHammingDistance(elid1: queryElid, elid2: elid))
    }
    .filter { $0.1 < 30 } // Distance threshold
    .sorted { $0.1 < $1.1 } // Nearest first
```

## Building from Source

### Prerequisites

1. Install Rust:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Install UniFFI bindgen:
   ```bash
   cargo install uniffi-bindgen --version 0.25
   ```

3. Add Apple targets:
   ```bash
   rustup target add aarch64-apple-ios x86_64-apple-ios aarch64-apple-ios-sim
   rustup target add aarch64-apple-darwin x86_64-apple-darwin
   ```

### Build XCFramework

```bash
cd bindings/elid-swift
./build-xcframework.sh
```

This creates `ElidFFI.xcframework` with:
- iOS arm64 (device)
- iOS x86_64 + arm64 (simulator)
- macOS x86_64 + arm64 (universal)

### Run Tests

```bash
swift test
```

## Performance

Benchmarks on M1 MacBook Pro:

| Operation              | Throughput          |
|------------------------|---------------------|
| Mini128 encoding       | ~100K embeddings/s  |
| Batch encoding (100)   | ~250K embeddings/s  |
| Hamming distance       | ~2M comparisons/s   |
| Morton encoding        | ~150K embeddings/s  |
| Hilbert encoding       | ~140K embeddings/s  |

## Cross-Language Compatibility

ELID bindings are available for multiple languages with **byte-identical output**:

- **Python**: `pip install elid-py`
- **TypeScript/Node.js**: `npm install elid-node`
- **Dart/Flutter**: `flutter pub add elid_flutter`
- **Swift** (this package)
- **Kotlin/JVM**: Coming soon
- **Ruby**: Coming soon

All implementations share the same test vectors and produce identical ELIDs for the same embeddings.

## Architecture

```
┌─────────────────────┐
│   Swift Package     │
│   (Elid module)     │
└──────────┬──────────┘
           │ Import
           ▼
┌─────────────────────┐
│  UniFFI Generated   │
│  Swift Bindings     │
│  (Elid.swift)       │
└──────────┬──────────┘
           │ Calls
           ▼
┌─────────────────────┐
│   XCFramework       │
│  (ElidFFI.xcframework) │
│   - iOS arm64       │
│   - iOS sim         │
│   - macOS universal │
└──────────┬──────────┘
           │ FFI
           ▼
┌─────────────────────┐
│   Rust Library      │
│   (elid-ffi)        │
│   via UniFFI        │
└──────────┬──────────┘
           │ Links to
           ▼
┌─────────────────────┐
│   Core Library      │
│   (elid-core)       │
│   - SimHash         │
│   - Morton/Hilbert  │
└─────────────────────┘
```

## Troubleshooting

### Module 'ElidFFI' not found

Ensure the XCFramework is properly embedded in your project:
1. Select your target in Xcode
2. Go to "Frameworks, Libraries, and Embedded Content"
3. Verify ElidFFI.xcframework is set to "Embed & Sign"

### Invalid ELID format

ELIDs use RFC 4648 base32hex encoding (uppercase, no padding). Valid characters: `0-9A-V`

### Hamming distance crashes

Hamming distance only works with Mini128 ELIDs. Calling it with Morton or Hilbert ELIDs will throw `ProfileMismatch`.

## Contributing

This is a generated binding package. To contribute:

1. Core algorithm changes: [elid-core](../../elid-core)
2. FFI interface: [elid-ffi](../elid-ffi)
3. Swift wrapper improvements: Submit PR with Swift-specific enhancements

## License

Dual-licensed under MIT or Apache-2.0.

## References

- [ELID Specification](../../README.md)
- [UniFFI Documentation](https://mozilla.github.io/uniffi-rs/)
- [Test Vectors](../test-vectors.json)

## Support

- Issues: [GitHub Issues](https://github.com/your-org/ELID/issues)
- Discussions: [GitHub Discussions](https://github.com/your-org/ELID/discussions)
