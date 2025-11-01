# ELID Flutter Bindings

Flutter bindings for **ELID** (Embedding Locality-preserving IDentifiers) - efficient encoding of embedding vectors into sortable string identifiers that preserve locality relationships.

[![pub package](https://img.shields.io/pub/v/elid.svg)](https://pub.dev/packages/elid)
[![CI](https://github.com/zachhandley/ELID/actions/workflows/bindings-flutter.yml/badge.svg)](https://github.com/zachhandley/ELID/actions)

## Features

- **Locality-Preserving**: Similar embeddings produce similar ELIDs (for SimHash)
- **Sortable**: String sort order preserves embedding space relationships (for Morton/Hilbert curves)
- **Compact**: 29 characters for 128-bit SimHash, 16-24 for space-filling curves
- **Fast**: Encode 5000+ embeddings per second on mobile devices
- **Zero-Copy**: Async APIs prevent UI blocking on large batches
- **Cross-Platform**: iOS, Android, macOS, Linux, Windows

## Installation

Add to your `pubspec.yaml`:

```yaml
dependencies:
  elid: ^0.1.0
```

Then run:

```bash
flutter pub get
```

## Quick Start

```dart
import 'package:elid/elid.dart';

void main() async {
  // Initialize (required once)
  await Elid.init();

  // Create an embedding (768D example)
  final embedding = List.generate(768, (i) => i * 0.001);

  // Encode to ELID
  final elidId = Elid.encode(embedding, Profile.mini128);
  print('ELID: $elidId'); // 29-character string

  // Decode back to bytes
  final bytes = Elid.decode(elidId);
  print('Bytes: ${bytes.length}'); // 18 bytes (2 header + 16 payload)

  // Check similarity
  final otherEmbedding = List.generate(768, (i) => i * 0.0015);
  final otherId = Elid.encode(otherEmbedding, Profile.mini128);
  final distance = Elid.hammingDistance(elidId, otherId);
  print('Hamming distance: $distance'); // Lower = more similar
}
```

## Encoding Profiles

### Mini128 (SimHash)
- **Use case**: Similarity search, approximate nearest neighbors
- **Properties**: 128-bit SimHash, 29 characters
- **Distance**: Hamming distance (0-128)

```dart
final elid = Elid.encode(embedding, Profile.mini128);
```

### Morton10x10 (Z-Order Curve)
- **Use case**: Fast spatial indexing, database queries
- **Properties**: 10D Morton encoding, 16-24 characters
- **Distance**: Not comparable via Hamming distance

```dart
final elid = Elid.encode(embedding, Profile.morton10x10);
```

### Hilbert10x10 (Hilbert Curve)
- **Use case**: Maximum locality preservation
- **Properties**: 10D Hilbert encoding, 16-24 characters, 5-10% better locality than Morton
- **Distance**: Not comparable via Hamming distance

```dart
final elid = Elid.encode(embedding, Profile.hilbert10x10);
```

## Async APIs

### Batch Encoding

Process large batches without blocking the UI:

```dart
final embeddings = List.generate(5000, (_) => List.filled(768, 0.5));
final elids = await Elid.encodeBatch(embeddings, Profile.mini128);
print('Encoded ${elids.length} embeddings');
```

### Streaming Encoding

Get progressive results for UI updates:

```dart
final embeddings = List.generate(1000, (_) => List.filled(768, 0.5));

await for (final elid in Elid.encodeStream(embeddings, Profile.mini128)) {
  print('Encoded: $elid');
  // Update progress indicator
}
```

## Performance

| Operation              | Time (avg)       | Notes                              |
|------------------------|------------------|------------------------------------|
| `encode()`             | <1ms             | 768D embedding, synchronous        |
| `encodeBatch(5000)`    | <10s             | Mid-range mobile device            |
| `hammingDistance()`    | <100μs           | Bitwise XOR + popcount             |
| `decode()`             | <500μs           | Base32hex decoding                 |

**Memory**: Embeddings are copied to Rust (unavoidable with flutter_rust_bridge).

## Platform Support

| Platform | Architectures              | Status      |
|----------|----------------------------|-------------|
| iOS      | arm64, x86_64 (simulator)  | ✅ Supported |
| Android  | arm64-v8a, armeabi-v7a, x64| ✅ Supported |
| macOS    | arm64, x86_64              | ✅ Supported |
| Linux    | x86_64                     | ✅ Supported |
| Windows  | x86_64                     | ✅ Supported |

## Example App

See [`example/lib/main.dart`](example/lib/main.dart) for a complete Flutter app demonstrating:
- Batch encoding with progress
- Real-time similarity search
- Performance benchmarking

Run the example:

```bash
cd example
flutter run
```

## API Reference

### Elid.init()

Initialize the ELID library. Must be called once before using other functions.

```dart
await Elid.init();
```

### Elid.encode()

Encode a single embedding (synchronous).

```dart
String encode(List<double> embedding, Profile profile)
```

- **embedding**: List of doubles, length 64-2048
- **profile**: `Profile.mini128`, `Profile.morton10x10`, or `Profile.hilbert10x10`
- **Returns**: ELID string (29 chars for Mini128, 16-24 for others)
- **Throws**: `ElidException` if dimensions invalid

### Elid.decode()

Decode ELID string to raw bytes (synchronous).

```dart
Uint8List decode(String elid)
```

- **elid**: ELID string identifier
- **Returns**: `Uint8List` including 2-byte header (18 bytes for Mini128; 15 for Morton/Hilbert)
- **Throws**: `ElidException` if ELID malformed

### Elid.hammingDistance()

Calculate Hamming distance between two ELIDs (synchronous).

```dart
int hammingDistance(String elid1, String elid2)
```

- **elid1, elid2**: ELID strings (must use same profile)
- **Returns**: Hamming distance (0-128 for Mini128)
- **Throws**: `ElidException` if profiles incompatible

### Elid.encodeBatch()

Encode multiple embeddings asynchronously.

```dart
Future<List<String>> encodeBatch(List<List<double>> embeddings, Profile profile)
```

- **embeddings**: List of embedding vectors
- **profile**: Encoding profile
- **Returns**: Future resolving to list of ELID strings

### Elid.encodeStream()

Stream-based encoding for progressive results.

```dart
Stream<String> encodeStream(List<List<double>> embeddings, Profile profile)
```

- **embeddings**: List of embedding vectors
- **profile**: Encoding profile
- **Returns**: Stream emitting ELID strings as encoded

## Error Handling

All functions throw `ElidException` on errors:

```dart
try {
  final elid = Elid.encode(embedding, Profile.mini128);
} on ElidException catch (e) {
  print('Error: ${e.message}');
}
```

Common errors:
- **Invalid dimensions**: Embedding not in 64-2048 range
- **Malformed ELID**: Invalid base32hex string
- **Profile mismatch**: Comparing ELIDs from different profiles

## Contributing

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for development setup and guidelines.

## License

Dual-licensed under MIT or Apache 2.0.

## References

- **Core Implementation**: [elid-core](../../elid-core)
- **SimHash Paper**: Charikar, M. (2002). "Similarity estimation techniques from rounding algorithms"
- **Space-Filling Curves**: [Hilbert Curve Research](../../HILBERT_CURVE_RESEARCH.md)
