# ELID Language Bindings

This directory contains language bindings for the ELID core library, enabling encoding and decoding of locality-preserving embedding identifiers across multiple programming languages.

## Available Bindings

### Phase 1: High-Priority Languages

- **elid-python/** - Python bindings via PyO3
  - Zero-copy NumPy array support
  - Batch processing with Rayon parallelism
  - Target: PyPI distribution

- **elid-node/** - TypeScript/Node.js bindings via napi-rs
  - TypedArray support with automatic TypeScript definitions
  - Async batch processing
  - WASM fallback for browsers
  - Target: npm distribution

- **elid-flutter/** - Dart/Flutter bindings via flutter_rust_bridge
  - Async APIs for non-blocking UI operations
  - Stream support for progressive results
  - Cross-platform: iOS, Android, Desktop, Web
  - Target: pub.dev distribution

### Phase 2: Native Platform Languages (UniFFI)

- **elid-ffi/** - Shared FFI layer using UniFFI
  - Generates bindings for Swift, Kotlin, and Ruby from single .udl interface
  - Automatic memory management via Arc<T>
  - Type-safe error handling

- **elid-swift/** - Swift package for iOS/macOS
  - Swift Package Manager support
  - XCFramework distribution
  - CocoaPods support

- **elid-kotlin/** - Kotlin library for Android
  - Gradle integration
  - AAR distribution
  - Maven Central publishing

- **elid-ruby/** - Ruby gem
  - Native extension
  - RubyGems distribution
  - Rails integration support

### Phase 3: Web Platform

- **elid-php/** - PHP bindings via FFI
  - C-compatible interface
  - Composer package
  - WordPress plugin integration

## FFI Type Mappings

All bindings expose three core functions with language-specific type mappings:

| Function | Input | Output | Notes |
|----------|-------|--------|-------|
| `encode` | Embedding array (f64), Profile enum | String (ELID) | 29-char base32hex for Mini128 |
| `decode` | String (ELID) | Bytes | Raw binary representation |
| `hamming_distance` | ELID string, ELID string | u32 | Requires matching profiles |
| `encode_batch` | Array of embeddings, Profile | Array of strings | Parallel processing |

### Language-Specific Types

- **Python**: `numpy.ndarray[float64]` → `str`
- **TypeScript**: `Float64Array` → `string`
- **Dart/Flutter**: `List<double>` → `String`
- **Swift**: `[Double]` → `String`
- **Kotlin**: `List<Double>` → `String`
- **Ruby**: `Array` → `String`
- **PHP**: `array` → `string`

## Cross-Language Compatibility

All bindings produce **byte-identical** ELID output for identical embedding input, ensuring cross-platform determinism. This is validated via:

1. **Reference test vectors** (`test-vectors.json`) generated from elid-core
2. **Cross-language test suite** (`scripts/cross-language-test.sh`)
3. **CI validation** on Linux, macOS, Windows, iOS, Android

## Performance Targets

- **Python**: Encoding throughput within 10% of Rust baseline (>5000 embeddings/sec for 768D vectors)
- **TypeScript**: Native addon overhead <5%
- **Flutter**: 5000 embeddings encoded in <10s on mid-range mobile device
- **Other bindings**: <10% overhead vs. Rust

## Building from Source

### Python
```bash
cd bindings/elid-python
maturin build --release
pip install target/wheels/*.whl
```

### TypeScript/Node.js
```bash
cd bindings/elid-node
npm install
npm run build
```

### Flutter
```bash
cd bindings/elid-flutter
flutter pub get
flutter build
```

### UniFFI-based (Swift/Kotlin/Ruby)
```bash
# Build shared FFI layer
cd bindings/elid-ffi
cargo build --release

# Language-specific package builds
cd ../elid-swift && swift build
cd ../elid-kotlin && ./gradlew build
cd ../elid-ruby && gem build elid.gemspec
```

### PHP
```bash
cd bindings/elid-php
composer install
```

## Testing

Each binding includes comprehensive tests:

- **Unit tests**: Language-specific test frameworks (pytest, vitest, flutter_test, XCTest, JUnit, RSpec, PHPUnit)
- **Integration tests**: Validate against reference test vectors
- **Cross-language tests**: Ensure byte-identical output across all bindings
- **Performance tests**: Validate encoding throughput meets targets

Run all tests:
```bash
# From repository root
./scripts/cross-language-test.sh
```

## CI/CD

Each binding has a dedicated GitHub Actions workflow for multi-platform builds:

- `.github/workflows/bindings-python.yml` - PyPI wheels (Linux, macOS, Windows)
- `.github/workflows/bindings-node.yml` - npm packages (all platforms)
- `.github/workflows/bindings-flutter.yml` - pub.dev package (iOS, Android, Desktop)
- `.github/workflows/bindings-uniffi.yml` - Swift/Kotlin/Ruby artifacts
- `.github/workflows/bindings-php.yml` - Composer package
- `.github/workflows/cross-language-test.yml` - Full compatibility suite

## Documentation

Each binding directory contains:
- **README.md**: Installation, usage examples, and API documentation
- **examples/**: Practical integration examples
- Type definitions (`.pyi`, `.d.ts`, etc.) for IDE autocomplete

## Contributing

When adding new bindings:

1. Create binding crate in `bindings/{language}/`
2. Implement wrapper around `elid-core` functions
3. Add tests validating against `test-vectors.json`
4. Configure CI workflow for multi-platform builds
5. Add cross-language test to `scripts/cross-language-test.sh`
6. Update this README with language-specific details

## License

All bindings inherit the dual MIT/Apache-2.0 license from elid-core.

## References

- [elid-core documentation](../elid-core/README.md)
- [PyO3 documentation](https://pyo3.rs/)
- [napi-rs documentation](https://napi.rs/)
- [flutter_rust_bridge documentation](https://cjycode.com/flutter_rust_bridge/)
- [UniFFI documentation](https://mozilla.github.io/uniffi-rs/)
- [PHP FFI documentation](https://www.php.net/manual/en/book.ffi.php)
